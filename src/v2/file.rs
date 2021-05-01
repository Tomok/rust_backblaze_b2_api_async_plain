use super::{AccountId, BucketId, InvalidData, ServerSideEncryption};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileName(String);

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    START,
    UPLOAD,
    HIDE,
    FOLDER,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct FileId(String);

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
type SHA1 = String;
type MD5 = String;
type FileInfo = serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInformation {
    account_id: AccountId,
    action: FileAction,
    bucket_id: BucketId,
    content_length: u64,
    content_sha1: Option<SHA1>,
    content_md5: Option<MD5>,
    content_type: Option<String>,
    file_id: Option<FileId>,
    file_info: FileInfo,
    file_name: String,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: i64,
}
