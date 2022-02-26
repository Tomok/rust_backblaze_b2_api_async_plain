use super::{
    errors::DownloadFileError, serialize_header_option, AuthorizationToken,
    CacheControlHeaderValueRef, ContentDispositionRef, ContentEncodingRef, ContentLanguageRef,
    ContentTypeRef, DownloadUrl, ExpiresHeaderValueRef, FileId, ServerSideEncryptionCustomerKey,
};

use headers::{HeaderMap, HeaderMapExt};
use serde::Serialize;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadParams<'s> {
    file_id: &'s FileId,

    #[builder(default, setter(strip_option))]
    #[serde(skip)] //serialized manually, as it may not be a url parameter
    range: Option<&'s headers::Range>,

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

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey>,
}

/// downloads a file by ID, does return a reqwest::Response object, if the server returned http status OK (200)
/// or PartialContent (206) if a range was used.
pub async fn b2_download_file_by_id(
    download_url: &DownloadUrl,
    authorization_token: Option<&AuthorizationToken>,
    params: &DownloadParams<'_>,
) -> Result<reqwest::Response, DownloadFileError> {
    let url = format!(
        "{}/b2api/v2/b2_download_file_by_id?{}",
        download_url.as_str(),
        serde_urlencoded::to_string(params).unwrap()
    );
    let mut headers = HeaderMap::with_capacity(1);
    if let Some(range) = params.range {
        headers.typed_insert(range.clone());
    }

    let mut request_builder = reqwest::Client::new().get(url).headers(headers);
    if let Some(auth) = authorization_token {
        request_builder = request_builder.header("Authorization", auth.as_str());
    }
    let resp = request_builder
        .send()
        .await
        .map_err(DownloadFileError::from)?;
    let expected_status = if params.range.is_none() {
        http::StatusCode::OK
    } else {
        http::StatusCode::PARTIAL_CONTENT
    };
    if resp.status() == expected_status {
        Ok(resp)
    } else {
        Err(DownloadFileError::from_response(resp).await)
    }
}
