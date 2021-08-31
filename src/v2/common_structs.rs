//! Common structs used by multiple B2 API calls

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JsonErrorObj {
    pub status: http_types::StatusCode,
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
pub struct DownloadAuthorizationToken(String);

impl DownloadAuthorizationToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
