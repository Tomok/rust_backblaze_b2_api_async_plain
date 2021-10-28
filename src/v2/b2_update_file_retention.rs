use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors::UpdateFileLockError, ApiUrl, AuthorizationToken, FileId, FileName, FileRetention,
    JsonErrorObj,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileRetentionRequest<'s> {
    file_name: &'s FileName,
    file_id: &'s FileId,
    file_retention: &'s FileRetention,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bypass_governance: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileRetentionOk {
    file_id: FileId,
    file_name: FileName,
    file_retention: FileRetention,
}

impl UpdateFileRetentionOk {
    /// Get a reference to the update file retention ok's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Get a reference to the update file retention ok's file name.
    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }

    /// Get a reference to the update file retention ok's file retention.
    pub fn file_retention(&self) -> &FileRetention {
        &self.file_retention
    }
}

pub async fn b2_update_file_retention(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &UpdateFileRetentionRequest<'_>,
) -> Result<UpdateFileRetentionOk, UpdateFileLockError> {
    let url = format!("{}/b2api/v2/b2_update_file_retention", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
