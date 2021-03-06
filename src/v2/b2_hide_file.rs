use serde::Serialize;

use super::{
    errors::GetFileInfoError, ApiUrl, AuthorizationToken, BucketId, FileInformation, FileName,
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
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(GetFileInfoError::from_response(resp).await)
    }
}
