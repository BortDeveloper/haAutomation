//! authgate — kleines, eigenstaendiges Forward-Auth-Sidecar.
//!
//! Behelfsloesung fuer den Fall, dass noch kein externes SSO (Authentik)
//! bereitsteht. Caddy ruft `authgate` per `forward_auth` und injiziert bei
//! Erfolg den `X-Authentik-Username`-Header — dieselbe Vertragsschnittstelle,
//! die spaeter Authentik bedient. Der Wechsel ist ein Einzeiler im Caddyfile.
//!
//! Ablauf:
//!   Browser -> Caddy --forward_auth--> authgate `/auth/verify`
//!                       200 + X-Authentik-Username  -> Caddy proxyt inventory
//!                       302 -> /auth/login          -> Browser (HTML-Clients)
//!                       401                         -> curl/API-Clients
//!
//! Routen (alle unter `/auth/`, damit Caddy sie per Prefix an authgate gibt):
//!   GET|*  /auth/verify   Forward-Auth-Pruefung (Session-Cookie)
//!   GET    /auth/login    Login-Formular
//!   POST   /auth/login    Login-Submit -> setzt signiertes Session-Cookie
//!   GET|*  /auth/logout   Cookie loeschen
//!   GET    /auth/health   Healthcheck (public)
//!
//! Sessions sind zustandslos: das Cookie traegt `user|exp` und eine
//! HMAC-SHA256-Signatur. Kein State, keine DB.
//!
//! Subkommandos: `serve`, `hashpw <user>`, `gensecret`.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server};

type HmacSha256 = Hmac<Sha256>;
type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// Name des Session-Cookies.
const COOKIE_NAME: &str = "authgate_session";
/// PBKDF2-Iterationen fuer neu erzeugte Hashes (OWASP-Empfehlung 2023 fuer
/// PBKDF2-HMAC-SHA256). Pro Datensatz gespeichert, alte Hashes bleiben gueltig.
const PBKDF2_ROUNDS: u32 = 600_000;
const SALT_LEN: usize = 16;
const HASH_LEN: usize = 32;
/// Versionspraefix im Cookie — erlaubt spaetere Formatwechsel.
const COOKIE_VERSION: &str = "v1";

// ============================================================ CLI

#[derive(Parser)]
#[command(name = "authgate", version, about = "home-inventory forward-auth sidecar")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Startet den Forward-Auth-Server.
    Serve,
    /// Erzeugt einen Passwort-Datensatz `user=rounds$salt$hash` fuer AUTHGATE_USERS.
    /// Passwort via Stdin oder Env AUTHGATE_PW.
    Hashpw {
        /// Benutzername.
        user: String,
    },
    /// Gibt ein zufaelliges 32-Byte-Secret (hex) fuer AUTHGATE_SESSION_SECRET aus.
    Gensecret,
}

// ============================================================ Konfiguration

/// Ein Benutzer-Datensatz: PBKDF2-HMAC-SHA256 mit Salt.
struct UserRecord {
    name: String,
    rounds: u32,
    salt: Vec<u8>,
    hash: Vec<u8>,
}

/// Laufzeit-Konfiguration, beim Start aus Env gelesen.
struct Config {
    listen: String,
    /// Header, der bei Erfolg gesetzt wird — muss zu inventorys AUTH_HEADER passen.
    header: String,
    /// Session-Lebensdauer in Sekunden.
    ttl: u64,
    /// HMAC-Schluessel fuer Cookie-Signaturen.
    secret: Vec<u8>,
    users: Vec<UserRecord>,
    /// `Secure`-Attribut am Cookie (nur bei HTTPS sinnvoll).
    cookie_secure: bool,
}

impl Config {
    fn from_env() -> Result<Self> {
        let listen = env_or("AUTHGATE_LISTEN", "0.0.0.0:9000");
        let header = env_or("AUTHGATE_HEADER", "X-Authentik-Username");
        let ttl = env_or("AUTHGATE_SESSION_TTL", "28800")
            .parse::<u64>()
            .map_err(|e| anyhow!("AUTHGATE_SESSION_TTL ungueltig: {e}"))?;

        let secret = match std::env::var("AUTHGATE_SESSION_SECRET") {
            Ok(s) if !s.trim().is_empty() => {
                let bytes = s.trim().as_bytes().to_vec();
                if bytes.len() < 16 {
                    eprintln!(
                        "WARN: AUTHGATE_SESSION_SECRET ist kurz ({} Byte) — \
                         min. 32 empfohlen (siehe `authgate gensecret`)",
                        bytes.len()
                    );
                }
                bytes
            }
            _ => {
                eprintln!(
                    "WARN: AUTHGATE_SESSION_SECRET nicht gesetzt — erzeuge ein \
                     ephemeres Secret. Sessions ueberleben keinen Neustart."
                );
                random_bytes(32)
            }
        };

        let users = match std::env::var("AUTHGATE_USERS") {
            Ok(raw) => parse_users(&raw)?,
            Err(_) => Vec::new(),
        };

        let cookie_secure = !matches!(
            env_or("AUTHGATE_COOKIE_SECURE", "true").to_ascii_lowercase().as_str(),
            "false" | "0" | "no"
        );

        Ok(Self { listen, header, ttl, secret, users, cookie_secure })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Parst `AUTHGATE_USERS`: `name=record` je Eintrag, mehrere mit `;` getrennt.
fn parse_users(raw: &str) -> Result<Vec<UserRecord>> {
    let mut out = Vec::new();
    for entry in raw.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (name, record) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("AUTHGATE_USERS-Eintrag ohne '=': {entry}"))?;
        out.push(parse_user_record(name.trim(), record.trim())?);
    }
    Ok(out)
}

/// Parst einen einzelnen Datensatz `rounds$salthex$hashhex`.
fn parse_user_record(name: &str, record: &str) -> Result<UserRecord> {
    if name.is_empty() || !name.bytes().all(is_username_byte) {
        return Err(anyhow!("ungueltiger Benutzername: {name:?}"));
    }
    let mut parts = record.split('$');
    let rounds = parts
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| anyhow!("Datensatz {name}: rounds fehlt/ungueltig"))?;
    let salt = parts
        .next()
        .and_then(hex_decode)
        .ok_or_else(|| anyhow!("Datensatz {name}: salt fehlt/ungueltig"))?;
    let hash = parts
        .next()
        .and_then(hex_decode)
        .ok_or_else(|| anyhow!("Datensatz {name}: hash fehlt/ungueltig"))?;
    if hash.len() != HASH_LEN {
        return Err(anyhow!(
            "Datensatz {name}: hash hat {} Byte, erwartet {HASH_LEN}",
            hash.len()
        ));
    }
    if rounds == 0 {
        return Err(anyhow!("Datensatz {name}: rounds=0"));
    }
    Ok(UserRecord { name: name.to_string(), rounds, salt, hash })
}

/// Erlaubte Zeichen im Benutzernamen — bewusst eng, damit Cookie/Logs/Header
/// keine Sonderzeichen-Behandlung brauchen.
fn is_username_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'@')
}

// ============================================================ Krypto

/// HMAC-SHA256 ueber `msg` mit `secret`.
fn sign(secret: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC nimmt jede Keylaenge");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

/// Kanonische, signierte Cookie-Nutzlast.
fn cookie_payload(user: &str, exp: u64) -> String {
    format!("{COOKIE_VERSION}\n{user}\n{exp}")
}

/// Baut ein signiertes Cookie: `v1.<userhex>.<exp>.<machex>`.
fn make_cookie(secret: &[u8], user: &str, exp: u64) -> String {
    let mac = sign(secret, cookie_payload(user, exp).as_bytes());
    format!(
        "{COOKIE_VERSION}.{}.{exp}.{}",
        hex_encode(user.as_bytes()),
        hex_encode(&mac)
    )
}

/// Prueft ein Cookie und liefert bei Gueltigkeit den Benutzernamen.
/// Signatur wird konstantzeitig verglichen, Ablauf gegen `now` geprueft.
fn verify_cookie(secret: &[u8], value: &str, now: u64) -> Option<String> {
    let mut parts = value.split('.');
    if parts.next()? != COOKIE_VERSION {
        return None;
    }
    let user = String::from_utf8(hex_decode(parts.next()?)?).ok()?;
    if user.is_empty() || !user.bytes().all(is_username_byte) {
        return None;
    }
    let exp: u64 = parts.next()?.parse().ok()?;
    let mac = hex_decode(parts.next()?)?;
    if parts.next().is_some() {
        return None; // ueberzaehlige Felder
    }
    let mut check = HmacSha256::new_from_slice(secret).expect("HMAC nimmt jede Keylaenge");
    check.update(cookie_payload(&user, exp).as_bytes());
    check.verify_slice(&mac).ok()?; // konstantzeitig
    if exp <= now {
        return None;
    }
    Some(user)
}

/// PBKDF2-HMAC-SHA256-Hash.
fn pbkdf2_hash(password: &[u8], salt: &[u8], rounds: u32) -> [u8; HASH_LEN] {
    pbkdf2_hmac_array::<Sha256, HASH_LEN>(password, salt, rounds)
}

/// Prueft ein Passwort gegen einen Datensatz (konstantzeitiger Vergleich).
fn verify_password(rec: &UserRecord, password: &str) -> bool {
    let computed = pbkdf2_hash(password.as_bytes(), &rec.salt, rec.rounds);
    ct_eq(&computed, &rec.hash)
}

/// Konstantzeitiger Byte-Vergleich.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// `n` kryptografisch zufaellige Bytes.
fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).expect("OS-RNG nicht verfuegbar");
    buf
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Systemzeit vor 1970")
        .as_secs()
}

// ============================================================ HTTP-Server

fn serve() -> Result<()> {
    let cfg = Config::from_env()?;
    let server = Server::http(&cfg.listen)
        .map_err(|e| anyhow!("bind {}: {}", cfg.listen, e))?;
    eprintln!(
        "authgate listening on {} (users: {}, cookie_secure: {}, ttl: {}s, header: {})",
        cfg.listen,
        cfg.users.len(),
        cfg.cookie_secure,
        cfg.ttl,
        cfg.header
    );
    if cfg.users.is_empty() {
        eprintln!("WARN: AUTHGATE_USERS leer — jeder Login schlaegt fehl (fail closed)");
    }
    for req in server.incoming_requests() {
        if let Err(e) = handle(req, &cfg) {
            eprintln!("handler error: {e:#}");
        }
    }
    Ok(())
}

fn handle(mut req: Request, cfg: &Config) -> Result<()> {
    let method = req.method().clone();
    let raw_url = req.url().to_string();
    let path = raw_url.split('?').next().unwrap_or("/");
    let query = raw_url.splitn(2, '?').nth(1).unwrap_or("").to_string();

    let resp = match (path, &method) {
        ("/auth/health", _) => text(200, "ok"),
        ("/auth/verify", _) => verify(&req, cfg),
        ("/auth/login", &Method::Get) => {
            let rd = safe_rd(&query_param(&query, "rd"));
            html(200, &login_page(&rd, None))
        }
        ("/auth/login", &Method::Post) => {
            let mut body = String::new();
            // Body-Lesefehler -> leerer Body -> Login schlaegt sauber fehl.
            let _ = req.as_reader().read_to_string(&mut body);
            login_submit(&body, cfg)
        }
        ("/auth/logout", _) => logout(cfg),
        _ => text(404, "not found"),
    };
    req.respond(resp)?;
    Ok(())
}

/// Forward-Auth-Pruefung. Caddy ruft das pro Request; die Antwort steuert,
/// ob Caddy zum Backend durchreicht.
fn verify(req: &Request, cfg: &Config) -> Resp {
    let session = header(req, "cookie").and_then(|c| cookie_value(c, COOKIE_NAME));
    let user = session
        .and_then(|v| verify_cookie(&cfg.secret, &v, now_unix()))
        // Benutzer muss noch existieren — Entfernen invalidiert offene Sessions.
        .filter(|u| cfg.users.iter().any(|r| &r.name == u));

    match user {
        Some(u) => Response::from_string("ok")
            .with_status_code(200)
            .with_header(
                Header::from_bytes(cfg.header.as_bytes(), u.as_bytes())
                    .expect("valider Headername"),
            ),
        None => {
            // Browser bekommt einen Login-Redirect, Maschinen ein klares 401.
            let wants_html = header(req, "accept")
                .map(|a| a.contains("text/html"))
                .unwrap_or(false);
            if wants_html {
                // Caddy reicht das urspruenglich angefragte Ziel als
                // X-Forwarded-Uri durch — dorthin nach dem Login zurueck.
                let target = header(req, "x-forwarded-uri").unwrap_or("/");
                redirect(302, &format!("/auth/login?rd={}", urlencode(target)))
            } else {
                text(401, "unauthorized")
            }
        }
    }
}

/// Verarbeitet das Login-Formular.
fn login_submit(body: &str, cfg: &Config) -> Resp {
    let form = parse_form(body);
    let field = |k: &str| {
        form.iter()
            .find(|(name, _)| name == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    let username = field("username");
    let password = field("password");
    let rd = safe_rd(&field("rd"));

    let ok = match cfg.users.iter().find(|r| r.name == username) {
        Some(rec) => verify_password(rec, &password),
        None => {
            // Dummy-Hash gegen User-Enumeration ueber Antwortzeiten.
            let _ = pbkdf2_hash(password.as_bytes(), b"authgate-dummy", PBKDF2_ROUNDS);
            false
        }
    };

    if !ok {
        // Modeste Brute-Force-Bremse.
        std::thread::sleep(Duration::from_millis(700));
        return html(401, &login_page(&rd, Some("Anmeldung fehlgeschlagen.")));
    }

    let exp = now_unix() + cfg.ttl;
    let cookie = make_cookie(&cfg.secret, &username, exp);
    redirect(302, &rd).with_header(set_cookie(cfg, &cookie, cfg.ttl as i64))
}

fn logout(cfg: &Config) -> Resp {
    redirect(302, "/auth/login").with_header(set_cookie(cfg, "", 0))
}

// ============================================================ HTML

fn login_page(rd: &str, error: Option<&str>) -> String {
    let err_block = match error {
        Some(msg) => format!("<p class=\"err\">{}</p>", html_escape(msg)),
        None => String::new(),
    };
    format!(
        r#"<!doctype html>
<html lang="de">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Home Inventory — Sign in</title>
<style>
  body {{ font-family: system-ui, sans-serif; background: #1b1f24; color: #e6e6e6;
         display: flex; min-height: 100vh; margin: 0; align-items: center;
         justify-content: center; }}
  form {{ background: #262b32; padding: 2rem; border-radius: 10px; width: 18rem;
          box-shadow: 0 6px 24px rgba(0,0,0,.4); }}
  h1 {{ font-size: 1.1rem; margin: 0 0 1.2rem; }}
  label {{ display: block; font-size: .8rem; margin: .7rem 0 .2rem; color: #9aa4af; }}
  input {{ width: 100%; box-sizing: border-box; padding: .55rem; border-radius: 6px;
           border: 1px solid #3a4049; background: #1b1f24; color: #e6e6e6; }}
  button {{ width: 100%; margin-top: 1.2rem; padding: .6rem; border: 0;
            border-radius: 6px; background: #3b82f6; color: #fff; font-weight: 600;
            cursor: pointer; }}
  .err {{ color: #f87171; font-size: .85rem; margin: 0 0 .5rem; }}
</style>
</head>
<body>
<form method="post" action="/auth/login">
  <h1>Home Inventory</h1>
  {err_block}
  <label for="u">Benutzer</label>
  <input id="u" name="username" autocomplete="username" autofocus required>
  <label for="p">Passwort</label>
  <input id="p" name="password" type="password" autocomplete="current-password" required>
  <input type="hidden" name="rd" value="{rd}">
  <button type="submit">Anmelden</button>
</form>
</body>
</html>
"#,
        err_block = err_block,
        rd = html_escape(rd),
    )
}

// ============================================================ Response-Helfer

fn text(status: u16, body: &str) -> Resp {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type("text/plain; charset=utf-8"))
}

fn html(status: u16, body: &str) -> Resp {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type("text/html; charset=utf-8"))
}

fn redirect(status: u16, location: &str) -> Resp {
    Response::from_string("")
        .with_status_code(status)
        .with_header(Header::from_bytes("Location", location).expect("valide Location"))
}

fn content_type(value: &str) -> Header {
    Header::from_bytes("Content-Type", value).expect("valider Content-Type")
}

/// Baut den `Set-Cookie`-Header. `max_age <= 0` loescht das Cookie.
fn set_cookie(cfg: &Config, value: &str, max_age: i64) -> Header {
    let mut s = format!(
        "{COOKIE_NAME}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}"
    );
    if cfg.cookie_secure {
        s.push_str("; Secure");
    }
    Header::from_bytes("Set-Cookie", s).expect("valider Set-Cookie")
}

// ============================================================ Parsing-Helfer

/// Case-insensitiver Header-Lookup (tiny_http kanonisiert nicht).
fn header<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers().iter().find_map(|h| {
        if h.field.as_str().as_str().eq_ignore_ascii_case(name) {
            Some(h.value.as_str())
        } else {
            None
        }
    })
}

/// Liest einen Cookie-Wert aus dem `Cookie`-Header.
fn cookie_value(header: &str, name: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(name) {
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parst einen `application/x-www-form-urlencoded`-Body.
fn parse_form(body: &str) -> Vec<(String, String)> {
    body.split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let mut it = pair.splitn(2, '=');
            let key = urldecode(it.next().unwrap_or(""));
            let value = urldecode(it.next().unwrap_or(""));
            (key, value)
        })
        .collect()
}

/// Holt einen Query-Parameter aus einem rohen Query-String.
fn query_param(query: &str, key: &str) -> String {
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        if it.next() == Some(key) {
            return urldecode(it.next().unwrap_or(""));
        }
    }
    String::new()
}

/// Open-Redirect-Schutz: nur lokale, absolute Pfade zulassen.
fn safe_rd(rd: &str) -> String {
    if rd.starts_with('/') && !rd.starts_with("//") && !rd.contains('\\') {
        rd.to_string()
    } else {
        "/".to_string()
    }
}

fn urldecode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => match (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                (Some(hi), Some(lo)) => {
                    out.push((hi << 4) | lo);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-Encoding fuer Query-Werte; `/` bleibt lesbar erhalten.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < b.len() {
        out.push((hex_val(b[i])? << 4) | hex_val(b[i + 1])?);
        i += 2;
    }
    Some(out)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ============================================================ Subkommandos

/// Liest ein Passwort aus AUTHGATE_PW oder interaktiv von Stdin.
fn read_password(user: &str) -> Result<String> {
    if let Ok(pw) = std::env::var("AUTHGATE_PW") {
        return Ok(pw);
    }
    eprint!("Passwort fuer {user} (Eingabe sichtbar): ");
    use std::io::Write;
    std::io::stderr().flush().ok();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let pw = line.trim_end_matches(['\n', '\r']).to_string();
    if pw.is_empty() {
        return Err(anyhow!("leeres Passwort"));
    }
    Ok(pw)
}

fn hashpw(user: &str) -> Result<()> {
    if user.is_empty() || !user.bytes().all(is_username_byte) {
        return Err(anyhow!(
            "ungueltiger Benutzername (erlaubt: a-z A-Z 0-9 . _ - @)"
        ));
    }
    let password = read_password(user)?;
    let salt = random_bytes(SALT_LEN);
    let hash = pbkdf2_hash(password.as_bytes(), &salt, PBKDF2_ROUNDS);
    println!(
        "{user}={PBKDF2_ROUNDS}${}${}",
        hex_encode(&salt),
        hex_encode(&hash)
    );
    eprintln!("# -> in AUTHGATE_USERS uebernehmen (mehrere Eintraege mit ';' trennen)");
    Ok(())
}

fn gensecret() -> Result<()> {
    println!("{}", hex_encode(&random_bytes(32)));
    eprintln!("# -> als AUTHGATE_SESSION_SECRET setzen");
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Serve => serve(),
        Cmd::Hashpw { user } => hashpw(&user),
        Cmd::Gensecret => gensecret(),
    }
}

// ============================================================ Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::net::TcpStream;
    use std::thread;

    const SECRET: &[u8] = b"test-secret-32-bytes-long-aaaaaa";

    // --- Krypto: Cookies ---

    #[test]
    fn cookie_roundtrip() {
        let exp = now_unix() + 100;
        let c = make_cookie(SECRET, "alice", exp);
        assert_eq!(verify_cookie(SECRET, &c, now_unix()).as_deref(), Some("alice"));
    }

    #[test]
    fn cookie_rejected_when_expired() {
        let c = make_cookie(SECRET, "alice", now_unix() - 1);
        assert_eq!(verify_cookie(SECRET, &c, now_unix()), None);
    }

    #[test]
    fn cookie_rejected_with_wrong_secret() {
        let c = make_cookie(SECRET, "alice", now_unix() + 100);
        assert_eq!(verify_cookie(b"other-secret-wrong-aaaaaaaaaaaaa", &c, now_unix()), None);
    }

    #[test]
    fn cookie_rejected_when_tampered() {
        let exp = now_unix() + 100;
        let c = make_cookie(SECRET, "alice", exp);
        // Benutzer faelschen, Rest unveraendert -> MAC passt nicht mehr.
        let forged = format!(
            "{COOKIE_VERSION}.{}.{exp}.{}",
            hex_encode(b"admin"),
            c.rsplit('.').next().unwrap()
        );
        assert_eq!(verify_cookie(SECRET, &forged, now_unix()), None);
    }

    #[test]
    fn cookie_rejected_when_garbage() {
        for bad in ["", "v1", "v2.616c.1.ff", "not-a-cookie", "v1.zz.1.ff"] {
            assert_eq!(verify_cookie(SECRET, bad, now_unix()), None, "bad={bad}");
        }
    }

    // --- Krypto: Passwoerter ---

    fn record(name: &str, pw: &str) -> UserRecord {
        let salt = random_bytes(SALT_LEN);
        // Wenige Runden im Test — Sicherheit kommt aus dem Algorithmus, nicht der Zeit.
        let hash = pbkdf2_hash(pw.as_bytes(), &salt, 1000).to_vec();
        UserRecord { name: name.to_string(), rounds: 1000, salt, hash }
    }

    #[test]
    fn password_verifies_correct() {
        let rec = record("alice", "korrekt-pferd-batterie");
        assert!(verify_password(&rec, "korrekt-pferd-batterie"));
    }

    #[test]
    fn password_rejects_wrong() {
        let rec = record("alice", "korrekt-pferd-batterie");
        assert!(!verify_password(&rec, "falsch"));
        assert!(!verify_password(&rec, ""));
    }

    #[test]
    fn user_record_parse_roundtrip() {
        let salt = random_bytes(SALT_LEN);
        let hash = pbkdf2_hash(b"pw", &salt, 5000);
        let line = format!("bob=5000${}${}", hex_encode(&salt), hex_encode(&hash));
        let users = parse_users(&line).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "bob");
        assert_eq!(users[0].rounds, 5000);
        assert!(verify_password(&users[0], "pw"));
    }

    #[test]
    fn user_record_parse_rejects_bad_input() {
        assert!(parse_users("noequalsign").is_err());
        assert!(parse_users("u=notanumber$ab$cd").is_err());
        assert!(parse_users("u=1000$xy$cd").is_err()); // salt kein hex
        assert!(parse_users("bad name=1000$ab$cd").is_err()); // Leerzeichen im Namen
        // Hash zu kurz:
        assert!(parse_users("u=1000$abcd$abcd").is_err());
    }

    #[test]
    fn multiple_users_parse() {
        let s1 = random_bytes(SALT_LEN);
        let s2 = random_bytes(SALT_LEN);
        let raw = format!(
            "a=1000${}${} ; b=1000${}${}",
            hex_encode(&s1),
            hex_encode(&pbkdf2_hash(b"pw-a", &s1, 1000)),
            hex_encode(&s2),
            hex_encode(&pbkdf2_hash(b"pw-b", &s2, 1000)),
        );
        let users = parse_users(&raw).unwrap();
        assert_eq!(users.len(), 2);
    }

    // --- Parsing-Helfer ---

    #[test]
    fn safe_rd_blocks_open_redirects() {
        assert_eq!(safe_rd("/api/devices"), "/api/devices");
        assert_eq!(safe_rd("/"), "/");
        assert_eq!(safe_rd("//evil.com"), "/");
        assert_eq!(safe_rd("https://evil.com"), "/");
        assert_eq!(safe_rd("/\\evil.com"), "/");
        assert_eq!(safe_rd("evil.com"), "/");
        assert_eq!(safe_rd(""), "/");
    }

    #[test]
    fn urldecode_handles_percent_and_plus() {
        assert_eq!(urldecode("a%2Fb+c"), "a/b c");
        assert_eq!(urldecode("%41%42"), "AB");
        assert_eq!(urldecode("plain"), "plain");
    }

    #[test]
    fn urlencode_roundtrips_through_decode() {
        let original = "/api/devices?q=a b&x=1";
        assert_eq!(urldecode(&urlencode(original)), original);
    }

    #[test]
    fn parse_form_extracts_fields() {
        let form = parse_form("username=alice&password=p%40ss+word&rd=%2Fhome");
        let get = |k: &str| form.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        assert_eq!(get("username"), Some("alice"));
        assert_eq!(get("password"), Some("p@ss word"));
        assert_eq!(get("rd"), Some("/home"));
    }

    #[test]
    fn cookie_value_extracted_from_header() {
        let h = "foo=bar; authgate_session=abc.def; other=1";
        assert_eq!(cookie_value(h, COOKIE_NAME).as_deref(), Some("abc.def"));
        assert_eq!(cookie_value("foo=bar", COOKIE_NAME), None);
    }

    #[test]
    fn hex_roundtrip() {
        let data = random_bytes(20);
        assert_eq!(hex_decode(&hex_encode(&data)).unwrap(), data);
        assert_eq!(hex_decode("odd"), None);
        assert_eq!(hex_decode("zz"), None);
    }

    // --- HTTP-Integration (roher TCP-Client, dependencyfrei) ---

    struct TestResponse {
        status: u16,
        headers: Vec<(String, String)>,
        body: String,
    }

    impl TestResponse {
        fn header(&self, name: &str) -> Option<&str> {
            self.headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_str())
        }
    }

    /// Schickt einen rohen HTTP/1.1-Request und parst die Antwort.
    fn request(addr: &str, method: &str, path: &str, extra: &[(&str, &str)], body: &str) -> TestResponse {
        let mut stream = TcpStream::connect(addr).unwrap();
        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n"
        );
        for (k, v) in extra {
            req.push_str(&format!("{k}: {v}\r\n"));
        }
        if !body.is_empty() {
            req.push_str(&format!(
                "Content-Type: application/x-www-form-urlencoded\r\n\
                 Content-Length: {}\r\n",
                body.len()
            ));
        }
        req.push_str("\r\n");
        req.push_str(body);
        stream.write_all(req.as_bytes()).unwrap();

        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        let text = String::from_utf8_lossy(&raw).into_owned();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        let mut lines = head.split("\r\n");
        let status = lines
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap();
        let headers = lines
            .filter_map(|l| l.split_once(": "))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        TestResponse { status, headers, body: body.to_string() }
    }

    fn spawn(users_env: &str) -> (String, Config) {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        let cfg = Config {
            listen: addr.clone(),
            header: "X-Authentik-Username".to_string(),
            ttl: 3600,
            secret: SECRET.to_vec(),
            users: parse_users(users_env).unwrap(),
            cookie_secure: false,
        };
        // Zweite, identische Config fuer den Server-Thread.
        let server_cfg = Config {
            listen: cfg.listen.clone(),
            header: cfg.header.clone(),
            ttl: cfg.ttl,
            secret: cfg.secret.clone(),
            users: parse_users(users_env).unwrap(),
            cookie_secure: cfg.cookie_secure,
        };
        thread::spawn(move || {
            for req in server.incoming_requests() {
                let _ = handle(req, &server_cfg);
            }
        });
        (addr, cfg)
    }

    /// Erzeugt einen `AUTHGATE_USERS`-Eintrag fuer Tests.
    fn user_env(name: &str, pw: &str) -> String {
        let salt = random_bytes(SALT_LEN);
        format!(
            "{name}=1000${}${}",
            hex_encode(&salt),
            hex_encode(&pbkdf2_hash(pw.as_bytes(), &salt, 1000))
        )
    }

    #[test]
    fn health_is_public() {
        let (addr, _) = spawn(&user_env("alice", "pw"));
        let r = request(&addr, "GET", "/auth/health", &[], "");
        assert_eq!(r.status, 200);
        assert_eq!(r.body, "ok");
    }

    #[test]
    fn verify_without_cookie_redirects_browser() {
        let (addr, _) = spawn(&user_env("alice", "pw"));
        let r = request(
            &addr,
            "GET",
            "/auth/verify",
            &[("Accept", "text/html"), ("X-Forwarded-Uri", "/api/devices")],
            "",
        );
        assert_eq!(r.status, 302);
        let loc = r.header("Location").unwrap();
        assert!(loc.starts_with("/auth/login?rd="), "loc={loc}");
        assert!(loc.contains("%2Fapi%2Fdevices"), "loc={loc}");
    }

    #[test]
    fn verify_without_cookie_returns_401_for_api() {
        let (addr, _) = spawn(&user_env("alice", "pw"));
        // Kein text/html im Accept -> Maschine -> 401, kein Redirect.
        let r = request(&addr, "GET", "/auth/verify", &[("Accept", "application/json")], "");
        assert_eq!(r.status, 401);
    }

    #[test]
    fn login_get_serves_form() {
        let (addr, _) = spawn(&user_env("alice", "pw"));
        let r = request(&addr, "GET", "/auth/login", &[], "");
        assert_eq!(r.status, 200);
        assert!(r.body.contains("<form"));
        assert!(r.body.contains("name=\"password\""));
    }

    #[test]
    fn login_post_valid_sets_cookie_and_redirects() {
        let (addr, _) = spawn(&user_env("alice", "geheim123"));
        let r = request(
            &addr,
            "POST",
            "/auth/login",
            &[],
            "username=alice&password=geheim123&rd=%2Fapi%2Fdevices",
        );
        assert_eq!(r.status, 302);
        assert_eq!(r.header("Location"), Some("/api/devices"));
        let sc = r.header("Set-Cookie").expect("Set-Cookie fehlt");
        assert!(sc.starts_with("authgate_session="));
        assert!(sc.contains("HttpOnly"));
    }

    #[test]
    fn login_post_invalid_returns_401() {
        let (addr, _) = spawn(&user_env("alice", "geheim123"));
        let r = request(
            &addr,
            "POST",
            "/auth/login",
            &[],
            "username=alice&password=falsch&rd=%2F",
        );
        assert_eq!(r.status, 401);
        assert!(r.header("Set-Cookie").is_none());
    }

    #[test]
    fn verify_with_valid_cookie_passes_and_sets_header() {
        let (addr, cfg) = spawn(&user_env("alice", "pw"));
        let cookie = make_cookie(&cfg.secret, "alice", now_unix() + 3600);
        let r = request(
            &addr,
            "GET",
            "/auth/verify",
            &[("Cookie", &format!("{COOKIE_NAME}={cookie}"))],
            "",
        );
        assert_eq!(r.status, 200);
        assert_eq!(r.header("X-Authentik-Username"), Some("alice"));
    }

    #[test]
    fn verify_rejects_cookie_for_removed_user() {
        // Cookie fuer "ghost", aber nur "alice" ist konfiguriert.
        let (addr, cfg) = spawn(&user_env("alice", "pw"));
        let cookie = make_cookie(&cfg.secret, "ghost", now_unix() + 3600);
        let r = request(
            &addr,
            "GET",
            "/auth/verify",
            &[
                ("Cookie", &format!("{COOKIE_NAME}={cookie}")),
                ("Accept", "application/json"),
            ],
            "",
        );
        assert_eq!(r.status, 401);
    }

    #[test]
    fn logout_clears_cookie() {
        let (addr, _) = spawn(&user_env("alice", "pw"));
        let r = request(&addr, "GET", "/auth/logout", &[], "");
        assert_eq!(r.status, 302);
        let sc = r.header("Set-Cookie").unwrap();
        assert!(sc.contains("Max-Age=0"), "sc={sc}");
    }
}
