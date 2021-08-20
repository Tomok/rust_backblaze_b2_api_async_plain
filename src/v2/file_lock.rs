use serde::{Deserialize, Serialize};

pub type FileRetention = serde_json::Value; // TODO!
pub type LegalHold = serde_json::Value; //TODO

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LegalHoldOnOff {
    On,
    Off,
}
