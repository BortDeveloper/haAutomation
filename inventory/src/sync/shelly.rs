use crate::types::Device;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;

/// Normalisierte Repraesentation eines Shelly-Geraets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellyDevice {
    pub mac: String,         // ohne Trenner, lowercase
    pub ip: String,
    pub id: String,          // shellyXxx-mac fuer Gen2, sonst "<type>-<mac>"
    pub model: String,       // type (Gen1) oder model (Gen2)
    pub app: Option<String>, // Gen2-spezifisch ("Plus2PM" etc.)
    pub firmware: String,    // ver (Gen2) oder fw (Gen1)
    pub gen: u8,
}

#[derive(Debug, Deserialize)]
struct Gen1Info {
    #[serde(default)]
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    mac: String,
    #[serde(default)]
    fw: String,
}

#[derive(Debug, Deserialize)]
struct Gen2Info {
    #[serde(default)]
    id: String,
    #[serde(default)]
    mac: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    ver: String,
    #[serde(default)]
    app: Option<String>,
}

/// Holt Info zu einem Shelly. Probiert zuerst Gen2 (RPC), faellt auf Gen1
/// zurueck wenn der Endpoint nicht existiert.
pub fn fetch_info(ip: &str) -> Result<ShellyDevice> {
    if let Ok(d) = fetch_gen2(ip) {
        return Ok(d);
    }
    fetch_gen1(ip)
}

fn fetch_gen2(ip: &str) -> Result<ShellyDevice> {
    let url = format!("http://{}/rpc/Shelly.GetDeviceInfo", ip);
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .into_string()
        .context("gen2 body")?;
    let info: Gen2Info = serde_json::from_str(&body).context("gen2 parsen")?;
    if info.mac.is_empty() {
        anyhow::bail!("gen2: leere mac");
    }
    Ok(ShellyDevice {
        mac: info.mac.to_ascii_lowercase(),
        ip: ip.to_string(),
        id: info.id,
        model: info.model,
        app: info.app,
        firmware: info.ver,
        gen: 2,
    })
}

fn fetch_gen1(ip: &str) -> Result<ShellyDevice> {
    let url = format!("http://{}/shelly", ip);
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .into_string()
        .context("gen1 body")?;
    let info: Gen1Info = serde_json::from_str(&body).context("gen1 parsen")?;
    if info.mac.is_empty() {
        anyhow::bail!("gen1: leere mac");
    }
    let mac = info.mac.to_ascii_lowercase();
    Ok(ShellyDevice {
        id: format!("{}-{}", info.type_.to_ascii_lowercase(), mac),
        mac,
        ip: ip.to_string(),
        model: info.type_,
        app: None,
        firmware: info.fw,
        gen: 1,
    })
}

/// mDNS-Scan ueber timeout Sekunden. Liefert IPv4-Adressen von Hosts,
/// deren Hostname mit "shelly" beginnt — das Namensschema aller aktuellen
/// Shellys, sowohl Gen1 als auch Gen2.
pub fn discover(timeout: Duration) -> Result<Vec<String>> {
    let mdns = mdns_sd::ServiceDaemon::new().context("mdns daemon")?;
    let receiver = mdns.browse("_http._tcp.local.").context("mdns browse")?;
    let mut ips: HashSet<String> = HashSet::new();
    let deadline = std::time::Instant::now() + timeout;
    while let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
        match receiver.recv_timeout(remaining) {
            Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                let host = info.get_hostname().to_ascii_lowercase();
                if !host.starts_with("shelly") {
                    continue;
                }
                for addr in info.get_addresses_v4() {
                    ips.insert(addr.to_string());
                }
            }
            Ok(_other) => {}
            Err(_) => break, // Timeout
        }
    }
    let _ = mdns.shutdown();
    Ok(ips.into_iter().collect())
}

pub fn map_to_devices(devices: &[ShellyDevice]) -> Vec<Device> {
    devices.iter().map(to_device).collect()
}

fn to_device(d: &ShellyDevice) -> Device {
    Device {
        source: "shelly".into(),
        source_id: d.mac.clone(),
        name: if d.id.is_empty() {
            d.mac.clone()
        } else {
            d.id.clone()
        },
        manufacturer: Some("Allterco Robotics".into()),
        model: Some(d.model.clone()),
        kind: Some(kind_of(d).into()),
        room: None,
    }
}

fn kind_of(d: &ShellyDevice) -> &'static str {
    let model = d.model.to_ascii_lowercase();
    let app = d.app.as_deref().unwrap_or("").to_ascii_lowercase();
    if model.contains("dimmer") || app.contains("dimmer") {
        "dimmer"
    } else if model.contains("rgbw") || app.contains("rgb") {
        "light"
    } else if model.contains("plug") || app.contains("plug") {
        "switch"
    } else if model.contains("roller") || app.contains("roller")
        || model.contains("2pm") || app.contains("2pm")
    {
        "cover"
    } else if model.starts_with("shht") || model.starts_with("snsn") || app.contains("h&t") {
        "sensor"
    } else if model.contains("sw") || app.contains("sw") || model.starts_with("shsw") {
        "switch"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tiny_http::{Header, Response, Server};

    const GEN2_BODY: &str = r#"{
        "name": null, "id": "shellyplus2pm-08b61fd83fdc", "mac": "08B61FD83FDC",
        "model": "SNSW-002P16EU", "gen": 2, "fw_id": "20231012-112414/1.0.7",
        "ver": "1.0.7", "app": "Plus2PM"
    }"#;

    const GEN1_BODY: &str = r#"{
        "type": "SHSW-25", "mac": "C45BBE60FDE9", "auth": false,
        "fw": "20230913-114008/v1.14.0", "longid": 1, "num_outputs": 2
    }"#;

    /// Mock-Server, der wahlweise auf gen2- oder gen1-URLs antwortet.
    fn spawn_mock(mode: &'static str) -> String {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        thread::spawn(move || {
            for req in server.incoming_requests() {
                let url = req.url().to_string();
                let resp = match (mode, url.as_str()) {
                    ("gen2", "/rpc/Shelly.GetDeviceInfo") => Response::from_string(GEN2_BODY)
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        ),
                    ("gen1", "/shelly") => Response::from_string(GEN1_BODY)
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        ),
                    // Gen1-only: /rpc gibt 404 -> auto-Fallback im Client
                    ("gen1", "/rpc/Shelly.GetDeviceInfo") => {
                        Response::from_string("not found").with_status_code(404)
                    }
                    _ => Response::from_string("nope").with_status_code(404),
                };
                let _ = req.respond(resp);
            }
        });
        addr
    }

    #[test]
    fn fetch_gen2_parses_firmware_and_mac() {
        let addr = spawn_mock("gen2");
        let d = fetch_info(&addr).unwrap();
        assert_eq!(d.gen, 2);
        assert_eq!(d.mac, "08b61fd83fdc");
        assert_eq!(d.firmware, "1.0.7");
        assert_eq!(d.model, "SNSW-002P16EU");
        assert_eq!(d.app.as_deref(), Some("Plus2PM"));
    }

    #[test]
    fn fetch_falls_back_to_gen1_when_rpc_404() {
        let addr = spawn_mock("gen1");
        let d = fetch_info(&addr).unwrap();
        assert_eq!(d.gen, 1);
        assert_eq!(d.mac, "c45bbe60fde9");
        assert_eq!(d.firmware, "20230913-114008/v1.14.0");
        assert_eq!(d.model, "SHSW-25");
    }

    #[test]
    fn map_to_devices_assigns_manufacturer_and_kind() {
        let devs = vec![
            ShellyDevice {
                mac: "aaa".into(),
                ip: "1.1.1.1".into(),
                id: "shellyplus2pm-aaa".into(),
                model: "SNSW-002P16EU".into(),
                app: Some("Plus2PM".into()),
                firmware: "1.0.7".into(),
                gen: 2,
            },
            ShellyDevice {
                mac: "bbb".into(),
                ip: "1.1.1.2".into(),
                id: "shsw-25-bbb".into(),
                model: "SHSW-25".into(),
                app: None,
                firmware: "v1.14.0".into(),
                gen: 1,
            },
        ];
        let mapped = map_to_devices(&devs);
        assert_eq!(mapped.len(), 2);
        assert!(mapped.iter().all(|d| d.source == "shelly"));
        assert!(mapped
            .iter()
            .all(|d| d.manufacturer.as_deref() == Some("Allterco Robotics")));
        // 2PM mapped als cover (Rolladen)
        assert_eq!(
            mapped.iter().find(|d| d.source_id == "aaa").unwrap().kind.as_deref(),
            Some("cover")
        );
        // SHSW-25 hat "sw" und "shsw" Muster -> switch
        assert_eq!(
            mapped.iter().find(|d| d.source_id == "bbb").unwrap().kind.as_deref(),
            Some("switch")
        );
    }

    #[test]
    fn fetch_handles_unknown_host_gracefully() {
        // Port 1 ist nicht offen -> ureq schlaegt fehl -> Fehler durchgereicht
        let err = fetch_info("127.0.0.1:1").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("GET"));
    }
}
