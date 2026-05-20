use crate::types::Device;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

/// Roh-Repraesentation eines CCU-Geraets aus `devicelist.cgi`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CcuDevice {
    pub address: String,
    pub name: String,
    pub device_type: String,
    pub interface: String,
    pub firmware: String,
    pub updatable: bool,
}

/// Baut den Basic-Auth-Header-Wert "Basic <base64(user:password)>".
/// Eigene Helper, damit der Test den Header-Wert unabhaengig vom Netzwerk
/// pruefen kann.
fn basic_auth_header(user: &str, password: &str) -> String {
    let credentials = B64.encode(format!("{user}:{password}"));
    format!("Basic {credentials}")
}

/// Holt die Geraeteliste der CCU.
///
/// `base_url` ohne Pfad, z.B. `http://ccu.example.local`. Pfad wird
/// angehaengt.
///
/// `user` / `password` sind optional. Sind beide gesetzt, wird ein
/// `Authorization: Basic …`-Header gesendet (RaspberryMatic 3.65+
/// Default mit aktivierter Authentisierung). Sind beide leer, geht
/// der Aufruf ohne Auth-Header raus (Status quo fuer CCUs ohne Auth).
/// Ist nur eins gesetzt, ist das ein klarer Konfigurationsfehler und
/// die Funktion bricht mit aussagekraeftiger Fehlermeldung ab.
pub fn fetch_devicelist(
    base_url: &str,
    user: Option<&str>,
    password: Option<&str>,
) -> Result<Vec<CcuDevice>> {
    // Halb-konfigurierte Auth ist ein User-Fehler: explizit ablehnen, nicht
    // stillschweigend Auth abschalten.
    match (user, password) {
        (Some(u), None) if !u.is_empty() => {
            anyhow::bail!(
                "CCU auth misconfigured: CCU_USER is set but CCU_PASSWORD is empty. \
                 Set both CCU_USER and CCU_PASSWORD together (or unset both for \
                 CCUs without authentication)."
            );
        }
        (None, Some(p)) if !p.is_empty() => {
            anyhow::bail!(
                "CCU auth misconfigured: CCU_PASSWORD is set but CCU_USER is empty. \
                 Set both CCU_USER and CCU_PASSWORD together (or unset both for \
                 CCUs without authentication)."
            );
        }
        _ => {}
    }

    let url = format!(
        "{}/addons/xmlapi/devicelist.cgi",
        base_url.trim_end_matches('/')
    );
    let mut req = ureq::get(&url);
    if let (Some(u), Some(p)) = (user, password) {
        if !u.is_empty() && !p.is_empty() {
            req = req.set("Authorization", &basic_auth_header(u, p));
        }
    }
    let res = req.call().with_context(|| format!("GET {url}"))?;
    let body = res.into_string().context("CCU-Response-Body lesen")?;

    // RaspberryMatic XML-API antwortet HTTP 200 mit Body
    // `<not_authenticated/>`, wenn Auth aktiv ist und kein/falscher
    // Auth-Header anliegt. Klare Diagnose statt nichtssagendem
    // XML-Parse-Error.
    if body.trim().contains("<not_authenticated/>") {
        anyhow::bail!(
            "CCU XML-API returned <not_authenticated/>. \
             Set CCU_USER + CCU_PASSWORD env vars (preferred over CLI args), \
             or disable CCU authentication for the XML-API addon."
        );
    }

    parse_devicelist(&body)
}

/// Parsed eine XML-Geraeteliste. Kanaele werden uebersprungen — wir wollen
/// nur die physischen Geraete (jeweils das aeussere <device>-Element).
pub fn parse_devicelist(xml: &str) -> Result<Vec<CcuDevice>> {
    let doc = roxmltree::Document::parse(xml).context("XML parsen")?;
    let root = doc.root_element();
    if root.tag_name().name() != "deviceList" {
        return Err(anyhow!(
            "unerwartetes Root-Element: {}",
            root.tag_name().name()
        ));
    }
    let mut out = Vec::new();
    for d in root.children().filter(|n| n.is_element() && n.tag_name().name() == "device") {
        let address = d.attribute("address").unwrap_or_default().to_string();
        let name = d.attribute("name").unwrap_or_default().to_string();
        let device_type = d.attribute("device_type").unwrap_or_default().to_string();
        let interface = d.attribute("interface").unwrap_or_default().to_string();
        let firmware = d.attribute("firmware").unwrap_or_default().to_string();
        let updatable = matches!(d.attribute("updatable"), Some("1"));
        if address.is_empty() {
            continue;
        }
        out.push(CcuDevice {
            address,
            name,
            device_type,
            interface,
            firmware,
            updatable,
        });
    }
    Ok(out)
}

/// Filtert virtuelle Empfaengerknoten (z.B. HmIP-RCV-50, BidCos-RCV-50) und
/// mapped die echten Geraete auf das kanonische Device.
pub fn map_to_devices(devices: &[CcuDevice]) -> Vec<Device> {
    devices.iter().filter(|d| !is_virtual(d)).map(to_device).collect()
}

fn is_virtual(d: &CcuDevice) -> bool {
    d.device_type.contains("-RCV-")
}

fn to_device(d: &CcuDevice) -> Device {
    Device {
        source: "ccu".into(),
        source_id: d.address.clone(),
        name: if d.name.is_empty() {
            d.address.clone()
        } else {
            d.name.clone()
        },
        manufacturer: Some("eQ-3".into()),
        model: Some(d.device_type.clone()),
        kind: Some(kind_of(&d.device_type).to_string()),
        room: None,
    }
}

/// Sehr grobes Kind-Mapping aus dem Geraete-Typ-Praefix.
fn kind_of(device_type: &str) -> &'static str {
    let dt = device_type.to_ascii_uppercase();
    if dt.contains("WTH") || dt.contains("CC") {
        "thermostat"
    } else if dt.contains("LC-SW") || dt.contains("-SW") || dt.contains("-PS") {
        "switch"
    } else if dt.contains("LC-BL") || dt.contains("FROLL") || dt.contains("BROLL") {
        "cover"
    } else if dt.contains("LC-DIM") {
        "dimmer"
    } else if dt.contains("-PB") || dt.contains("-RC-") {
        "button"
    } else if dt.contains("SEC") || dt.contains("MOT") {
        "sensor"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> String {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("ccu_devicelist.xml");
        std::fs::read_to_string(p).unwrap()
    }

    #[test]
    fn parse_returns_four_raw_devices() {
        let xml = fixture();
        let devs = parse_devicelist(&xml).unwrap();
        assert_eq!(devs.len(), 4);
        assert_eq!(devs[0].address, "0001AAAAAAAAA1");
        assert_eq!(devs[0].firmware, "2.6.4");
        assert!(devs[2].updatable);
    }

    #[test]
    fn parse_rejects_wrong_root() {
        let err = parse_devicelist("<wrongRoot/>").unwrap_err();
        assert!(format!("{err:#}").contains("unerwartetes Root-Element"));
    }

    #[test]
    fn map_filters_virtual_receivers() {
        let xml = fixture();
        let devs = parse_devicelist(&xml).unwrap();
        let mapped = map_to_devices(&devs);
        assert_eq!(mapped.len(), 3); // HmIP-RCV-50 raus
        assert!(mapped.iter().all(|d| d.source == "ccu"));
        let ids: Vec<&str> = mapped.iter().map(|d| d.source_id.as_str()).collect();
        assert!(ids.contains(&"0001AAAAAAAAA1"));
        assert!(ids.contains(&"MEQ0000001"));
        assert!(ids.contains(&"OEQ0000001"));
    }

    #[test]
    fn map_assigns_kind_and_manufacturer() {
        let xml = fixture();
        let mapped = map_to_devices(&parse_devicelist(&xml).unwrap());
        let thermo = mapped.iter().find(|d| d.source_id == "0001AAAAAAAAA1").unwrap();
        assert_eq!(thermo.kind.as_deref(), Some("thermostat"));
        assert_eq!(thermo.manufacturer.as_deref(), Some("eQ-3"));
        assert_eq!(thermo.model.as_deref(), Some("HmIP-WTH-2"));

        let switch_dev = mapped.iter().find(|d| d.source_id == "MEQ0000001").unwrap();
        assert_eq!(switch_dev.kind.as_deref(), Some("switch"));
    }
}
