use reqwest::Body;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::header_serializer::HeadersFrom;

use super::{
    errors::UploadFileError, server_side_encryption::EncryptionAlgorithm,
    CacheControlHeaderValueRef, ContentDispositionRef, ContentLanguageRef, ExpiresHeaderValueRef,
    FileInformation, FileName, JsonErrorObj, Md5Ref, Mime, ServerSideEncryptionCustomerKey,
    Sha1Ref, TimeStamp, UploadParameters,
};

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UploadFileParameters<'s> {
    #[serde(rename = "X-Bz-File-Name")]
    file_name: &'s FileName,

    /// content type parameter, if not set "b2/x-auto" will be sent, causing backblaze to determine the right type
    #[serde(rename = "Content-Type", default = "b2_content_type_default")]
    #[builder(default=Mime::auto())]
    content_type: &'s Mime,

    #[serde(rename = "Content-Length")]
    content_length: u64,

    #[serde(rename = "X-Bz-Content-Sha1")]
    content_sha1: Sha1Ref<'s>,

    #[serde(rename = "X-Bz-Info-src_last_modified_millis")]
    #[builder(default, setter(strip_option))]
    src_last_modified_millis: Option<TimeStamp>,

    #[serde(rename = "X-Bz-Info-b2-content-disposition")]
    #[builder(default, setter(strip_option))]
    content_disposition: Option<ContentDispositionRef<'s>>,

    #[serde(rename = "X-Bz-Info-b2-content-language")]
    #[builder(default, setter(strip_option))]
    content_language: Option<ContentLanguageRef<'s>>,

    #[serde(rename = "X-Bz-Info-b2-expires")]
    #[builder(default, setter(strip_option))]
    expires: Option<ExpiresHeaderValueRef<'s>>,

    #[serde(rename = "X-Bz-Info-b2-cache-control")]
    #[builder(default, setter(strip_option))]
    cache_control: Option<CacheControlHeaderValueRef<'s>>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Algorithm")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_algorithm: Option<EncryptionAlgorithm>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Key")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_customer_key: Option<&'s ServerSideEncryptionCustomerKey>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Key-Md5")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_customer_key_md5: Option<Md5Ref<'s>>,
}

pub async fn b2_upload_file<'a, T: Into<Body>>(
    uploader_params: &'a mut UploadParameters,
    upload_file_params: &'a UploadFileParameters<'a>,
    file_contents: T,
) -> Result<FileInformation, UploadFileError> {
    let resp = reqwest::Client::new()
        .post(uploader_params.upload_url().as_str())
        .header(
            "Authorization",
            uploader_params.authorization_token().as_str(),
        )
        .headers_from(upload_file_params)
        .body(file_contents)
        .send()
        .await
        .map_err(UploadFileError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(UploadFileError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(UploadFileError::from)?;
        Err(raw_error.into())
    }
}
