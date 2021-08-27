use serde::Serialize;

use super::{
    errors::GetFileInfoError, ApiUrl, AuthorizationToken, FileId, FileInformation, JsonErrorObj,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetFileInfoRequest<'a> {
    file_id: &'a FileId,
}

pub async fn b2_get_file_info(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    file_id: &FileId,
) -> Result<FileInformation, GetFileInfoError> {
    let url = format!("{}/b2api/v2/b2_get_file_info", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&GetFileInfoRequest { file_id })
        .send()
        .await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
