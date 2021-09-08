use crate::v2::JsonErrorObj;

use super::{errors::LargeFileError, ApiUrl, AuthorizationToken, FileId, FileInformation, Sha1Ref};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FinishLargeFileRequest<'shas, 'file_id> {
    file_id: &'file_id FileId,
    part_sha1_array: &'shas [Sha1Ref<'shas>],
}

pub async fn b2_finish_large_file<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &'a AuthorizationToken,
    file_id: &'a FileId,
    part_sha1s: &'a [Sha1Ref<'a>],
) -> Result<FileInformation, LargeFileError> {
    let request_data = FinishLargeFileRequest {
        file_id,
        part_sha1_array: part_sha1s,
    };
    let url = format!("{}/b2api/v2/b2_finish_large_file", api_url.as_str());

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
