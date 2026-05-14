use tiny_http::Request;

/// Auth-Konfiguration. Wird beim Start aus Env gelesen.
pub struct Config {
    pub header_name: String,
    pub bypass: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let header_name = std::env::var("AUTH_HEADER")
            .unwrap_or_else(|_| "X-Authentik-Username".to_string());
        let bypass = matches!(
            std::env::var("AUTH_BYPASS").as_deref(),
            Ok("1") | Ok("true") | Ok("yes")
        );
        Self { header_name, bypass }
    }
}

/// Liefert den authentifizierten User oder None. Header werden case-insensitive
/// verglichen (tiny_http kanonisiert nicht, deshalb explizit ueber equiv).
pub fn extract_user(req: &Request, cfg: &Config) -> Option<String> {
    if cfg.bypass {
        return Some("dev".to_string());
    }
    let want = cfg.header_name.as_str();
    req.headers().iter().find_map(|h| {
        let got: &str = h.field.as_str().as_str();
        if got.eq_ignore_ascii_case(want) {
            Some(h.value.as_str().to_string())
        } else {
            None
        }
    })
}

// Hinweis: from_env-Tests waeren wegen std::env-Race-Conditions zwischen
// parallel laufenden cargo-Tests unzuverlaessig. Die Default-Werte sind
// als Konstanten im Code lesbar. Verhaltenstest passiert in http::tests.
