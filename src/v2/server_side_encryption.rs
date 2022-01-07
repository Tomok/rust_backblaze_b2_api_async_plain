use std::fmt::Display;

use reqwest::RequestBuilder;
use serde::{de, Deserialize, Serialize};

use super::Md5DigestRef;

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
        let sse = SerializeableServerSideEncryption::deserialize(deserializer)?;
        match sse.mode {
            Some("SSE-B2") => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseB2),
            Some("SSE-C") => is_aes_encryption_algorithm(sse.algorithm).map(|_| Self::SseC),
            None | Some("none") => {
                if sse.algorithm.is_none() {
                    Ok(Self::None)
                } else {
                    Err(de::Error::unknown_field("algorithm", &[]))
                }
            }
            Some(mode) => Err(de::Error::unknown_variant(
                mode,
                &["none", "SSE-B2", "SSE-C"],
            )),
        }
    }
}

const CUSTOMER_KEY_BYTES: usize = 32usize;

#[derive(Debug, Serialize, Clone)]
#[serde(into = "SerializableServerSideEncryptionCustomerKey")]
pub enum ServerSideEncryptionCustomerKey<'s> {
    None,
    SseB2,
    SseC {
        customer_key: &'s [u8; CUSTOMER_KEY_BYTES],
        customer_key_md5: Md5DigestRef<'s>,
    },
}

impl<'s> ServerSideEncryptionCustomerKey<'s> {
    /// Adds these values to a RequestBuilder as headers
    pub(crate) fn add_to_request_as_header(&self, request: RequestBuilder) -> RequestBuilder {
        match self {
            ServerSideEncryptionCustomerKey::None => request,
            ServerSideEncryptionCustomerKey::SseB2 => {
                request.header("X-Bz-Server-Side-Encryption", "AES256")
            }
            ServerSideEncryptionCustomerKey::SseC {
                customer_key,
                customer_key_md5,
            } => request
                .header("X-Bz-Server-Side-Encryption-Customer-Algorithm", "AES256")
                .header(
                    "X-Bz-Server-Side-Encryption-Customer-Key",
                    base64_encode_sse(customer_key),
                )
                .header(
                    "X-Bz-Server-Side-Encryption-Customer-Key-Md5",
                    base64_encode_sse(customer_key_md5.bytes()),
                ),
        }
    }
}

const BASE64_CONFIG: base64::Config = base64::Config::new(base64::CharacterSet::Standard, false);

pub(crate) fn base64_encode_sse(s: impl AsRef<[u8]>) -> String {
    base64::encode_config(s, BASE64_CONFIG)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableServerSideEncryptionCustomerKey {
    algorithm: Option<&'static str>,
    #[serde(default)]
    mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_key: Option<String>, // base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_key_md5: Option<String>, // base64 encoded
}

impl<'s> From<ServerSideEncryptionCustomerKey<'s>> for SerializableServerSideEncryptionCustomerKey {
    fn from(sse_ck: ServerSideEncryptionCustomerKey) -> Self {
        match sse_ck {
            ServerSideEncryptionCustomerKey::None => Self {
                algorithm: None,
                mode: None,
                customer_key: None,
                customer_key_md5: None,
            },
            ServerSideEncryptionCustomerKey::SseB2 => Self {
                algorithm: Some("AES256"),
                mode: Some("SSE-B2"),
                customer_key: None,
                customer_key_md5: None,
            },
            ServerSideEncryptionCustomerKey::SseC {
                customer_key,
                customer_key_md5,
            } => {
                let key = base64_encode_sse(customer_key);
                //assert_eq!(CUSTOMER_KEY_ENCODED_BYTES, key.len());
                let key_md5 = base64_encode_sse(customer_key_md5.bytes());
                //assert_eq!(CUSTOMER_KEY_MD5_ENCODED_BYTES, key_md5.len());
                Self {
                    algorithm: Some("AES256"),
                    mode: Some("SSE-C"),
                    customer_key: Some(key),
                    customer_key_md5: Some(key_md5),
                }
            }
        }
    }
}
