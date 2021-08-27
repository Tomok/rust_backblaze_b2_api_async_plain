use crate::v2::JsonErrorObj;

use super::{
    errors::ListBucketsError, AccountId, ApiUrl, AuthorizationToken, BucketId, FileId, FileName,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelFileRequest<'s> {
    file_id: &'s FileId,
}

pub async fn b2_cancel_large_file(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    file_id: &FileId,
) -> Result<CancelFileOk, ListBucketsError> {
    //TODO: ListBucketsError has the right fields ... but a very bad name in this case ... move and fix naming

    let url = format!("{}/b2api/v2/b2_cancel_large_file", api_url.as_str());
    let request_data = CancelFileRequest { file_id };
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&request_data);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
