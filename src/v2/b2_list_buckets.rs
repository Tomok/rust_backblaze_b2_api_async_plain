use super::{
    buckets::{
        BucketId, BucketInfo, BucketName, BucketRevision, BucketType, BucketTypes, LifeCycleRule,
    },
    AccountId, ApiUrl, AuthorizationToken, Error, FileLockConfiguration, JsonErrorObj,
};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest<'a> {
    #[builder(setter(into))]
    account_id: &'a AccountId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_id: Option<BucketId>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_name: Option<BucketName>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_types: Option<BucketTypes>,
}

impl<'a> ListBucketsRequest<'a> {
    pub fn new(
        account_id: &'a AccountId,
        bucket_id: Option<BucketId>,
        bucket_name: Option<BucketName>,
        bucket_types: Option<BucketTypes>,
    ) -> Self {
        Self {
            account_id,
            bucket_id,
            bucket_name,
            bucket_types,
        }
    }
}

#[derive(Debug)]
pub enum ListBucketsError {
    BadRequest {
        raw_error: JsonErrorObj,
    },
    Unauthorized {
        raw_error: JsonErrorObj,
    },
    /// not listed in the api in <https://www.backblaze.com/b2/docs/b2_list_buckets.html> but I assume this could happen as well
    TransactionCapExceeded {
        raw_error: JsonErrorObj,
    },
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    account_id: AccountId,
    bucket_id: BucketId,
    bucket_name: BucketName,
    bucket_type: BucketType,
    bucket_info: BucketInfo,
    #[serde(default)]
    cors_rules: serde_json::Value, // it's not part of the example, so maybe optional???                 //todo!!!
    file_lock_configuration: FileLockConfiguration,
    default_server_side_encryption: serde_json::Value, //todo !!!
    lifecycle_rules: Vec<LifeCycleRule>,
    #[serde(default)]
    revision: Option<BucketRevision>, // it's not part of the example, so maybe optional???
    #[serde(default)]
    options: Option<serde_json::Value>, //todo!!!
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsOk {
    buckets: Vec<Bucket>,
}

impl ListBucketsOk {
    /// Get a reference to the buckets
    pub fn buckets(&self) -> &[Bucket] {
        &self.buckets
    }
}

impl From<reqwest::Error> for ListBucketsError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        ListBucketsError::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

impl From<JsonErrorObj> for ListBucketsError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_list_buckets(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_body: &ListBucketsRequest<'_>,
) -> Result<ListBucketsOk, ListBucketsError> {
    let url = format!("{}/b2api/v2/b2_list_buckets", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_body);
    let resp = request.send().await.map_err(ListBucketsError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: ListBucketsOk = resp.json().await.map_err(ListBucketsError::from)?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(ListBucketsError::from)?;
        Err(raw_error.into())
    }
}

#[cfg(test)]
mod test {
    use crate::v2::test::mock_server::*;

    use super::super::{AccountId, ApiUrl, AuthorizationToken};
    use super::{b2_list_buckets, ListBucketsRequest};

    #[tokio::test]
    async fn test_ok() {
        let mock_server = B2MockServer::start().await;
        mock_server.register_default_list_bucket_handler().await;
        let res = b2_list_buckets(
            &ApiUrl(mock_server.uri()),
            &AuthorizationToken(FAKE_AUTHORIZATION_TOKEN.into()),
            &ListBucketsRequest::builder()
                .account_id(&AccountId(FAKE_ACCOUNT_ID.to_string()))
                .build(),
        )
        .await;
        dbg!(&res);
        assert!(res.is_ok());
    }
}
