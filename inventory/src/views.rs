use crate::types::Device;

/// Komplette HTML-Seite mit Geraete-Tabelle. Inline-Styles, kein Template-Engine.
pub fn devices_page(devices: &[Device]) -> String {
    let rows: String = devices.iter().map(row).collect();
    format!(
        "<!doctype html>\n\
         <html lang=\"de\"><head><meta charset=\"utf-8\">\n\
         <title>haBortfeld inventory</title>\n\
         <style>\n\
         body{{font-family:system-ui,sans-serif;margin:2rem;color:#222;}}\n\
         h1{{margin:0 0 .25rem;}}\n\
         .count{{color:#888;font-size:.9rem;margin-bottom:1rem;}}\n\
         table{{border-collapse:collapse;width:100%;}}\n\
         th,td{{padding:.4rem .6rem;border-bottom:1px solid #ddd;text-align:left;font-size:.9rem;}}\n\
         th{{background:#f4f4f4;font-weight:600;}}\n\
         tbody tr:hover{{background:#fafafa;}}\n\
         .src{{font-family:monospace;color:#666;}}\n\
         </style></head><body>\n\
         <h1>Inventar</h1>\n\
         <div class=\"count\">{} Geraete</div>\n\
         <table>\n\
         <thead><tr><th>Source</th><th>ID</th><th>Name</th><th>Hersteller</th><th>Modell</th><th>Art</th><th>Raum</th></tr></thead>\n\
         <tbody>\n{}</tbody>\n\
         </table>\n\
         </body></html>",
        devices.len(),
        rows,
    )
}

fn row(d: &Device) -> String {
    format!(
        "<tr><td class=\"src\">{}</td><td class=\"src\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
        esc(&d.source),
        esc(&d.source_id),
        esc(&d.name),
        esc(d.manufacturer.as_deref().unwrap_or("")),
        esc(d.model.as_deref().unwrap_or("")),
        esc(d.kind.as_deref().unwrap_or("")),
        esc(d.room.as_deref().unwrap_or("")),
    )
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(source: &str, id: &str, name: &str) -> Device {
        Device {
            source: source.into(),
            source_id: id.into(),
            name: name.into(),
            manufacturer: None,
            model: None,
            kind: None,
            room: None,
        }
    }

    #[test]
    fn html_contains_table_and_count() {
        let html = devices_page(&[d("ha", "x.y", "Foo")]);
        assert!(html.contains("<table>"));
        assert!(html.contains("Foo"));
        assert!(html.contains("1 Geraete"));
    }

    #[test]
    fn html_escapes_dangerous_chars() {
        let html = devices_page(&[d("ha", "x", "<script>alert(1)</script>")]);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
