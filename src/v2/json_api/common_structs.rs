//! Common structs used by multiple B2 API calls

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonErrorObj {
    pub status: http_types::StatusCode,
    pub code: String,
    pub message: String,
}

impl JsonErrorObj {
    pub fn machine_readable<'a>(&'a self) -> MachineReadableJsonErrorObj<'a> {
        MachineReadableJsonErrorObj {
            status: self.status,
            code: &self.code,
        }
    }
}

/// The parts of JsonErrorObj that are intended to be read by code
/// usefull for match statements
#[derive(Debug, PartialEq, Eq)]
pub struct MachineReadableJsonErrorObj<'a> {
    pub status: http_types::StatusCode,
    pub code: &'a str,
}

#[derive(Debug)]
pub enum Error {
    JSONError(JsonErrorObj),
    ReqwestError(reqwest::Error),
}
