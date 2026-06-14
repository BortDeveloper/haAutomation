use crate::{auth, db, views};
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use tiny_http::{Header, Method, Request, Response, Server};

pub fn bind(addr: &str) -> Result<Server> {
    Server::http(addr).map_err(|e| anyhow!("bind {}: {}", addr, e))
}

pub fn serve(server: Server, conn: Connection, auth_cfg: auth::Config) -> Result<()> {
    for req in server.incoming_requests() {
        if let Err(e) = handle(req, &conn, &auth_cfg) {
            eprintln!("request handler error: {e:#}");
        }
    }
    Ok(())
}

fn handle(req: Request, conn: &Connection, auth_cfg: &auth::Config) -> Result<()> {
    let method = req.method().clone();
    let url = req.url().to_string();
    let user = auth::extract_user(&req, auth_cfg);

    let response = match (&method, url.as_str(), user.as_deref()) {
        // Public Endpoints (Healthchecks brauchen keine Auth)
        (&Method::Get, "/health", _) => text("ok", 200),

        // Alles andere: kein User -> 401
        (_, _, None) => text("unauthorized", 401),

        // Authentifizierte Routen
        (&Method::Get, "/api/devices", Some(_)) => {
            let devices = db::list_devices(conn)?;
            json(serde_json::to_string(&devices)?, 200)
        }
        (&Method::Get, "/", Some(_)) => {
            let devices = db::list_devices(conn)?;
            html(views::devices_page(&devices), 200)
        }
        _ => text("not found", 404),
    };
    req.respond(response)?;
    Ok(())
}

type Resp = Response<std::io::Cursor<Vec<u8>>>;

fn text(body: &str, status: u16) -> Resp {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type("text/plain; charset=utf-8"))
}

fn json(body: String, status: u16) -> Resp {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type("application/json"))
}

fn html(body: String, status: u16) -> Resp {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type("text/html; charset=utf-8"))
}

fn content_type(value: &str) -> Header {
    Header::from_bytes("Content-Type", value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml_io;
    use std::path::PathBuf;
    use std::thread;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    /// Header-Name, der in Tests verwendet wird, damit nichts mit dem Default
    /// "X-Authentik-Username" kollidiert.
    const TEST_HEADER: &str = "X-Test-User";

    fn strict() -> auth::Config {
        auth::Config {
            header_name: TEST_HEADER.into(),
            bypass: false,
        }
    }

    fn bypass() -> auth::Config {
        auth::Config {
            header_name: TEST_HEADER.into(),
            bypass: true,
        }
    }

    fn spawn(with_fixture: bool, cfg: auth::Config) -> String {
        let server = bind("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        if with_fixture {
            let devices = yaml_io::load_devices(fixture("devices.yaml")).unwrap();
            db::upsert_devices(&conn, &devices).unwrap();
        }
        thread::spawn(move || {
            let _ = serve(server, conn, cfg);
        });
        addr
    }

    // --- S4-Tests (unter Bypass, da Auth jetzt aktiv ist) ---

    #[test]
    fn health_returns_ok_under_bypass() {
        let addr = spawn(false, bypass());
        let res = ureq::get(&format!("http://{addr}/health")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.into_string().unwrap(), "ok");
    }

    #[test]
    fn unknown_route_returns_404_when_authenticated() {
        let addr = spawn(false, bypass());
        let err = ureq::get(&format!("http://{addr}/does-not-exist"))
            .call()
            .unwrap_err();
        match err {
            ureq::Error::Status(404, _) => {}
            other => panic!("expected 404, got {other:?}"),
        }
    }

    // --- S5-Tests (unter Bypass) ---

    #[test]
    fn api_devices_returns_json_array_of_3() {
        let addr = spawn(true, bypass());
        let res = ureq::get(&format!("http://{addr}/api/devices")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.header("Content-Type"), Some("application/json"));
        let body = res.into_string().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn root_returns_html_with_devices() {
        let addr = spawn(true, bypass());
        let res = ureq::get(&format!("http://{addr}/")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.header("Content-Type").unwrap().contains("text/html"));
        let body = res.into_string().unwrap();
        assert!(body.contains("<table>"));
        assert!(body.contains("Room A Ceiling"));
        assert!(body.contains("3 Geraete"));
    }

    // --- S7-Tests (Auth-Verhalten) ---

    #[test]
    fn root_without_header_returns_401() {
        let addr = spawn(false, strict());
        let err = ureq::get(&format!("http://{addr}/")).call().unwrap_err();
        match err {
            ureq::Error::Status(401, _) => {}
            other => panic!("expected 401, got {other:?}"),
        }
    }

    #[test]
    fn api_with_header_returns_200() {
        let addr = spawn(true, strict());
        let res = ureq::get(&format!("http://{addr}/api/devices"))
            .set(TEST_HEADER, "alice")
            .call()
            .unwrap();
        assert_eq!(res.status(), 200);
    }

    #[test]
    fn api_with_bypass_returns_200_without_header() {
        let addr = spawn(true, bypass());
        let res = ureq::get(&format!("http://{addr}/api/devices"))
            .call()
            .unwrap();
        assert_eq!(res.status(), 200);
    }

    #[test]
    fn health_is_public_even_under_strict_auth() {
        let addr = spawn(false, strict());
        let res = ureq::get(&format!("http://{addr}/health")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.into_string().unwrap(), "ok");
    }
}
