use anyhow::{anyhow, Result};
use tiny_http::{Method, Request, Response, Server};

/// Bindet einen Server an addr (z.B. "0.0.0.0:8080" oder "127.0.0.1:0").
pub fn bind(addr: &str) -> Result<Server> {
    Server::http(addr).map_err(|e| anyhow!("bind {}: {}", addr, e))
}

/// Blockierende Annahme-Schleife. Jeder Request geht durch `handle`.
pub fn serve(server: Server) -> Result<()> {
    for req in server.incoming_requests() {
        if let Err(e) = handle(req) {
            eprintln!("request handler error: {e:#}");
        }
    }
    Ok(())
}

fn handle(req: Request) -> Result<()> {
    let response = match (req.method(), req.url()) {
        (Method::Get, "/health") => Response::from_string("ok").with_status_code(200),
        _ => Response::from_string("not found").with_status_code(404),
    };
    req.respond(response)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn spawn_server() -> String {
        let server = bind("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        thread::spawn(move || {
            let _ = serve(server);
        });
        addr
    }

    #[test]
    fn health_returns_ok() {
        let addr = spawn_server();
        let res = ureq::get(&format!("http://{addr}/health")).call().unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.into_string().unwrap(), "ok");
    }

    #[test]
    fn unknown_route_returns_404() {
        let addr = spawn_server();
        let err = ureq::get(&format!("http://{addr}/does-not-exist"))
            .call()
            .unwrap_err();
        match err {
            ureq::Error::Status(404, _) => {}
            other => panic!("expected 404, got {other:?}"),
        }
    }
}
