use super::{
    errors::GenericB2Error, AccountId, ApiUrl, AuthorizationToken, BucketId, FileId, FileName,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelFileOk {
    file_id: FileId,
    account_id: AccountId,
    bucket_id: BucketId,
    file_name: FileName,
}

impl CancelFileOk {
    /// Get a reference to the cancel file ok's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Get a reference to the cancel file ok's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the cancel file ok's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }

    /// Get a reference to the cancel file ok's file name.
    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelFileRequest<'s> {
    file_id: &'s FileId,
}

pub async fn b2_cancel_large_file(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    file_id: &FileId,
) -> Result<CancelFileOk, GenericB2Error> {
    let url = format!("{}/b2api/v2/b2_cancel_large_file", api_url.as_str());
    let request_data = CancelFileRequest { file_id };
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&request_data);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(GenericB2Error::from_response(resp).await)
    }
}
