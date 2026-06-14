use serde::{Deserialize, Serialize};

/// Ein Geraet im Inventar. Spiegelt die Tabelle `devices` ohne die
/// DB-verwalteten Felder (id, first_seen, last_seen, active).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Device {
    pub source: String,
    pub source_id: String,
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
}
