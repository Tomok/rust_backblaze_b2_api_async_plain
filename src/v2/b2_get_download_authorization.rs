use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
    num::NonZeroU32,
};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors, serialize_header_option, ApiUrl, AuthorizationToken, BucketId,
    CacheControlHeaderValueRef, ContentDispositionRef, ContentEncodingRef, ContentLanguageRef,
    ContentTypeRef, DownloadOnlyAuthorizationToken, ExpiresHeaderValueRef, FileNamePrefix,
    JsonErrorObj,
};

#[derive(Debug)]
pub struct InvalidDownloadAuthorizationDurationError {
    value_attempted: u32,
}

impl InvalidDownloadAuthorizationDurationError {
    pub fn new(value_attempted: u32) -> Self {
        Self { value_attempted }
    }
}

impl std::error::Error for InvalidDownloadAuthorizationDurationError {}

impl Display for InvalidDownloadAuthorizationDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid download authorization duration: {} - must be a value between 1 and {}",
            self.value_attempted,
            ValidDownloadAuthorizationDurationInSeconds::max_download_authorization_duration_secs()
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidDownloadAuthorizationDurationInSeconds(NonZeroU32);

impl ValidDownloadAuthorizationDurationInSeconds {
    pub const fn max_download_authorization_duration_secs() -> u32 {
        604800u32
    }
}
impl TryFrom<u32> for ValidDownloadAuthorizationDurationInSeconds {
    type Error = InvalidDownloadAuthorizationDurationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if 1 <= value && value <= Self::max_download_authorization_duration_secs() {
            Ok(Self(value.try_into().unwrap())) // unwrap is safe as it was checked by the if above
        } else {
            Err(InvalidDownloadAuthorizationDurationError::new(value))
        }
    }
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct GetDownloadAuthorizationRequest<'s> {
    bucket_id: &'s BucketId,
    file_name_prefix: &'s FileNamePrefix,
    valid_duration_in_seconds: ValidDownloadAuthorizationDurationInSeconds,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_content_disposition: Option<ContentDispositionRef<'s>>,

    #[builder(default, setter(strip_option, into))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_content_language: Option<ContentLanguageRef<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_expires: Option<ExpiresHeaderValueRef<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_cache_control: Option<CacheControlHeaderValueRef<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_content_encoding: Option<ContentEncodingRef<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    b2_content_type: Option<ContentTypeRef<'s>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDownloadAuthorizationOk {
    bucket_id: BucketId,
    file_name_prefix: FileNamePrefix,
    authorization_token: DownloadOnlyAuthorizationToken,
}

impl GetDownloadAuthorizationOk {
    /// Get a reference to the get download authorization ok's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }

    /// Get a reference to the get download authorization ok's file name prefix.
    pub fn file_name_prefix(&self) -> &FileNamePrefix {
        &self.file_name_prefix
    }

    /// Get a reference to the get download authorization ok's authorization token.
    pub fn authorization_token(&self) -> &DownloadOnlyAuthorizationToken {
        &self.authorization_token
    }
}

pub async fn b2_get_download_authorization<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &'a AuthorizationToken,
    request_data: &'a GetDownloadAuthorizationRequest<'a>,
) -> Result<GetDownloadAuthorizationOk, errors::GetDownloadAuthorizationError> {
    let url = format!(
        "{}/b2api/v2/b2_get_download_authorization",
        api_url.as_str()
    );

    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&request_data);

    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
