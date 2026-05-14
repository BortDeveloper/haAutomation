use crate::{db, views};
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use tiny_http::{Header, Method, Request, Response, Server};

pub fn bind(addr: &str) -> Result<Server> {
    Server::http(addr).map_err(|e| anyhow!("bind {}: {}", addr, e))
}

/// Blockierende Annahme-Schleife. tiny_http verarbeitet Requests sequentiell aus
/// dem internen Queue, deshalb genuegt eine einzige Connection ohne Pool/Mutex.
pub fn serve(server: Server, conn: Connection) -> Result<()> {
    for req in server.incoming_requests() {
        if let Err(e) = handle(req, &conn) {
            eprintln!("request handler error: {e:#}");
        }
    }
    Ok(())
}

fn handle(req: Request, conn: &Connection) -> Result<()> {
    let response = match (req.method(), req.url()) {
        (Method::Get, "/health") => text("ok", 200),
        (Method::Get, "/api/devices") => {
            let devices = db::list_devices(conn)?;
            json(serde_json::to_string(&devices)?, 200)
        }
        (Method::Get, "/") => {
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

    fn spawn(with_fixture: bool) -> String {
        let server = bind("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        if with_fixture {
            let devices = yaml_io::load_devices(fixture("devices.yaml")).unwrap();
            db::upsert_devices(&conn, &devices).unwrap();
        }
        thread::spawn(move || {
            let _ = serve(server, conn);
        });
        addr
    }

    #[test]
    fn health_returns_ok() {
        let addr = spawn(false);
        let res = ureq::get(&format!("http://{addr}/health")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.into_string().unwrap(), "ok");
    }

    #[test]
    fn unknown_route_returns_404() {
        let addr = spawn(false);
        let err = ureq::get(&format!("http://{addr}/does-not-exist"))
            .call()
            .unwrap_err();
        match err {
            ureq::Error::Status(404, _) => {}
            other => panic!("expected 404, got {other:?}"),
        }
    }

    #[test]
    fn api_devices_returns_json_array_of_3() {
        let addr = spawn(true);
        let res = ureq::get(&format!("http://{addr}/api/devices")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.header("Content-Type"), Some("application/json"));
        let body = res.into_string().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn root_returns_html_with_devices() {
        let addr = spawn(true);
        let res = ureq::get(&format!("http://{addr}/")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.header("Content-Type").unwrap().contains("text/html"));
        let body = res.into_string().unwrap();
        assert!(body.contains("<table>"));
        assert!(body.contains("Kueche Decke")); // aus der Fixture
        assert!(body.contains("3 Geraete"));
    }
}
