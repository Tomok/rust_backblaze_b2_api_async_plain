use std::fmt::Display;

use serde::{de, Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum ServerSideEncryption {
    None,
    SseB2,
    SseC,
}

impl Display for ServerSideEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServerSideEncryption::None => "none",
            ServerSideEncryption::SseB2 => "SSE-B2",
            ServerSideEncryption::SseC => "SSE-C",
        };
        f.write_str(s)
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

#[derive(Debug, Deserialize)]
struct SerializeableServerSideEncryption<'s> {
    mode: &'s str,
    algorithm: Option<&'s str>,
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
        let sse = SerializeableServerSideEncryption::deserialize(deserializer)?;
        match sse.mode {
            "SSE-B2" => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseB2),
            "SSE-C" => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseC),
            "none" => {
                if sse.algorithm.is_none() {
                    Ok(Self::None)
                } else {
                    Err(de::Error::unknown_field(&"algorithm", &[]))
                }
            }
            mode => Err(de::Error::unknown_variant(
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
