use crate::types::Device;
use anyhow::{Context, Result};
use serde::Deserialize;

/// Roh-Repraesentation einer HA-State-Antwort. Wir parsen nur die Felder,
/// die wir brauchen; der Rest darf da sein.
#[derive(Debug, Deserialize)]
pub struct HaEntity {
    pub entity_id: String,
    #[serde(default)]
    #[allow(dead_code)] // wird ggf. spaeter fuer Aktiv-Filter genutzt
    pub state: String,
    #[serde(default)]
    pub attributes: HaAttributes,
}

#[derive(Debug, Default, Deserialize)]
pub struct HaAttributes {
    #[serde(default)]
    pub friendly_name: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// Holt alle Entity-States aus Home Assistant.
/// base_url ohne /api-Suffix, z.B. "http://192.168.10.5:8123".
pub fn fetch_states(base_url: &str, token: &str) -> Result<Vec<HaEntity>> {
    let url = format!("{}/api/states", base_url.trim_end_matches('/'));
    let res = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/json")
        .call()
        .with_context(|| format!("GET {url}"))?;
    let body = res.into_string().context("HA-Response-Body lesen")?;
    let entities: Vec<HaEntity> =
        serde_json::from_str(&body).context("HA-States parsen")?;
    Ok(entities)
}

/// Filtert auf reale Geraete-Domains und mapped sie auf das kanonische Device.
pub fn map_to_devices(entities: &[HaEntity]) -> Vec<Device> {
    entities
        .iter()
        .filter(|e| is_device_entity(&e.entity_id))
        .map(to_device)
        .collect()
}

fn domain_of(entity_id: &str) -> &str {
    entity_id.split_once('.').map(|x| x.0).unwrap_or("")
}

fn is_device_entity(entity_id: &str) -> bool {
    matches!(
        domain_of(entity_id),
        "light"
            | "switch"
            | "sensor"
            | "binary_sensor"
            | "climate"
            | "cover"
            | "fan"
            | "media_player"
            | "lock"
            | "vacuum"
            | "camera"
            | "valve"
            | "water_heater"
            | "button"
            | "humidifier"
            | "siren"
    )
}

fn to_device(e: &HaEntity) -> Device {
    let domain = domain_of(&e.entity_id);
    Device {
        source: "ha".into(),
        source_id: e.entity_id.clone(),
        name: e
            .attributes
            .friendly_name
            .clone()
            .unwrap_or_else(|| e.entity_id.clone()),
        manufacturer: e.attributes.manufacturer.clone(),
        model: e.attributes.model.clone(),
        kind: Some(domain.to_string()),
        room: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tiny_http::{Header, Response, Server};

    const FIXTURE: &str = r#"[
        {"entity_id": "light.kueche", "state": "on",
         "attributes": {"friendly_name": "Kueche Decke", "manufacturer": "IKEA Tradfri", "model": "LED2003G10"}},
        {"entity_id": "binary_sensor.tuer_eingang", "state": "off",
         "attributes": {"friendly_name": "Tuer Eingang"}},
        {"entity_id": "sensor.temperatur_wohnzimmer", "state": "21.5",
         "attributes": {"friendly_name": "Temp Wohnzimmer", "unit_of_measurement": "C"}},
        {"entity_id": "automation.morgens", "state": "on", "attributes": {}},
        {"entity_id": "weather.home", "state": "sunny", "attributes": {}},
        {"entity_id": "sun.sun", "state": "above_horizon", "attributes": {}}
    ]"#;

    const TOKEN: &str = "secret-llat-token";

    fn spawn_mock() -> String {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap().to_string();
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

                let expected = format!("Bearer {TOKEN}");
                let resp = if req.url() == "/api/states" && auth == expected {
                    Response::from_string(FIXTURE)
                        .with_status_code(200)
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        )
                } else if req.url() == "/api/states" {
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
    fn fetch_states_returns_all_entities() {
        let addr = spawn_mock();
        let entities = fetch_states(&format!("http://{addr}"), TOKEN).unwrap();
        assert_eq!(entities.len(), 6);
        assert_eq!(entities[0].entity_id, "light.kueche");
        assert_eq!(
            entities[0].attributes.friendly_name.as_deref(),
            Some("Kueche Decke")
        );
    }

    #[test]
    fn fetch_states_fails_with_wrong_token() {
        let addr = spawn_mock();
        let err = fetch_states(&format!("http://{addr}"), "wrong").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("GET") && msg.contains("/api/states"));
    }

    #[test]
    fn fetch_states_handles_trailing_slash() {
        let addr = spawn_mock();
        let entities = fetch_states(&format!("http://{addr}/"), TOKEN).unwrap();
        assert_eq!(entities.len(), 6);
    }

    #[test]
    fn map_filters_out_non_devices() {
        let entities: Vec<HaEntity> = serde_json::from_str(FIXTURE).unwrap();
        let devices = map_to_devices(&entities);
        // light, binary_sensor, sensor sind Geraete; automation/weather/sun nicht.
        assert_eq!(devices.len(), 3, "got: {devices:?}");
        assert!(devices.iter().all(|d| d.source == "ha"));
        let ids: Vec<&str> = devices.iter().map(|d| d.source_id.as_str()).collect();
        assert!(ids.contains(&"light.kueche"));
        assert!(ids.contains(&"binary_sensor.tuer_eingang"));
        assert!(ids.contains(&"sensor.temperatur_wohnzimmer"));
    }

    #[test]
    fn map_preserves_friendly_name_manufacturer_kind() {
        let entities: Vec<HaEntity> = serde_json::from_str(FIXTURE).unwrap();
        let devices = map_to_devices(&entities);
        let kueche = devices
            .iter()
            .find(|d| d.source_id == "light.kueche")
            .unwrap();
        assert_eq!(kueche.name, "Kueche Decke");
        assert_eq!(kueche.manufacturer.as_deref(), Some("IKEA Tradfri"));
        assert_eq!(kueche.model.as_deref(), Some("LED2003G10"));
        assert_eq!(kueche.kind.as_deref(), Some("light"));
    }

    #[test]
    fn map_falls_back_to_entity_id_when_no_friendly_name() {
        let json = r#"[{"entity_id": "switch.namenslos", "state": "on", "attributes": {}}]"#;
        let entities: Vec<HaEntity> = serde_json::from_str(json).unwrap();
        let devices = map_to_devices(&entities);
        assert_eq!(devices[0].name, "switch.namenslos");
    }
}
