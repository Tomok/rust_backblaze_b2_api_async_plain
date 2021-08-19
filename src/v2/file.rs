use super::{AccountId, BucketId, FileRetention, InvalidData, LegalHold, ServerSideEncryption};
use serde::{de, Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct FileName(String);

impl FileName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileName {
    type Error = InvalidData;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        if bytes.len() <= 1024 {
            for (i, c) in value.chars().enumerate() {
                let v = c as u32;
                if v < 32 {
                    return Err(InvalidData::new(format!("Invalid character {:#?} at position {} - characters below 32 are not allowed", c, i)));
                }
                if v == 127 {
                    return Err(InvalidData::new(format!(
                        "Invalid DEL character at position {}",
                        i
                    )));
                }
            }
            Ok(Self(value))
        } else {
            Err(InvalidData::new(format!(
                "To many bytes in Filename, at most 1024 are allowed, but {:#?} were used",
                bytes.len()
            )))
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    Start,
    Upload,
    Hide,
    Folder,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileId(String);

impl FileId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileId {
    type Error = InvalidData;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 200 {
            Err(InvalidData::new(format!(
                "File ID may not have more than 200 characters, but had {} characters",
                value.len()
            )))
        } else {
            Ok(Self(value))
        }
    }
}

// TODO: more specific types...
pub type Sha1 = String;
pub type Md5 = String;
pub type FileInfo = serde_json::Value;
pub type TimeStamp = i64;
/// Content Disposition value acc. to the grammar specified in RFC 6266
pub type ContentDisposition = String; //TODO: create struct and check for RFC-6266 compliance
/// Content Language value acc. to RFC 2616
pub type ContentLanguage = String; //TODO: create struct and check for RFC compliance
/// expires header acc. to RFC 2616
pub type ExpiresHeaderValue = String; //TODO: create struct and check for RFC compliance
/// expires cache-control header value acc. to RFC 2616
pub type CacheControlHeaderValue = String; //TODO: create struct and check for RFC compliance
/// expires content-encoding header value acc. to RFC 2616
pub type ContentEncodingHeaderValue = String; //TODO: create struct and check for RFC compliance

/// own Mime type based on [http_types::Mime] to add Serde Support
#[derive(Debug, PartialEq, Eq)]
pub struct Mime(http_types::Mime);

impl Mime {
    /// Use this mime type to have the server determine the mime type by file extension.
    /// The content type mappings can be found here: <https://www.backblaze.com/b2/docs/content-types.html>:
    pub fn auto() -> Self {
        Self::from_str("b2/auto").unwrap()
    }
}

impl FromStr for Mime {
    type Err = http_types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(http_types::Mime::from_str(s)?))
    }
}

impl Mime {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Serialize for Mime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        http_types::Mime::from_str(&s)
            .map(|m| Self(m))
            .map_err(|_e| de::Error::invalid_value(de::Unexpected::Str(&s), &"Valid Mime type"))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInformation {
    account_id: AccountId,
    action: FileAction,
    bucket_id: BucketId,
    content_length: u64,
    content_sha1: Option<Sha1>,
    content_md5: Option<Md5>,
    content_type: Option<Mime>,
    file_id: Option<FileId>,
    file_info: FileInfo,
    file_name: String,
    file_retention: Option<FileRetention>,
    legal_hold: Option<LegalHold>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

impl FileInformation {
    /// Get a reference to the file information's file id.
    pub fn file_id(&self) -> Option<&FileId> {
        self.file_id.as_ref()
    }
}
