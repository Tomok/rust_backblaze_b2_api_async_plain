use std::{convert::TryFrom, num::NonZeroU16};

use super::{
    file::FileInformation, ApiUrl, AuthorizationToken, BucketId, Error, FileName, InvalidData,
    JsonErrorObj,
};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxFileCount(NonZeroU16);

impl TryFrom<u16> for MaxFileCount {
    type Error = InvalidData;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 10000 {
            Err(InvalidData::new(format!(
                "At most 10000 files may be requested at once, but {} were requested",
                value
            )))
        } else {
            // if value was set to 0 the default value of 100 is used acc. to the documentation, so we might as well set it here...
            let v = NonZeroU16::try_from(value).unwrap_or_else(|_| NonZeroU16::new(100).unwrap()); //this unwrap is save, as 100 != 0
            Ok(Self(v))
        }
    }
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListFileNamesRequest {
    bucket_id: BucketId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_file_name: Option<FileName>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_file_count: Option<MaxFileCount>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    delimiter: Option<String>,
}

impl ListFileNamesRequest {
    pub fn new(
        bucket_id: BucketId,
        start_file_name: Option<FileName>,
        max_file_count: Option<MaxFileCount>,
        prefix: Option<String>,
        delimiter: Option<String>,
    ) -> Self {
        Self {
            bucket_id,
            start_file_name,
            max_file_count,
            prefix,
            delimiter,
        }
    }

    /// Set the list file names request's start file name.
    pub fn set_start_file_name(&mut self, start_file_name: Option<FileName>) {
        self.start_file_name = start_file_name;
    }

    /// Set the list file names request's max file count.
    pub fn set_max_file_count(&mut self, max_file_count: Option<MaxFileCount>) {
        self.max_file_count = max_file_count;
    }

    /// Set the list file names request's prefix.
    pub fn set_prefix(&mut self, prefix: Option<String>) {
        self.prefix = prefix;
    }

    /// Set the list file names request's delimiter.
    pub fn set_delimiter(&mut self, delimiter: Option<String>) {
        self.delimiter = delimiter;
    }

    /// Get a reference to the list file names request's start file name.
    pub fn start_file_name(&self) -> &Option<FileName> {
        &self.start_file_name
    }

    /// Get a reference to the list file names request's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }
}

/// Error returned by b2_list_file_names
///
/// based on (official documentation for b2_list_file_names)[https://www.backblaze.com/b2/docs/b2_list_file_names.html]
/// intentionally does not include out_of_range, as this should be prevented by [MaxFileCount], if it is received [ListFileNamesError::Unexpected] will be used
#[derive(Debug)]
pub enum ListFileNamesError {
    /// The request had the wrong fields or illegal values. The message returned with the error will describe the problem.
    BadRequest {
        raw_error: JsonErrorObj,
    },
    InvalidBucketId {
        raw_error: JsonErrorObj,
    },

    Unauthorized {
        raw_error: JsonErrorObj,
    },
    /// not listed in the api in (official documentation)[https://www.backblaze.com/b2/docs/b2_list_file_names.html] but I assume this could happen as well
    TransactionCapExceeded {
        raw_error: JsonErrorObj,
    },
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    /// Timed out while iterating and skipping files
    BadRequestTimeout {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

impl From<JsonErrorObj> for ListFileNamesError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (400, "invalid_bucket_id") => Self::InvalidBucketId { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (403, "transaction_cap_exceeded") => Self::TransactionCapExceeded { raw_error: e },
            (503, "bad_request") => Self::BadRequestTimeout { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}
impl From<reqwest::Error> for ListFileNamesError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFileNamesOk {
    files: Vec<FileInformation>,
    next_file_name: Option<FileName>,
}

impl ListFileNamesOk {
    /// Get a reference to the list file names ok's next file name.
    pub fn next_file_name(&self) -> &Option<FileName> {
        &self.next_file_name
    }

    /// Get a reference to the list file names ok's files.
    pub fn files(&self) -> &Vec<FileInformation> {
        &self.files
    }
}

pub async fn b2_list_file_names(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_body: &ListFileNamesRequest,
) -> Result<ListFileNamesOk, ListFileNamesError> {
    let url = format!("{}/b2api/v2/b2_list_file_names", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_body);
    let resp = request.send().await.map_err(ListFileNamesError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: ListFileNamesOk = resp.json().await.map_err(ListFileNamesError::from)?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(ListFileNamesError::from)?;
        Err(raw_error.into())
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use crate::v2::test::mock_server::*;

    use super::{b2_list_file_names, ListFileNamesRequest};
    use super::{ApiUrl, AuthorizationToken};

    #[tokio::test]
    async fn test_ok() {
        let mock_server = B2MockServer::start().await;
        mock_server.register_default_list_file_names_handler().await;
        let res = b2_list_file_names(
            &ApiUrl(mock_server.uri()),
            &AuthorizationToken(FAKE_AUTHORIZATION_TOKEN.into()),
            &ListFileNamesRequest::builder()
                .bucket_id(FAKE_BUCKET_ID.to_owned().try_into().unwrap())
                .build(),
        )
        .await;
        dbg!(&res);
        assert!(res.is_ok());
    }
}