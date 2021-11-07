use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::v2::JsonErrorObj;

use super::{
    errors::ListFileVersionsError, ApiUrl, AuthorizationToken, BucketId, FileId, FileInformation,
    FileName, MaxFileCount,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListFileVersionsRequest<'s> {
    bucket_id: &'s BucketId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_file_name: Option<&'s FileName>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_file_id: Option<&'s FileId>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_file_count: Option<MaxFileCount>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<&'s str>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    delimiter: Option<&'s str>,
}

impl<'s> ListFileVersionsRequest<'s> {
    pub fn new(
        bucket_id: &'s BucketId,
        start_file_name: Option<&'s FileName>,
        start_file_id: Option<&'s FileId>,
        max_file_count: Option<MaxFileCount>,
        prefix: Option<&'s str>,
        delimiter: Option<&'s str>,
    ) -> Self {
        Self {
            bucket_id,
            start_file_name,
            start_file_id,
            max_file_count,
            prefix,
            delimiter,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFileVersionsOk {
    files: Vec<FileInformation>,
    next_file_name: Option<FileName>,
    next_file_id: Option<FileId>,
}

impl ListFileVersionsOk {
    /// Get a reference to the list file versions ok's files.
    pub fn files(&self) -> &[FileInformation] {
        self.files.as_slice()
    }

    /// Get a reference to the list file versions ok's next file name.
    pub fn next_file_name(&self) -> Option<&FileName> {
        self.next_file_name.as_ref()
    }

    /// Get a reference to the list file versions ok's next file id.
    pub fn next_file_id(&self) -> Option<&FileId> {
        self.next_file_id.as_ref()
    }
}

pub async fn b2_list_file_versions<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &AuthorizationToken,
    request_body: &'a ListFileVersionsRequest<'a>,
) -> Result<ListFileVersionsOk, ListFileVersionsError> {
    let url = format!("{}/b2api/v2/b2_list_file_versions", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_body);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        let auth_ok = resp.json().await?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
