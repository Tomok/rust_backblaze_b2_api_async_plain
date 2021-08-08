use reqwest::Body;
use serde::Serialize;
use std::str::FromStr;
use typed_builder::TypedBuilder;

use crate::header_serializer::HeadersFrom;

use super::{
    server_side_encryption::EncryptionAlgorithm, CacheControlHeaderValue, ContentDisposition,
    ContentLanguage, Error, ExpiresHeaderValue, FileInformation, FileName, JsonErrorObj, Md5, Mime,
    ServerSideEncryptionCustomerKey, Sha1, TimeStamp, UploadParameters,
};

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UploadFileParameters {
    #[serde(rename = "X-Bz-File-Name")]
    file_name: FileName,

    /// content type parameter, if not set "b2/x-auto" will be sent, causing backblaze to determine the right type
    #[serde(rename = "Content-Type", default = "b2_content_type_default")]
    #[builder(default=Mime::from_str("b2/x-auto").unwrap())]
    content_type: Mime,

    #[serde(rename = "Content-Length")]
    content_length: u64,

    #[serde(rename = "X-Bz-Content-Sha1")]
    content_sha1: Sha1,

    #[serde(rename = "X-Bz-Info-src_last_modified_millis")]
    #[builder(default, setter(strip_option))]
    src_last_modified_millis: Option<TimeStamp>,

    #[serde(rename = "X-Bz-Info-b2-content-disposition")]
    #[builder(default, setter(strip_option))]
    content_disposition: Option<ContentDisposition>,

    #[serde(rename = "X-Bz-Info-b2-content-language")]
    #[builder(default, setter(strip_option))]
    content_language: Option<ContentLanguage>,

    #[serde(rename = "X-Bz-Info-b2-expires")]
    #[builder(default, setter(strip_option))]
    expires: Option<ExpiresHeaderValue>,

    #[serde(rename = "X-Bz-Info-b2-cache-control")]
    #[builder(default, setter(strip_option))]
    cache_control: Option<CacheControlHeaderValue>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Algorithm")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_algorithm: Option<EncryptionAlgorithm>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Key")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_customer_key: Option<ServerSideEncryptionCustomerKey>,

    #[serde(rename = "X-Bz-Server-Side-Encryption-Customer-Key-Md5")]
    #[builder(default, setter(strip_option))]
    server_side_encryption_customer_key_md5: Option<Md5>,
}

#[derive(Debug)]
pub enum UploadFileError {
    BadRequest {
        raw_error: JsonErrorObj,
    },
    Unauthorized {
        raw_error: JsonErrorObj,
    },
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    CapExceeded {
        raw_error: JsonErrorObj,
    },
    // Method Not allowed listed in documentation, but skipped here, as the request method forces POST
    RequestTimeout {
        raw_error: JsonErrorObj,
    },
    /// acc. to documentaion: Call [b2_get_upload_url] again to get a new auth token
    ServiceUnavailable {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

impl From<reqwest::Error> for UploadFileError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for UploadFileError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (403, "cap_exceeded") => Self::CapExceeded { raw_error: e },
            (408, "request_timeout") => Self::RequestTimeout { raw_error: e },
            (503, "service_unavailable") => Self::ServiceUnavailable { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_upload_file<T: Into<Body>>(
    uploader_params: &mut UploadParameters,
    upload_file_params: &UploadFileParameters,
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
