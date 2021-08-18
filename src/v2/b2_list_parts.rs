use std::{convert::TryFrom, num::NonZeroU16};

use super::{
    ApiUrl, AuthorizationToken, FileId, InvalidData, JsonErrorObj, ListBucketsError, Md5,
    PartNumber, ServerSideEncryption, Sha1, TimeStamp,
};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaxPartCount(NonZeroU16);

impl TryFrom<u16> for MaxPartCount {
    type Error = InvalidData;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 1000 {
            Err(InvalidData::new(format!(
                "At most 1000 parts may be requested at once, but {} were requested",
                value
            )))
        } else {
            let v = NonZeroU16::try_from(value).map_err(|_| {
                InvalidData::new(
                    "Requesting 0 parts is not possible, please select a number between 1 and 1000"
                        .to_string(),
                )
            })?;
            Ok(Self(v))
        }
    }
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListPartsRequest {
    file_id: FileId,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_part_number: Option<PartNumber>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_part_count: Option<MaxPartCount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPartsOk {
    parts: Vec<Part>,
    next_part_number: Option<PartNumber>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    file_id: FileId,
    part_number: PartNumber,
    content_length: u64,
    content_sha1: Sha1,
    content_md5: Option<Md5>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

pub async fn b2_list_parts(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_parameters: &ListPartsRequest,
) -> Result<ListPartsOk, ListBucketsError> {
    //TODO: ListBucketsError has the right fields ... but a very bad name in this case ... move and fix naming

    let url = format!("{}/b2api/v2/b2_list_parts", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_parameters);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let ok: ListPartsOk = resp.json().await?;
        Ok(ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
