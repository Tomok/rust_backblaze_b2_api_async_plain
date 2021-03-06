use super::{
    AccountId, BucketId, FileLegalHold, FileRetention, InvalidCharacterError, ServerSideEncryption,
    StringSpecializationError,
};
use headers::CacheControl;
use hex::{FromHex, FromHexError, ToHex};
use lazy_static::lazy_static;
use mime::Mime;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, str::FromStr};

/// a character used to delimit a query for a list of files
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FileNameDelimiter(char);

impl FileNameDelimiter {}

impl TryFrom<char> for FileNameDelimiter {
    type Error = InvalidCharacterError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let v = value as u32;
        if v < 32 || v == 127 {
            Err(Self::Error::new(
                value,
                0,
                "An UTF-8 character without characters below 32 and DEL (code 127)",
            ))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FileName(String);

impl FileName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileName {
    type Error = StringSpecializationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::Error::check_length(&value, 1, 1024)?;
        Self::Error::check_characters(
            &value,
            |c| {
                let v = c as u32;
                !(v < 32 || v == 127)
            },
            "UTF-8 string without characters below 32 and DEL (code 127)",
        )?;
        Ok(Self(value))
    }
}

/// A prefix for [FileName]s contrary to [FileName] this may be empty
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FileNamePrefix(String);

impl FileNamePrefix {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileNamePrefix {
    type Error = StringSpecializationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::Error::check_length(&value, 0, 1024)?;
        Self::Error::check_characters(
            &value,
            |c| {
                let v = c as u32;
                !(v < 32 || v == 127)
            },
            "UTF-8 string without characters below 32, and DEL (code 127)",
        )?;
        Ok(Self(value))
    }
}

impl TryFrom<FileNamePrefix> for FileName {
    type Error = StringSpecializationError;

    fn try_from(value: FileNamePrefix) -> Result<Self, Self::Error> {
        let len = value.as_str().len();
        if len <= 1 {
            Err(StringSpecializationError::invalid_length(len, 1, 1024))
        } else {
            Ok(Self(value.0))
        }
    }
}

impl From<FileName> for FileNamePrefix {
    fn from(file_name: FileName) -> Self {
        Self(file_name.0)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    Start,
    Upload,
    Hide,
    Folder,
    Copy, //not in documentation, but was returned on b2_copy_file...
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileId(String);

impl FileId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileId {
    type Error = StringSpecializationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::Error::check_length(&value, 1, 200)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String")]
pub struct Sha1Digest {
    bytes: [u8; 20],
}

impl From<Sha1Digest> for String {
    // returns the digest as String
    fn from(s: Sha1Digest) -> Self {
        s.bytes.encode_hex()
    }
}

impl<'a> TryFrom<&'a str> for Sha1Digest {
    type Error = FromHexError;

    /// try to get the digest from a hex string
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self {
            bytes: <[u8; 20]>::from_hex(value)?,
        })
    }
}

impl Sha1Digest {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self { bytes }
    }
}

#[cfg(feature = "sha1")]
impl From<sha1::Digest> for Sha1Digest {
    fn from(digest: sha1::Digest) -> Self {
        Self {
            bytes: digest.bytes(),
        }
    }
}

pub type Sha1DigestRef<'s> = &'s Sha1Digest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String")]
pub struct Md5Digest {
    bytes: [u8; 16],
}

impl Md5Digest {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8; 16] {
        &self.bytes
    }
}

impl<'a> TryFrom<&'a str> for Md5Digest {
    type Error = FromHexError;

    /// try to get the digest from a hex string
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self {
            bytes: <[u8; 16]>::from_hex(value)?,
        })
    }
}

#[cfg(feature = "md5")]
impl From<md5::Digest> for Md5Digest {
    fn from(digest: md5::Digest) -> Self {
        Self { bytes: digest.0 }
    }
}

impl From<Md5Digest> for String {
    fn from(m: Md5Digest) -> Self {
        m.bytes.encode_hex()
    }
}

pub type Md5DigestRef<'s> = &'s Md5Digest;
#[cfg(feature = "b2_unstable")]
pub type FileInfo = serde_json::Value;
pub type TimeStamp = i64;
/// Content Disposition value acc. to the grammar specified in RFC 6266
pub type ContentDisposition = headers::ContentDisposition;
pub type ContentDispositionRef<'s> = &'s ContentDisposition;

mod content_language;
pub use content_language::ContentLanguage;
pub type ContentLanguageRef<'s> = &'s ContentLanguage;
/// expires header acc. to RFC 2616
pub type ExpiresHeaderValue = headers::Expires;
pub type ExpiresHeaderValueRef<'s> = &'s ExpiresHeaderValue;
/// expires cache-control header value acc. to RFC 2616
pub type CacheControlHeaderValue = CacheControl;
pub type CacheControlHeaderValueRef<'s> = &'s CacheControl;
/// expires content-encoding header value acc. to RFC 2616
pub type ContentEncoding = headers::ContentEncoding;
pub type ContentEncodingRef<'s> = &'s ContentEncoding;

/// expires content-type header value acc. to RFC 2616
pub type ContentType = headers::ContentType;
pub type ContentTypeRef<'s> = &'s ContentType;
lazy_static! {
    pub static ref CONTENT_TYPE_AUTO: ContentType =
        ContentType::from("b2/auto".parse::<mime::Mime>().unwrap());
}

struct MimeVisitor {}

impl<'de> de::Visitor<'de> for MimeVisitor {
    type Value = Mime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "A string representing a Mime")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Mime::from_str(v).map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
    }
}

fn deserialize_mime<'de, D>(deserializer: D) -> Result<Mime, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(MimeVisitor {})
}

struct MimeOptionVisitor {}

impl<'de> de::Visitor<'de> for MimeOptionVisitor {
    type Value = Option<Mime>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "A string representing a Mime or None")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Some(deserialize_mime(deserializer)?))
    }
}

fn deserialize_mime_option<'de, D>(deserializer: D) -> Result<Option<Mime>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(MimeOptionVisitor {})
}

/// the Backblaze API can return a String "none" for a not set sha1 sum ...
/// as this still should be mapped to [Option::None] a custom deserializer is necessary
struct Sha1OptionVisitor {}

impl<'de> de::Visitor<'de> for Sha1OptionVisitor {
    type Value = Option<Sha1Digest>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "A string representing an SHA1 digest as hex, \"None\"(as string), or None (as the option type)")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(if s == "none" {
            None
        } else {
            Some(
                Sha1Digest::try_from(s)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))?,
            )
        })
    }
}

fn deserialize_sha1_option<'de, D>(deserializer: D) -> Result<Option<Sha1Digest>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(Sha1OptionVisitor {})
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInformation {
    account_id: AccountId,
    action: FileAction,
    bucket_id: BucketId,
    content_length: u64,
    #[serde(deserialize_with = "deserialize_sha1_option")]
    content_sha1: Option<Sha1Digest>,
    content_md5: Option<Md5Digest>,
    #[serde(deserialize_with = "deserialize_mime_option", default)]
    content_type: Option<mime::Mime>,
    file_id: Option<FileId>,

    #[cfg(feature = "b2_unstable")]
    file_info: FileInfo,
    file_name: FileName,
    file_retention: Option<FileRetention>,
    legal_hold: Option<FileLegalHold>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

impl FileInformation {
    /// Get a reference to the file information's file id.
    pub fn file_id(&self) -> Option<&FileId> {
        self.file_id.as_ref()
    }

    /// Get a reference to the file information's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the file information's action.
    pub fn action(&self) -> &FileAction {
        &self.action
    }

    /// Get a reference to the file information's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }

    /// Get a reference to the file information's content length.
    pub fn content_length(&self) -> &u64 {
        &self.content_length
    }

    /// Get a reference to the file information's content sha1.
    pub fn content_sha1(&self) -> Option<&Sha1Digest> {
        self.content_sha1.as_ref()
    }

    /// Get a reference to the file information's content md5.
    pub fn content_md5(&self) -> Option<&Md5Digest> {
        self.content_md5.as_ref()
    }

    #[cfg(feature = "b2_unstable")]
    /// Get a reference to the file information's file info.
    pub fn file_info(&self) -> &FileInfo {
        &self.file_info
    }

    /// Get a reference to the file information's file retention.
    pub fn file_retention(&self) -> Option<&FileRetention> {
        self.file_retention.as_ref()
    }

    /// Get a reference to the file information's legal hold.
    pub fn legal_hold(&self) -> Option<&FileLegalHold> {
        self.legal_hold.as_ref()
    }

    /// Get a reference to the file information's server side encryption.
    pub fn server_side_encryption(&self) -> Option<&ServerSideEncryption> {
        self.server_side_encryption.as_ref()
    }

    /// Get a reference to the file information's upload timestamp.
    pub fn upload_timestamp(&self) -> &TimeStamp {
        &self.upload_timestamp
    }

    /// Get a reference to the file information's file name.
    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }

    /// Get a reference to the file information's content type.
    pub fn content_type(&self) -> Option<&Mime> {
        self.content_type.as_ref()
    }
}
