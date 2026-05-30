use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

// Node-RED-Sync via HA-Supervisor-Ingress (ADR-0009).
//
// Iteration 1: YAML-Snapshot only, keine SQLite-Tabellen.
// Iteration 2 (separates Folge-ADR): SQLite-Tabellen `nodered_flow` +
// `nodered_node`, sobald Cross-Source-Joins gefordert sind.

/// Holt den Flow-Snapshot von Node-RED ueber den HA-Supervisor-Ingress.
///
/// `base_url`  — HA-Basis-URL ohne /api-Suffix, z.B. `https://ha.local:8123`.
/// `ingress_path` — Pfad zwischen Basis und `/flows`. Beispiel:
/// `"api/hassio_ingress/<session>"` oder add-on-spezifischer Reverse-Proxy-
/// Pfad. Bei der Implementierung gegen die laufende Test-HA verifiziert.
/// `token` — HA Long-Lived Access Token (Bearer).
pub fn fetch_flows(base_url: &str, ingress_path: &str, token: &str) -> Result<Value> {
    let base = base_url.trim_end_matches('/');
    let path = ingress_path.trim_start_matches('/').trim_end_matches('/');
    let url = if path.is_empty() {
        format!("{base}/flows")
    } else {
        format!("{base}/{path}/flows")
    };
    let res = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/json")
        .call()
        .with_context(|| format!("GET {url}"))?;
    let body = res.into_string().context("Node-RED-Response-Body lesen")?;
    let flows: Value = serde_json::from_str(&body).context("Node-RED-/flows parsen")?;
    Ok(flows)
}

/// Maskiert rekursiv alle JSON-Object-Felder, deren Schluessel auf bekannte
/// Credential-Felder matcht. Greift Inline-Klartext-Credentials in Config-
/// Nodes ab (CRITICAL-Befund 2026-05-22). Best-Effort: deckt konventionelle
/// Feldnamen; Custom-Schemata mit ungewoehnlichen Namen muessen separat
/// gepflegt werden (Backlog: Allow-List + --audit-Mode).
pub fn sanitize(value: &mut Value) {
    sanitize_inner(value);
}

fn sanitize_inner(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_sensitive_key(k) {
                    mask_in_place(v);
                } else {
                    sanitize_inner(v);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                sanitize_inner(v);
            }
        }
        _ => {}
    }
}

/// Ersetzt String-Werte durch "***masked***". Bei zusammengesetzten Werten
/// (Object/Array) werden ALLE Children rekursiv maskiert — Defense-in-Depth:
/// wenn ein Feld als sensitive markiert ist (z.B. `credentials`), gelten
/// alle seine Inhalte als sensitive, unabhaengig vom Kind-Key. Beispiel:
/// `credentials: { user: "alice", password: "p4ss" }` -> beide Werte
/// werden maskiert, nicht nur das Passwort.
fn mask_in_place(value: &mut Value) {
    match value {
        Value::String(_) => *value = Value::String("***masked***".into()),
        Value::Object(map) => {
            for (_k, v) in map.iter_mut() {
                mask_in_place(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                mask_in_place(v);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    k.contains("password")
        || k.contains("secret")
        || k.contains("token")
        || k.contains("credential")
        || k == "api_key"
        || k == "apikey"
        || k == "api-key"
}

/// Schreibt den Flow-Snapshot deterministisch nach `<dir>/nodered.yaml`.
/// Determinismus: Top-Level-Array wird nach `id` sortiert; gleiche Eingabe
/// erzeugt byte-identische Ausgabe — damit `git status` clean bleibt, wenn
/// sich am Flow nichts geaendert hat.
pub fn write_flows_yaml(dir: impl AsRef<Path>, flows: &Value) -> Result<PathBuf> {
    let dir = dir.as_ref();
    std::fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
    let sorted = sort_by_id(flows);
    let path = dir.join("nodered.yaml");
    let s = serde_yaml_ng::to_string(&sorted).context("serialize nodered.yaml")?;
    std::fs::write(&path, s).with_context(|| format!("schreiben {}", path.display()))?;
    Ok(path)
}

fn sort_by_id(flows: &Value) -> Value {
    let Value::Array(arr) = flows else {
        return flows.clone();
    };
    let mut sorted: Vec<Value> = arr.clone();
    sorted.sort_by(|a, b| {
        let ka = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let kb = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        ka.cmp(kb)
    });
    Value::Array(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;
    use tiny_http::{Header, Response, Server};

    const TOKEN: &str = "secret-llat-token";

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("nodered_flows.json")
    }

    fn fixture_string() -> String {
        std::fs::read_to_string(fixture_path()).expect("fixture nodered_flows.json")
    }

    fn spawn_mock(ingress_path: &'static str) -> String {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        let trimmed = ingress_path.trim_matches('/');
        let expected_path = if trimmed.is_empty() {
            "/flows".to_string()
        } else {
            format!("/{trimmed}/flows")
        };
        let body = fixture_string();
        thread::spawn(move || {
            for req in server.incoming_requests() {
                let auth = req
                    .headers()
                    .iter()
                    .find(|h| {
                        let n: &str = h.field.as_str().as_str();
                        n.eq_ignore_ascii_case("Authorization")
                    })
                    .map(|h| h.value.as_str().to_string())
                    .unwrap_or_default();
                let expected_auth = format!("Bearer {TOKEN}");
                let resp = if req.url() == expected_path && auth == expected_auth {
                    Response::from_string(body.clone())
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        )
                } else if req.url() == expected_path {
                    Response::from_string("forbidden").with_status_code(401)
                } else {
                    Response::from_string("not found").with_status_code(404)
                };
                let _ = req.respond(resp);
            }
        });
        addr
    }

    #[test]
    fn fetch_flows_returns_full_snapshot() {
        let addr = spawn_mock("api/hassio_ingress/abc123");
        let v =
            fetch_flows(&format!("http://{addr}"), "api/hassio_ingress/abc123", TOKEN).unwrap();
        assert!(v.is_array(), "expected top-level array");
        let arr = v.as_array().unwrap();
        // Fixture hat 7 Eintraege: 1 tab + 1 ccu-config + 1 inject + 1 ccu-set +
        // 1 debug + 1 mqtt-broker + 1 ha-server.
        assert_eq!(arr.len(), 7);
    }

    #[test]
    fn fetch_flows_handles_trailing_slash_on_base_url() {
        let addr = spawn_mock("nodered");
        let v = fetch_flows(&format!("http://{addr}/"), "nodered", TOKEN).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 7);
    }

    #[test]
    fn fetch_flows_handles_empty_ingress_path() {
        let addr = spawn_mock("");
        let v = fetch_flows(&format!("http://{addr}"), "", TOKEN).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 7);
    }

    #[test]
    fn fetch_flows_fails_with_wrong_token() {
        let addr = spawn_mock("nodered");
        let err = fetch_flows(&format!("http://{addr}"), "nodered", "wrong").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("GET") && msg.contains("/flows"));
    }

    #[test]
    fn sanitize_masks_password_secret_token_credential_apikey() {
        let raw = fixture_string();
        let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        sanitize(&mut v);
        let s = serde_json::to_string(&v).unwrap();
        // Klartext-Credentials der Fixture duerfen NICHT mehr drinstehen.
        assert!(
            !s.contains("supersecret-do-not-leak"),
            "ccu password leaked: {s}"
        );
        assert!(
            !s.contains("inline-secret-key-abc123"),
            "credential_secret leaked: {s}"
        );
        assert!(
            !s.contains("ha-mqtt-pass-xyz789"),
            "mqtt password leaked: {s}"
        );
        assert!(!s.contains("eyJ0eXAi"), "api_key leaked: {s}");
        assert!(s.contains("***masked***"), "no mask marker found: {s}");
    }

    #[test]
    fn sanitize_preserves_non_sensitive_fields() {
        let raw = fixture_string();
        let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        sanitize(&mut v);
        let s = serde_json::to_string(&v).unwrap();
        // Diese Felder duerfen NICHT maskiert werden.
        assert!(s.contains("Test Flow"));
        assert!(s.contains("Test CCU"));
        assert!(s.contains("10.0.0.20"));
        assert!(s.contains("\"Admin\""), "username Admin verloren");
        assert!(s.contains("BidCos-RF"));
        assert!(s.contains("MEQ0123456:1"));
    }

    #[test]
    fn sanitize_masks_nested_credentials_object() {
        // credentials: { user, password } — der ganze Block ist sensitive
        // (Key "credentials" matched), darin wird rekursiv maskiert.
        let raw = r#"{
            "id": "x",
            "credentials": { "user": "alice", "password": "p4ss" }
        }"#;
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        sanitize(&mut v);
        let s = serde_json::to_string(&v).unwrap();
        assert!(!s.contains("p4ss"));
        assert!(!s.contains("alice"));
        assert!(s.contains("***masked***"));
    }

    #[test]
    fn write_flows_yaml_is_byte_identical_for_same_input() {
        let raw = fixture_string();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let p1 = write_flows_yaml(dir.path(), &v).unwrap();
        let bytes1 = std::fs::read(&p1).unwrap();
        let p2 = write_flows_yaml(dir.path(), &v).unwrap();
        let bytes2 = std::fs::read(&p2).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn write_flows_yaml_sorts_array_by_id() {
        let v: serde_json::Value = serde_json::json!([
            { "id": "z-last", "type": "foo" },
            { "id": "a-first", "type": "bar" },
            { "id": "m-middle", "type": "baz" }
        ]);
        let dir = tempfile::tempdir().unwrap();
        let p = write_flows_yaml(dir.path(), &v).unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        let pos_a = s.find("a-first").expect("a-first im YAML");
        let pos_m = s.find("m-middle").expect("m-middle im YAML");
        let pos_z = s.find("z-last").expect("z-last im YAML");
        assert!(pos_a < pos_m && pos_m < pos_z, "not sorted:\n{s}");
    }

    #[test]
    fn write_flows_yaml_input_order_does_not_change_output() {
        let v_a: serde_json::Value = serde_json::json!([
            { "id": "a", "v": 1 },
            { "id": "b", "v": 2 },
            { "id": "c", "v": 3 }
        ]);
        let v_b: serde_json::Value = serde_json::json!([
            { "id": "c", "v": 3 },
            { "id": "a", "v": 1 },
            { "id": "b", "v": 2 }
        ]);
        let dir = tempfile::tempdir().unwrap();
        let p1 = write_flows_yaml(dir.path().join("a"), &v_a).unwrap();
        let p2 = write_flows_yaml(dir.path().join("b"), &v_b).unwrap();
        assert_eq!(std::fs::read(&p1).unwrap(), std::fs::read(&p2).unwrap());
    }

    #[test]
    fn write_flows_yaml_creates_parent_dir() {
        let v: serde_json::Value = serde_json::json!([]);
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let p = write_flows_yaml(&nested, &v).unwrap();
        assert!(p.exists());
    }

    #[test]
    fn is_sensitive_key_matches_common_credentials() {
        assert!(is_sensitive_key("password"));
        assert!(is_sensitive_key("PASSWORD"));
        assert!(is_sensitive_key("credential_secret"));
        assert!(is_sensitive_key("api_key"));
        assert!(is_sensitive_key("apiKey"));
        assert!(is_sensitive_key("client_secret"));
        assert!(is_sensitive_key("oauth_token"));
        assert!(!is_sensitive_key("username"));
        assert!(!is_sensitive_key("host"));
        assert!(!is_sensitive_key("port"));
        assert!(!is_sensitive_key("name"));
        assert!(!is_sensitive_key("type"));
    }

    #[test]
    fn fetch_then_sanitize_then_write_is_idempotent_end_to_end() {
        // End-to-end: GET → sanitize → write zweimal. Bytes muessen gleich sein.
        let addr = spawn_mock("api/hassio_ingress/abc");
        let dir = tempfile::tempdir().unwrap();

        let mut v1 =
            fetch_flows(&format!("http://{addr}"), "api/hassio_ingress/abc", TOKEN).unwrap();
        sanitize(&mut v1);
        let p1 = write_flows_yaml(dir.path(), &v1).unwrap();
        let bytes1 = std::fs::read(&p1).unwrap();

        let mut v2 =
            fetch_flows(&format!("http://{addr}"), "api/hassio_ingress/abc", TOKEN).unwrap();
        sanitize(&mut v2);
        let p2 = write_flows_yaml(dir.path(), &v2).unwrap();
        let bytes2 = std::fs::read(&p2).unwrap();

        assert_eq!(bytes1, bytes2, "two runs differ — sync would dirty git");
    }
}
