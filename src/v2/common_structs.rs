//! Common structs used by multiple B2 API calls

use std::fmt::{Debug, Display};

use http::StatusCode;
use serde::{Deserialize, Serialize};

// needed in mock_server, hence public for crate
pub mod status_code_serialization {
    use std::convert::TryInto;

    use http::StatusCode;
    use serde::{
        de::{self, Unexpected},
        Deserializer, Serializer,
    };

    pub fn serialize<S>(status_code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(status_code.as_u16())
    }

    struct Visitor {}

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = StatusCode;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "an unsigned integer that is a valid HTTP Status")
        }

        // overwriting visit_u64 since other trait functions forward to this
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let v_u16 = v.try_into().map_err(|_| {
                de::Error::invalid_value(Unexpected::Unsigned(v), &"Valid http status code")
            })?;
            StatusCode::from_u16(v_u16).map_err(|_| {
                de::Error::invalid_value(Unexpected::Unsigned(v), &"Valid http status code")
            })
        }

        //overwrite just in case signed int is used
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(v.try_into().map_err(|_| {
                de::Error::invalid_value(Unexpected::Signed(v), &"Valid http status code")
            })?)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(Visitor {})
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JsonErrorObj {
    #[serde(with = "status_code_serialization")]
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

/// Invalid data was received, the contents of message are subject to change,
/// so please do not implement logic based on those.
#[derive(Debug)]
pub struct InvalidData {
    message: String,
}

impl InvalidData {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    /// Get a reference to the invalid data's message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for InvalidData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Data Received: {}", self.message)
    }
}

impl std::error::Error for InvalidData {}

#[derive(Debug)]
pub enum Error {
    JsonError(JsonErrorObj),
    ReqwestError(reqwest::Error),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiUrl(pub(crate) String);

impl ApiUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadUrl(String);

impl DownloadUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountId(pub(crate) String);

impl AccountId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationToken(pub(crate) String);

impl AuthorizationToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadOnlyAuthorizationToken(String);

impl DownloadOnlyAuthorizationToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait DownloadAuthorizationToken {
    fn download_token_as_str(&self) -> &str;
}

impl DownloadAuthorizationToken for AuthorizationToken {
    fn download_token_as_str(&self) -> &str {
        self.as_str()
    }
}

impl DownloadAuthorizationToken for DownloadOnlyAuthorizationToken {
    fn download_token_as_str(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug)]
pub struct InvalidCharacterError {
    character: char,
    position: usize,
    expected: &'static str,
}

impl InvalidCharacterError {
    pub fn new(character: char, position: usize, expected: &'static str) -> Self {
        Self {
            character,
            position,
            expected,
        }
    }

    /// Get the invalid character
    pub fn character(&self) -> char {
        self.character
    }

    /// Get the invalid character error's position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get a textual description of the allowed characters
    pub fn expected(&self) -> &'static str {
        self.expected
    }
}

impl std::error::Error for InvalidCharacterError {}

impl Display for InvalidCharacterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "found unexpected character {:?} in String at position {}. Allowed are {}",
            self.character, self.position, self.expected
        )
    }
}

#[derive(Debug)]
pub struct InvalidLengthError {
    /// the length of the String passed in
    length: usize,
    /// the maximal allowed length
    allowed_length: usize,
}

impl InvalidLengthError {
    pub fn new(length: usize, allowed_length: usize) -> Self {
        Self {
            length,
            allowed_length,
        }
    }

    /// Get the invalid length.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Get the allowed length.
    pub fn allowed_length(&self) -> usize {
        self.allowed_length
    }
}
impl std::error::Error for InvalidLengthError {}

impl Display for InvalidLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Passed String is longer than allowed: {} > {}",
            self.length, self.allowed_length
        )
    }
}

#[derive(Debug)]
pub enum StringSpecializationError {
    InvalidCharacterError(InvalidCharacterError),
    InvalidLengthError(InvalidLengthError),
}

impl StringSpecializationError {
    pub fn invalid_length(length: usize, allowed_length: usize) -> Self {
        Self::InvalidLengthError(InvalidLengthError::new(length, allowed_length))
    }
    pub fn invalid_character(character: char, position: usize, expected: &'static str) -> Self {
        Self::InvalidCharacterError(InvalidCharacterError::new(character, position, expected))
    }
}

impl Display for StringSpecializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringSpecializationError::InvalidCharacterError(e) => std::fmt::Display::fmt(&e, f),
            StringSpecializationError::InvalidLengthError(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for StringSpecializationError {}
