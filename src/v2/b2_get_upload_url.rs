use serde::{Deserialize, Serialize};

use super::{errors::GetUploadUrlError, ApiUrl, AuthorizationToken, BucketId, JsonErrorObj};

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

//TODO: since this is used by [crate::v2::b2_get_upload_part_url] as well, move to somewhere else?
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
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await.map_err(GetUploadUrlError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(GetUploadUrlError::from)?;
        Err(raw_error.into())
    }
}
