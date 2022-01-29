#[cfg(feature = "b2_unstable")]
use super::FileInfo;
use super::{
    errors::LargeFileError, serialize_content_type_header, ApiUrl, AuthorizationToken, BucketId,
    ContentTypeRef, FileInformation, FileName, FileRetention, JsonErrorObj, LegalHold,
    ServerSideEncryptionCustomerKey, CONTENT_TYPE_AUTO,
};
use serde::Serialize;
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct StartLargeFileParameters<'s> {
    bucket_id: &'s BucketId,
    file_name: &'s FileName,

    #[builder(default = &CONTENT_TYPE_AUTO)]
    #[serde(serialize_with = "serialize_content_type_header")]
    content_type: ContentTypeRef<'s>,

    #[cfg(feature = "b2_unstable")]
    #[builder(default, setter(strip_option))]
    file_info: Option<&'s FileInfo>, // <- TODO: right type??

    #[builder(default, setter(strip_option))]
    file_retention: Option<&'s FileRetention>,

    #[builder(default, setter(strip_option))]
    legal_hold: Option<&'s LegalHold>,

    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
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
