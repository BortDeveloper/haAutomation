use crate::types::Device;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Eine Bridge-Konfig — pro Eintrag IP + API-Key.
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeConfig {
    pub ip: String,
    pub token: String,
    #[serde(default)]
    pub name: Option<String>, // optional, fuer Logging
}

/// Interne Repraesentation eines Hue-Lights oder -Sensors.
#[derive(Debug, Clone)]
pub struct HueDevice {
    pub uniqueid: String,
    pub name: String,
    pub manufacturername: Option<String>,
    pub productname: Option<String>,
    pub modelid: Option<String>,
    pub swversion: Option<String>,
    pub kind: HueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HueKind {
    Light,
    Sensor,
}

// === Rohformat aus der Hue-API v1 ===
#[derive(Debug, Deserialize)]
struct HueLight {
    #[serde(default)]
    name: String,
    #[serde(default)]
    modelid: Option<String>,
    #[serde(default)]
    productname: Option<String>,
    #[serde(default)]
    manufacturername: Option<String>,
    #[serde(default)]
    swversion: Option<String>,
    #[serde(default)]
    uniqueid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HueSensor {
    #[serde(default)]
    name: String,
    #[serde(default)]
    modelid: Option<String>,
    #[serde(default)]
    productname: Option<String>,
    #[serde(default)]
    manufacturername: Option<String>,
    #[serde(default)]
    swversion: Option<String>,
    #[serde(default)]
    uniqueid: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    type_: String,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Vec<BridgeConfig>> {
    let path = path.as_ref();
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("hue config lesen: {}", path.display()))?;
    let cfg: Vec<BridgeConfig> = serde_yaml_ng::from_str(&s)
        .with_context(|| format!("hue config parsen: {}", path.display()))?;
    Ok(cfg)
}

pub fn fetch_lights(ip: &str, token: &str) -> Result<Vec<HueDevice>> {
    let url = format!("http://{}/api/{}/lights", ip, token);
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .into_string()
        .context("hue-lights body")?;
    let raw: BTreeMap<String, HueLight> =
        serde_json::from_str(&body).context("hue-lights parsen")?;
    Ok(raw
        .into_iter()
        .filter_map(|(_id, l)| {
            l.uniqueid.clone().map(|uid| HueDevice {
                uniqueid: uid,
                name: l.name,
                manufacturername: l.manufacturername,
                productname: l.productname,
                modelid: l.modelid,
                swversion: l.swversion,
                kind: HueKind::Light,
            })
        })
        .collect())
}

pub fn fetch_sensors(ip: &str, token: &str) -> Result<Vec<HueDevice>> {
    let url = format!("http://{}/api/{}/sensors", ip, token);
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .into_string()
        .context("hue-sensors body")?;
    let raw: BTreeMap<String, HueSensor> =
        serde_json::from_str(&body).context("hue-sensors parsen")?;
    Ok(raw
        .into_iter()
        // Hue meldet auch interne pseudo-Sensoren wie CLIPGenericStatus —
        // die haben keine uniqueid und werden hier rausgefiltert.
        .filter_map(|(_id, s)| {
            s.uniqueid.clone().map(|uid| HueDevice {
                uniqueid: uid,
                name: s.name,
                manufacturername: s.manufacturername,
                productname: s.productname.or(Some(s.type_)),
                modelid: s.modelid,
                swversion: s.swversion,
                kind: HueKind::Sensor,
            })
        })
        .collect())
}

pub fn map_to_devices(devices: &[HueDevice]) -> Vec<Device> {
    devices.iter().map(to_device).collect()
}

fn to_device(d: &HueDevice) -> Device {
    Device {
        source: "hue".into(),
        source_id: d.uniqueid.clone(),
        name: if d.name.is_empty() {
            d.uniqueid.clone()
        } else {
            d.name.clone()
        },
        manufacturer: d.manufacturername.clone(),
        model: d.productname.clone().or_else(|| d.modelid.clone()),
        kind: Some(
            match d.kind {
                HueKind::Light => "light",
                HueKind::Sensor => "sensor",
            }
            .into(),
        ),
        room: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tiny_http::{Header, Response, Server};

    const LIGHTS_JSON: &str = r#"{
        "1": {"name": "Room B Ceiling", "modelid": "LCT016", "productname": "Hue color lamp",
              "manufacturername": "Signify Netherlands B.V.", "swversion": "1.122.2",
              "uniqueid": "00:17:88:01:08:aa:bb:cc-0b"},
        "2": {"name": "Dining Table", "modelid": "LCT015", "productname": "Hue color lamp",
              "manufacturername": "Signify Netherlands B.V.", "swversion": "1.122.2",
              "uniqueid": "00:17:88:01:08:dd:ee:ff-0b"}
    }"#;

    const SENSORS_JSON: &str = r#"{
        "10": {"name": "Motion Hallway", "modelid": "SML001", "productname": "Hue motion sensor",
               "manufacturername": "Signify Netherlands B.V.", "swversion": "6.1.1.27575",
               "uniqueid": "00:17:88:01:09:11:22:33-02-0406", "type": "ZLLPresence"},
        "20": {"name": "Daylight", "type": "Daylight"}
    }"#;

    const TOKEN: &str = "test-token";

    fn spawn_mock() -> String {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
        thread::spawn(move || {
            for req in server.incoming_requests() {
                let url = req.url().to_string();
                let resp = if url == format!("/api/{TOKEN}/lights") {
                    Response::from_string(LIGHTS_JSON)
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        )
                } else if url == format!("/api/{TOKEN}/sensors") {
                    Response::from_string(SENSORS_JSON)
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        )
                } else {
                    Response::from_string("nope").with_status_code(404)
                };
                let _ = req.respond(resp);
            }
        });
        addr
    }

    #[test]
    fn fetch_lights_returns_two() {
        let addr = spawn_mock();
        let devs = fetch_lights(&addr, TOKEN).unwrap();
        assert_eq!(devs.len(), 2);
        let first = devs.iter().find(|d| d.name == "Room B Ceiling").unwrap();
        assert_eq!(first.swversion.as_deref(), Some("1.122.2"));
        assert_eq!(first.uniqueid, "00:17:88:01:08:aa:bb:cc-0b");
        assert_eq!(first.kind, HueKind::Light);
    }

    #[test]
    fn fetch_sensors_drops_internal_pseudo_sensors() {
        let addr = spawn_mock();
        let devs = fetch_sensors(&addr, TOKEN).unwrap();
        // Daylight hat keine uniqueid -> raus
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].name, "Motion Hallway");
        assert_eq!(devs[0].kind, HueKind::Sensor);
    }

    #[test]
    fn map_to_devices_preserves_firmware_relevant_data() {
        let addr = spawn_mock();
        let mut all = fetch_lights(&addr, TOKEN).unwrap();
        all.extend(fetch_sensors(&addr, TOKEN).unwrap());
        let devices = map_to_devices(&all);
        assert_eq!(devices.len(), 3);
        assert!(devices.iter().all(|d| d.source == "hue"));
        let motion = devices
            .iter()
            .find(|d| d.name == "Motion Hallway")
            .unwrap();
        assert_eq!(motion.kind.as_deref(), Some("sensor"));
        assert_eq!(motion.model.as_deref(), Some("Hue motion sensor"));
    }

    #[test]
    fn load_config_parses_multi_bridge_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hue.yaml");
        std::fs::write(
            &path,
            "- ip: 10.0.0.10\n  token: abc\n- ip: 10.0.0.11\n  token: def\n  name: outdoor_a\n",
        )
        .unwrap();
        let cfg = load_config(&path).unwrap();
        assert_eq!(cfg.len(), 2);
        assert_eq!(cfg[0].ip, "10.0.0.10");
        assert_eq!(cfg[1].name.as_deref(), Some("outdoor_a"));
    }
}
