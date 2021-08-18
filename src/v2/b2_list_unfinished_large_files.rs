use std::{convert::TryFrom, num::NonZeroU8};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::v2::JsonErrorObj;

use super::{
    ApiUrl, AuthorizationToken, BucketId, FileId, FileName, InvalidData, ListBucketsError,
    ListFileNamesOk,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaxUnfinishedLargeFileCount(NonZeroU8);

impl TryFrom<u8> for MaxUnfinishedLargeFileCount {
    type Error = InvalidData;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 100 {
            Err(InvalidData::new(format!(
                "At most 100 unfinished large files may be requested at once, but {} were requested",
                value
            )))
        } else {
            let v = NonZeroU8::try_from(value).map_err(|_| {
                InvalidData::new(
                    "Requesting 0 files is not possible, please select a number between 1 and 100"
                        .to_string(),
                )
            })?;
            Ok(Self(v))
        }
    }
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListUnfinishedLargeFilesRequest {
    bucket_id: BucketId,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    name_prefix: Option<FileName>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_file_id: Option<FileId>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_file_count: Option<MaxUnfinishedLargeFileCount>,
}

pub async fn b2_list_unfinished_large_files(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_parameters: &ListUnfinishedLargeFilesRequest,
) -> Result<ListFileNamesOk, ListBucketsError> {
    //TODO: ListBucketsError has the right fields ... but a very bad name in this case ... move and fix naming

    let url = format!(
        "{}/b2api/v2/b2_list_unfinished_large_files",
        api_url.as_str()
    );
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_parameters);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: ListFileNamesOk = resp.json().await?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
