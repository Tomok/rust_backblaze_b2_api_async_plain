use reqwest::Body;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::header_serializer::HeadersFrom;

use super::{
    errors::UploadFileError, serialize_content_type_header, serialize_header_option,
    CacheControlHeaderValueRef, ContentDispositionRef, ContentLanguageRef, ContentTypeRef,
    ExpiresHeaderValueRef, FileInformation, FileName, ServerSideEncryptionCustomerKey,
    Sha1DigestRef, TimeStamp, UploadParameters, CONTENT_TYPE_AUTO,
};

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UploadFileParameters<'s> {
    #[serde(rename = "X-Bz-File-Name")]
    file_name: &'s FileName,

    /// content type parameter, if not set "b2/x-auto" will be sent, causing backblaze to determine the right type
    #[serde(
        rename = "Content-Type",
        default = "b2_content_type_default",
        serialize_with = "serialize_content_type_header"
    )]
    #[builder(default=&CONTENT_TYPE_AUTO)]
    content_type: ContentTypeRef<'s>,

    #[serde(rename = "Content-Length")]
    content_length: u64,

    #[serde(rename = "X-Bz-Content-Sha1")]
    content_sha1: Sha1DigestRef<'s>,

    #[serde(rename = "X-Bz-Info-src_last_modified_millis")]
    #[builder(default, setter(strip_option))]
    src_last_modified_millis: Option<TimeStamp>,

    #[builder(default, setter(strip_option))]
    #[serde(
        rename = "X-Bz-Info-b2-content-disposition",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    content_disposition: Option<ContentDispositionRef<'s>>,

    #[builder(default, setter(strip_option, into))]
    #[serde(
        rename = "X-Bz-Info-b2-content-language",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    content_language: Option<ContentLanguageRef<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(
        rename = "X-Bz-Info-b2-expires",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    expires: Option<ExpiresHeaderValueRef<'s>>,

    #[serde(
        rename = "X-Bz-Info-b2-cache-control",
        serialize_with = "serialize_header_option"
    )]
    #[builder(default, setter(strip_option))]
    cache_control: Option<CacheControlHeaderValueRef<'s>>,

    #[serde(skip)] // will be serialized manually
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
}

pub async fn b2_upload_file<'a, T: Into<Body>>(
    uploader_params: &'a mut UploadParameters,
    upload_file_params: &'a UploadFileParameters<'a>,
    file_contents: T,
) -> Result<FileInformation, UploadFileError> {
    let mut request = reqwest::Client::new()
        .post(uploader_params.upload_url().as_str())
        .header(
            "Authorization",
            uploader_params.authorization_token().as_str(),
        )
        .headers_from(upload_file_params)
        .body(file_contents);
    if let Some(sse) = upload_file_params.server_side_encryption {
        request = sse.add_to_request_as_header(request);
    }
    let resp = request.send().await.map_err(UploadFileError::from)?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await.map_err(UploadFileError::from)?)
    } else {
        Err(UploadFileError::from_response(resp).await)
    }
}
