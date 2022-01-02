use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors::UpdateFileLockError, ApiUrl, AuthorizationToken, FileId, FileName, JsonErrorObj,
    LegalHoldOnOff,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileLegalHoldRequest<'s> {
    file_name: &'s FileName,
    file_id: &'s FileId,
    legal_hold: LegalHoldOnOff,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileLegalHoldOk {
    file_name: FileName,
    file_id: FileId,
    legal_hold: LegalHoldOnOff,
}

impl UpdateFileLegalHoldOk {
    /// Get a reference to the update file legal hold ok's file name.
    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }

    /// Get a reference to the update file legal hold ok's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Get a reference to the update file legal hold ok's legal hold.
    pub fn legal_hold(&self) -> &LegalHoldOnOff {
        &self.legal_hold
    }
}

pub async fn b2_update_file_legal_hold(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &UpdateFileLegalHoldRequest<'_>,
) -> Result<UpdateFileLegalHoldOk, UpdateFileLockError> {
    let url = format!("{}/b2api/v2/b2_update_file_legal_hold", api_url.as_str());
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
