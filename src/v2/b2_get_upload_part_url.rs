use crate::v2::JsonErrorObj;

use super::{
    b2_get_upload_url::UploadUrl, errors::GetUploadUrlError, ApiUrl, AuthorizationToken, FileId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPartUrlParameters {
    file_id: FileId,
    upload_url: UploadUrl,
    authorization_token: AuthorizationToken,
}

impl UploadPartUrlParameters {
    /// Get a reference to the upload part url parameters's upload url.
    pub fn upload_url(&self) -> &UploadUrl {
        &self.upload_url
    }

    /// Get a reference to the upload part url parameters's authorization token.
    pub fn authorization_token(&self) -> &AuthorizationToken {
        &self.authorization_token
    }

    /// Get a reference to the upload part url parameters's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetUploadPartUrlRequest<'a> {
    file_id: &'a FileId,
}

impl<'a> From<&'a FileId> for GetUploadPartUrlRequest<'a> {
    fn from(file_id: &'a FileId) -> Self {
        Self { file_id }
    }
}

pub async fn b2_get_upload_part_url(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    file_id: &FileId,
) -> Result<UploadPartUrlParameters, GetUploadUrlError> {
    let url = format!("{}/b2api/v2/b2_get_upload_part_url", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&GetUploadPartUrlRequest::from(file_id));
    let resp = request.send().await.map_err(GetUploadUrlError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(GetUploadUrlError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(GetUploadUrlError::from)?;
        Err(raw_error.into())
    }
}
