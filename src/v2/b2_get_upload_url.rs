use serde::{Deserialize, Serialize};

use super::{ApiUrl, AuthorizationToken, BucketId, Error, JsonErrorObj};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetUploadUrlRequest<'s> {
    bucket_id: &'s BucketId,
}

impl<'s> From<&'s BucketId> for GetUploadUrlRequest<'s> {
    fn from(bucket_id: &'s BucketId) -> Self {
        Self { bucket_id }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadUrl(String);

impl UploadUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadParameters {
    bucket_id: BucketId,
    upload_url: UploadUrl,
    authorization_token: AuthorizationToken,
}

impl UploadParameters {
    /// Get a reference to the upload parameters's upload url.
    pub fn upload_url(&self) -> &UploadUrl {
        &self.upload_url
    }

    /// Get a reference to the upload parameters's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }

    /// Get a reference to the upload parameters's authorization token.
    pub fn authorization_token(&self) -> &AuthorizationToken {
        &self.authorization_token
    }
}

#[derive(Debug)]
pub enum GetUploadUrlError {
    // TODO: update acc. to documentation
    BadRequest { raw_error: JsonErrorObj },
    Unauthorized { raw_error: JsonErrorObj },
    BadAuthToken { raw_error: JsonErrorObj },
    ExpiredAuthToken { raw_error: JsonErrorObj },
    StorageCapExceeded { raw_error: JsonErrorObj },
    ServiceUnavaliabe { raw_error: JsonErrorObj },
    Unexpected { raw_error: Error },
}

impl From<reqwest::Error> for GetUploadUrlError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for GetUploadUrlError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (403, "storage_cap_exceeded") => Self::StorageCapExceeded { raw_error: e },
            (503, "service_unavailable") => Self::ServiceUnavaliabe { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_get_upload_url(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    bucket_id: &BucketId,
) -> Result<UploadParameters, GetUploadUrlError> {
    let url = format!("{}/b2api/v2/b2_get_upload_url", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&GetUploadUrlRequest::from(bucket_id));
    let resp = request.send().await.map_err(GetUploadUrlError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(GetUploadUrlError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(GetUploadUrlError::from)?;
        Err(raw_error.into())
    }
}
