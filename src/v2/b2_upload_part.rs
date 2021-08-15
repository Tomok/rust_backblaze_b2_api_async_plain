use reqwest::Body;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::header_serializer::HeadersFrom;

use super::{
    server_side_encryption::EncryptionAlgorithm, Error, FileId, JsonErrorObj, Md5, PartNumber,
    ServerSideEncryption, ServerSideEncryptionCustomerKey, Sha1, TimeStamp,
    UploadPartUrlParameters,
};

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UploadPartParameters {
    #[serde(rename = "X-Bz-Part-Number")]
    part_number: PartNumber,

    #[serde(rename = "Content-Length")]
    content_length: u64,

    #[serde(rename = "X-Bz-Content-Sha1")]
    content_sha1: Sha1,

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
pub enum UploadPartError {
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

impl From<reqwest::Error> for UploadPartError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for UploadPartError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (408, "request_timeout") => Self::RequestTimeout { raw_error: e },
            (503, "service_unavailable") => Self::ServiceUnavailable { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPartOk {
    file_id: FileId,
    part_number: PartNumber,
    content_length: u64,
    content_sha1: Sha1,
    content_md5: Option<Md5>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

pub async fn b2_upload_part<T: Into<Body>>(
    uploader_params: &mut UploadPartUrlParameters,
    upload_part_params: &UploadPartParameters,
    file_contents: T,
) -> Result<UploadPartOk, UploadPartError> {
    let resp = reqwest::Client::new()
        .post(uploader_params.upload_url().as_str())
        .header(
            "Authorization",
            uploader_params.authorization_token().as_str(),
        )
        .headers_from(upload_part_params)
        .body(file_contents)
        .send()
        .await
        .map_err(UploadPartError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(UploadPartError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(UploadPartError::from)?;
        Err(raw_error.into())
    }
}
