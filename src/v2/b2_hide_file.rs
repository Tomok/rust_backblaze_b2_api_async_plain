use serde::Serialize;

use super::{
    ApiUrl, AuthorizationToken, BucketId, FileInformation, FileName, GetFileInfoError, JsonErrorObj,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HideFileRequest<'a> {
    bucket_id: &'a BucketId,
    file_name: &'a FileName,
}

pub async fn b2_hide_file(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    bucket_id: &BucketId,
    file_name: &FileName,
) -> Result<FileInformation, GetFileInfoError> {
    let url = format!("{}/b2api/v2/b2_hide_file", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&HideFileRequest {
            bucket_id,
            file_name,
        })
        .send()
        .await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
