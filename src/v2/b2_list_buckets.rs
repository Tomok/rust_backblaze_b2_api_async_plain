use super::{
    buckets::{BucketInfo, BucketName, BucketRevision, BucketType, BucketTypes},
    AccountId, ApiUrl, AuthorizationToken, Error, JsonErrorObj, MachineReadableJsonErrorObj,
};
use http_types::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest<'a> {
    #[builder(setter(into))]
    account_id: &'a AccountId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_id: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_name: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_types: Option<BucketTypes>,
}

#[derive(Debug)]
pub enum ListBucketsError {
    BadRequest {
        raw_error: JsonErrorObj,
    },
    Unauthorized {
        raw_error: JsonErrorObj,
    },
    /// not listed in the api in [https://www.backblaze.com/b2/docs/b2_list_buckets.html] but I assume this could happen as well
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
struct Bucket {
    account_id: AccountId,
    bucket_id: String,
    bucket_name: BucketName,
    bucket_type: BucketType,
    bucket_info: BucketInfo,
    #[serde(default)]
    cors_rules: serde_json::Value,    // it's not part of the example, so maybe optional???                 //todo!!!
    default_server_side_encryption: serde_json::Value, //todo !!!
    lifecycle_rules: serde_json::Value,
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

impl From<reqwest::Error> for ListBucketsError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        ListBucketsError::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

pub async fn b2_list_buckets<'a>(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_body: &ListBucketsRequest<'a>,
) -> Result<ListBucketsOk, ListBucketsError> {
    let url = format!("{}/b2api/v2/b2_list_buckets", api_url.as_str());
    let resp = reqwest::Client::new()
        .get(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_body)
        .send()
        .await
        .map_err(|e| ListBucketsError::from(e))?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: ListBucketsOk = resp.json().await.map_err(|e| ListBucketsError::from(e))?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(|e| ListBucketsError::from(e))?;
        let err = match raw_error.machine_readable() {
            MachineReadableJsonErrorObj {
                status: StatusCode::BadRequest,
                code: "bad_request",
            } => ListBucketsError::BadRequest { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Unauthorized,
                code: "unauthorized",
            } => ListBucketsError::Unauthorized { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Unauthorized,
                code: "bad_auth_token",
            } => ListBucketsError::BadAuthToken { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Unauthorized,
                code: "expired_auth_token",
            } => ListBucketsError::ExpiredAuthToken { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Forbidden,
                code: "transaction_cap_exceeded",
            } => ListBucketsError::TransactionCapExceeded { raw_error },
            _ => ListBucketsError::Unexpected {
                raw_error: Error::JSONError(raw_error),
            },
        };
        Err(err)
    }
}

#[cfg(test)]
mod test {
    use crate::v2::test::mock_server::*;

    use super::{b2_list_buckets, ListBucketsRequest};
    use super::super::{ApiUrl, AuthorizationToken, AccountId};

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
        ).await;
        dbg!(&res);
        assert!(res.is_ok());
    }
}
