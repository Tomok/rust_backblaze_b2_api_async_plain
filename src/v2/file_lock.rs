use serde::{de, Deserialize, Serialize};

use super::TimeStamp;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "unit", rename_all = "camelCase")]
pub enum Period {
    Days { value: u64 },
    Years { value: u64 },
}

impl Period {
    pub fn days(days: u64) -> Self {
        Self::Days { value: days }
    }

    pub fn years(years: u64) -> Self {
        Self::Years { value: years }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum DefaultFileRetention {
    #[serde(rename = "null")]
    None,
    Compliance {
        period: Period,
    },
    Governance {
        period: Period,
    },
}

impl DefaultFileRetention {
    pub fn mode(&self) -> Option<FileRetentionMode> {
        match self {
            DefaultFileRetention::None => None,
            DefaultFileRetention::Compliance { period: _ } => Some(FileRetentionMode::Compliance),
            DefaultFileRetention::Governance { period: _ } => Some(FileRetentionMode::Governance),
        }
    }

    pub fn period(&self) -> Option<&Period> {
        match self {
            DefaultFileRetention::None => None,
            DefaultFileRetention::Compliance { period } => Some(period),
            DefaultFileRetention::Governance { period } => Some(period),
        }
    }
}

impl<'de> Deserialize<'de> for DefaultFileRetention {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialized = DeserialiableDefaultFileRetention::deserialize(deserializer)?;
        match deserialized.mode {
            None => match deserialized.period {
                Some(_) => Err(de::Error::invalid_value(de::Unexpected::Option, &"None")),
                None => Ok(Self::None),
            },
            Some(mode) => match deserialized.period {
                None => Err(de::Error::invalid_value(de::Unexpected::Option, &"Period")),
                Some(period) => match mode {
                    FileRetentionMode::Compliance => Ok(Self::Compliance { period }),
                    FileRetentionMode::Governance => Ok(Self::Governance { period }),
                },
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeserialiableDefaultFileRetention {
    #[serde(default)]
    mode: Option<FileRetentionMode>,
    #[serde(default)]
    period: Option<Period>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FileRetentionMode {
    Compliance,
    Governance,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileRetention {
    mode: Option<FileRetentionMode>,
    retain_until_timestamp: Option<TimeStamp>,
}

impl FileRetention {
    /// creates an enabled file retention setting
    pub fn new(mode: FileRetentionMode, retain_until_timestamp: TimeStamp) -> Self {
        Self {
            mode: Some(mode),
            retain_until_timestamp: Some(retain_until_timestamp),
        }
    }

    /// returns the file retention setting to be used, to disable file retention
    pub fn disabled() -> Self {
        Self {
            mode: None,
            retain_until_timestamp: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLockConfigurationValue {
    default_retention: DefaultFileRetention,
    is_file_lock_enabled: bool,
}

#[derive(Debug)]
pub enum FileLockConfiguration {
    ClientAuthorizedToRead { value: FileLockConfigurationValue },
    ClientNotAuthorizedToRead,
}

impl<'de> Deserialize<'de> for FileLockConfiguration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialized = DeserializeableFileLockConfiguration::deserialize(deserializer)?;
        if deserialized.is_client_authorized_to_read {
            match deserialized.value {
                None => Err(de::Error::invalid_value(
                    de::Unexpected::Option,
                    &"FileLockConfigurationValue",
                )),
                Some(value) => Ok(Self::ClientAuthorizedToRead { value }),
            }
        } else {
            match deserialized.value {
                Some(_) => Err(de::Error::invalid_value(de::Unexpected::Option, &"None")),
                None => Ok(Self::ClientNotAuthorizedToRead),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeserializeableFileLockConfiguration {
    is_client_authorized_to_read: bool,
    value: Option<FileLockConfigurationValue>,
}

pub type LegalHold = serde_json::Value; //TODO

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LegalHoldOnOff {
    On,
    Off,
}
