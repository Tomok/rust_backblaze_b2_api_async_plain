use super::{
    errors::StartLargeFileError, ApiUrl, AuthorizationToken, BucketId, FileInfo, FileInformation,
    FileName, FileRetention, JsonErrorObj, LegalHold, Mime, ServerSideEncryptionCustomerKey,
};

use serde::Serialize;
use std::str::FromStr;
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct StartLargeFileParameters {
    bucket_id: BucketId,
    file_name: FileName,
    #[builder(default=Mime::from_str("b2/x-auto").unwrap())]
    content_type: Mime,
    #[builder(default, setter(strip_option))]
    file_info: Option<FileInfo>, // <- TODO: right type??
    #[builder(default, setter(strip_option))]
    file_retention: Option<FileRetention>,
    #[builder(default, setter(strip_option))]
    legal_hold: Option<LegalHold>,
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<ServerSideEncryptionCustomerKey>, // <- TODO: right type??
}

pub async fn b2_start_large_file(
    api_url: &ApiUrl,
    authorization: &AuthorizationToken,
    params: &StartLargeFileParameters,
) -> Result<FileInformation, StartLargeFileError> {
    let url = format!("{}/b2api/v2/b2_start_large_file", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization.as_str())
        .json(&params)
        .send()
        .await
        .map_err(StartLargeFileError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await.map_err(StartLargeFileError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(StartLargeFileError::from)?;
        Err(raw_error.into())
    }
}
