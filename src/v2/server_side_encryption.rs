use std::fmt::Display;

use serde::{de, Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum ServerSideEncryption {
    None,
    SseB2,
    SseC,
}

impl ServerSideEncryption {
    fn as_str(&self) -> &'static str {
        match self {
            ServerSideEncryption::None => "none",
            ServerSideEncryption::SseB2 => "SSE-B2",
            ServerSideEncryption::SseC => "SSE-C",
        }
    }
}

impl Display for ServerSideEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum EncryptionAlgorithm {
    Aes256,
}

impl ServerSideEncryption {
    pub fn algorithm(&self) -> Option<EncryptionAlgorithm> {
        match self {
            ServerSideEncryption::None => None,
            ServerSideEncryption::SseB2 => Some(EncryptionAlgorithm::Aes256),
            ServerSideEncryption::SseC => Some(EncryptionAlgorithm::Aes256),
        }
    }
}

impl Serialize for ServerSideEncryption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable: SerializeableServerSideEncryption = self.into();
        serializable.serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SerializeableServerSideEncryption<'s> {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    algorithm: Option<&'s str>,
    #[serde(default)]
    mode: Option<&'s str>,
}

impl<'s> From<&'s ServerSideEncryption> for SerializeableServerSideEncryption<'static> {
    fn from(sse: &'s ServerSideEncryption) -> Self {
        let mode = Some(sse.as_str());
        let algorithm = match sse {
            ServerSideEncryption::None => None,
            ServerSideEncryption::SseB2 | ServerSideEncryption::SseC => Some("AES256"),
        };
        Self { mode, algorithm }
    }
}

/// small helper function that checks, if a is Some("AES256"), if not it raises a matching deserialization error
fn is_aes_encryption_algorithm<E>(a: Option<&str>) -> Result<(), E>
where
    E: de::Error,
{
    match a {
        Some("AES256") => Ok(()),
        Some(other) => Err(de::Error::invalid_value(
            de::Unexpected::Str(other),
            &"AES256",
        )),
        None => Err(de::Error::missing_field("algorithm")),
    }
}

impl<'de> Deserialize<'de> for ServerSideEncryption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        dbg!("THL1");
        let sse = SerializeableServerSideEncryption::deserialize(deserializer)?;
        dbg!(&sse);
        match sse.mode {
            Some("SSE-B2") => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseB2),
            Some("SSE-C") => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseC),
            None | Some("none") => {
                if sse.algorithm.is_none() {
                    Ok(Self::None)
                } else {
                    Err(de::Error::unknown_field(&"algorithm", &[]))
                }
            }
            Some(mode) => Err(de::Error::unknown_variant(
                mode,
                &["none", "SSE-B2", "SSE-C"],
            )),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServerSideEncryptionCustomerKey {
    //TODO
}
