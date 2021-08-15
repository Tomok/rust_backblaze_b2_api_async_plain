use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Display};

#[derive(Debug)]
pub struct InvalidPartNumberError {
    value: u16,
}

impl std::error::Error for InvalidPartNumberError {}

impl Display for InvalidPartNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid Part number: {} - must be a value between 1 and 10000",
            self.value
        )
    }
}

/// number of a file part, starts with 1, is 10000 at most.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct PartNumber(u16);

impl TryFrom<u16> for PartNumber {
    type Error = InvalidPartNumberError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if 1u16 <= value && value <= 10000u16 {
            Ok(Self(value))
        } else {
            Err(Self::Error { value })
        }
    }
}
