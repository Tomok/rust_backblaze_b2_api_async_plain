use super::{
    errors::LargeFileError, ApiUrl, AuthorizationToken, BucketId, FileInfo, FileInformation,
    FileName, FileRetention, JsonErrorObj, LegalHold, Mime, ServerSideEncryptionCustomerKey,
};

use serde::Serialize;
use std::str::FromStr;
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct StartLargeFileParameters<'s> {
    bucket_id: &'s BucketId,
    file_name: &'s FileName,
    #[builder(default=Mime::from_str("b2/x-auto").unwrap())]
    content_type: Mime,
    #[builder(default, setter(strip_option))]
    file_info: Option<&'s FileInfo>, // <- TODO: right type??
    #[builder(default, setter(strip_option))]
    file_retention: Option<&'s FileRetention>,
    #[builder(default, setter(strip_option))]
    legal_hold: Option<&'s LegalHold>,
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey>, // <- TODO: right type??
}

pub async fn b2_start_large_file<'a>(
    api_url: &'a ApiUrl,
    authorization: &'a AuthorizationToken,
    params: &'a StartLargeFileParameters<'a>,
) -> Result<FileInformation, LargeFileError> {
    let url = format!("{}/b2api/v2/b2_start_large_file", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization.as_str())
        .json(&params)
        .send()
        .await
        .map_err(LargeFileError::from)?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await.map_err(LargeFileError::from)?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(LargeFileError::from)?;
        Err(raw_error.into())
    }
}
