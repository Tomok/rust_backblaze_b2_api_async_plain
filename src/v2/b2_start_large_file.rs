use super::{
    ApiUrl, AuthorizationToken, BucketId, Error, FileInfo, FileInformation, FileName,
    FileRetention, JsonErrorObj, LegalHold, Mime, ServerSideEncryptionCustomerKey,
};

use serde::Serialize;
use std::str::FromStr;
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct StartLargeFileParameters {
    bucket_id: BucketId,
    file_name: FileName,
    #[builder(default=Mime::from_str("b2/x-auto").unwrap())]
    content_type: Mime,
    #[builder(default, setter(strip_option))]
    file_info: Option<FileInfo>, // <- TODO: right type??
    #[builder(default, setter(strip_option))]
    file_retention: Option<FileRetention>,
    #[builder(default, setter(strip_option))]
    legal_hold: Option<LegalHold>,
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<ServerSideEncryptionCustomerKey>, // <- TODO: right type??
}

pub enum StartLargeFileError {
    BadRequest { raw_error: JsonErrorObj },
    BadBucketId { raw_error: JsonErrorObj },
    Unauthorized { raw_error: JsonErrorObj },
    BadAuthToken { raw_error: JsonErrorObj },
    ExpiredAuthToken { raw_error: JsonErrorObj },
    Unexpected { raw_error: Error },
}

impl From<reqwest::Error> for StartLargeFileError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}
impl From<JsonErrorObj> for StartLargeFileError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (400, "bad_bucket_id") => Self::BadBucketId { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_start_large_file(
    api_url: &ApiUrl,
    authorization: &AuthorizationToken,
    params: &StartLargeFileParameters,
) -> Result<FileInformation, StartLargeFileError> {
    let url = format!("{}/b2api/v2/b2_start_large_file", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization.as_str())
        .json(&params)
        .send()
        .await
        .map_err(StartLargeFileError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(StartLargeFileError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(StartLargeFileError::from)?;
        Err(raw_error.into())
    }
}
