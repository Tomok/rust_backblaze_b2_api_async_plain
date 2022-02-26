use std::{convert::TryFrom, num::NonZeroU16};

use super::{
    errors::GenericB2Error, ApiUrl, AuthorizationToken, FileId, InvalidData, Md5Digest, PartNumber,
    ServerSideEncryption, Sha1Digest, TimeStamp,
};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
pub struct ListPartsRequest<'s> {
    file_id: &'s FileId,

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

impl ListPartsOk {
    /// Get a reference to the list parts ok's parts.
    pub fn parts(&self) -> &[Part] {
        self.parts.as_slice()
    }

    /// Get a reference to the list parts ok's next part number.
    pub fn next_part_number(&self) -> Option<&PartNumber> {
        self.next_part_number.as_ref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    file_id: FileId,
    part_number: PartNumber,
    content_length: u64,
    content_sha1: Sha1Digest,
    content_md5: Option<Md5Digest>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

impl Part {
    /// Get a reference to the part's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Get a reference to the part's part number.
    pub fn part_number(&self) -> &PartNumber {
        &self.part_number
    }

    /// Get a reference to the part's content length.
    pub fn content_length(&self) -> &u64 {
        &self.content_length
    }

    /// Get a reference to the part's content sha1.
    pub fn content_sha1(&self) -> &Sha1Digest {
        &self.content_sha1
    }

    /// Get a reference to the part's content md5.
    pub fn content_md5(&self) -> Option<&Md5Digest> {
        self.content_md5.as_ref()
    }

    /// Get a reference to the part's server side encryption.
    pub fn server_side_encryption(&self) -> Option<&ServerSideEncryption> {
        self.server_side_encryption.as_ref()
    }

    /// Get a reference to the part's upload timestamp.
    pub fn upload_timestamp(&self) -> &TimeStamp {
        &self.upload_timestamp
    }
}

pub async fn b2_list_parts(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_parameters: &ListPartsRequest<'_>,
) -> Result<ListPartsOk, GenericB2Error> {
    let url = format!("{}/b2api/v2/b2_list_parts", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_parameters);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        let ok: ListPartsOk = resp.json().await?;
        Ok(ok)
    } else {
        Err(GenericB2Error::from_response(resp).await)
    }
}
