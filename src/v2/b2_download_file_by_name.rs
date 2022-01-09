use super::{
    errors::DownloadFileError, serialize_header_option, BucketName, CacheControlHeaderValueRef,
    ContentDispositionRef, ContentEncodingRef, ContentLanguageRef, ContentTypeRef,
    DownloadAuthorizationToken, DownloadUrl, ExpiresHeaderValueRef, FileName, JsonErrorObj,
    ServerSideEncryptionCustomerKey,
};

use headers::{HeaderMap, HeaderMapExt};
use serde::Serialize;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct DownloadFileByNameRequest<'s, AuthToken>
where
    AuthToken: 's + DownloadAuthorizationToken + Serialize,
{
    bucket_name: &'s BucketName,
    file_name: &'s FileName,
    #[builder(default, setter(strip_option))]
    range: Option<&'s headers::Range>,
    #[builder(default, setter(strip_option))]
    authorization: Option<&'s AuthToken>,
    #[builder(default, setter(strip_option))]
    b2_content_disposition: Option<ContentDispositionRef<'s>>,
    #[builder(default, setter(strip_option))]
    b2_content_language: Option<ContentLanguageRef<'s>>,
    #[builder(default, setter(strip_option))]
    b2_expires: Option<ExpiresHeaderValueRef<'s>>,
    #[builder(default, setter(strip_option))]
    b2_cache_control: Option<CacheControlHeaderValueRef<'s>>,
    #[builder(default, setter(strip_option))]
    b2_content_encoding: Option<ContentEncodingRef<'s>>,
    #[builder(default, setter(strip_option))]
    b2_content_type: Option<ContentTypeRef<'s>>,
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
}

impl<'s, AuthToken> DownloadFileByNameRequest<'s, AuthToken>
where
    AuthToken: DownloadAuthorizationToken + Serialize,
{
    /// returns the parameters that cannot be passed as headers as [DownloadFileByNameUrlParameters]
    fn as_url_params(&'s self) -> DownloadFileByNameUrlParameters<'s> {
        DownloadFileByNameUrlParameters {
            b2_content_disposition: self.b2_content_disposition,
            b2_content_language: self.b2_content_language,
            b2_expires: self.b2_expires,
            b2_cache_control: self.b2_cache_control,
            b2_content_encoding: self.b2_content_encoding,
            b2_content_type: self.b2_content_type,
        }
    }
}

/// Parameters to generate a download url
/// these intentionally do not include Authorization and server-side-encryption authentication,
/// as those might be cached by intermediate servers, apear in logs, ...
/// so it is better to pass them as headers instead
#[derive(Debug, TypedBuilder, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFileByNameUrlParameters<'s> {
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

/// gets a download URL for a given file in a given directory
///
/// public as it might be usefull to give this url to a different application/client/...
pub fn get_b2_download_file_by_name_url<'s>(
    download_url: &DownloadUrl,
    bucket_name: &BucketName,
    file_name: &FileName,
    params: &DownloadFileByNameUrlParameters<'s>,
) -> String {
    format!(
        "{}/file/{}/{}?{}",
        download_url.as_str(),
        bucket_name.as_str(),
        file_name.as_str(),
        serde_urlencoded::to_string(params).unwrap()
    )
}

/// downloads a file by Name, does return a reqwest::Response object, if the server returned http status OK (200)
/// or PartialContent (206) if a range was used.
pub async fn b2_download_file_by_name<'a, 'b, AuthToken>(
    download_url: &DownloadUrl,
    request: &DownloadFileByNameRequest<'b, AuthToken>,
) -> Result<reqwest::Response, DownloadFileError>
where
    AuthToken: DownloadAuthorizationToken + Serialize,
{
    let url = get_b2_download_file_by_name_url(
        download_url,
        request.bucket_name,
        request.file_name,
        &request.as_url_params(),
    );
    let mut request_builder = reqwest::Client::new().get(url);
    if let Some(auth) = request.authorization {
        request_builder = request_builder.header("Authorization", auth.download_token_as_str());
    }
    if let Some(sse) = request.server_side_encryption {
        request_builder = sse.add_to_request_as_header(request_builder);
    }
    if let Some(range) = request.range {
        let mut headers = HeaderMap::with_capacity(1);
        headers.typed_insert(range.clone());
        request_builder = request_builder.headers(headers);
    }
    let resp = request_builder
        .send()
        .await
        .map_err(DownloadFileError::from)?;
    let expected_status = if request.range.is_none() {
        http::StatusCode::OK
    } else {
        http::StatusCode::PARTIAL_CONTENT
    };
    if resp.status().as_u16() == expected_status {
        Ok(resp)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
