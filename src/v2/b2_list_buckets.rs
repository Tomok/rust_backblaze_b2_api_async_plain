use super::{
    buckets::{
        BucketId, BucketInfo, BucketName, BucketRevision, BucketType, BucketTypes, LifeCycleRule,
    },
    errors::GenericB2Error,
    AccountId, ApiUrl, AuthorizationToken, FileLockConfiguration,
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
    bucket_id: Option<&'a BucketId>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_name: Option<&'a BucketName>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_types: Option<&'a BucketTypes>,
}

impl<'a> ListBucketsRequest<'a> {
    pub fn new(
        account_id: &'a AccountId,
        bucket_id: Option<&'a BucketId>,
        bucket_name: Option<&'a BucketName>,
        bucket_types: Option<&'a BucketTypes>,
    ) -> Self {
        Self {
            account_id,
            bucket_id,
            bucket_name,
            bucket_types,
        }
    }
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

impl Bucket {
    /// Get a reference to the bucket's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the bucket's bucket id.
    pub fn bucket_id(&self) -> &BucketId {
        &self.bucket_id
    }

    /// Get a reference to the bucket's bucket name.
    pub fn bucket_name(&self) -> &BucketName {
        &self.bucket_name
    }

    /// Get a reference to the bucket's bucket type.
    pub fn bucket_type(&self) -> &BucketType {
        &self.bucket_type
    }

    /// Get a reference to the bucket's bucket info.
    pub fn bucket_info(&self) -> &BucketInfo {
        &self.bucket_info
    }

    /// Get a reference to the bucket's file lock configuration.
    pub fn file_lock_configuration(&self) -> &FileLockConfiguration {
        &self.file_lock_configuration
    }

    /// Get a reference to the bucket's lifecycle rules.
    pub fn lifecycle_rules(&self) -> &[LifeCycleRule] {
        self.lifecycle_rules.as_slice()
    }

    /// Get a reference to the bucket's revision.
    pub fn revision(&self) -> Option<&BucketRevision> {
        self.revision.as_ref()
    }
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

pub async fn b2_list_buckets(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request_body: &ListBucketsRequest<'_>,
) -> Result<ListBucketsOk, GenericB2Error> {
    let url = format!("{}/b2api/v2/b2_list_buckets", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request_body);
    let resp = request.send().await.map_err(GenericB2Error::from)?;
    if resp.status() == http::StatusCode::OK {
        let auth_ok: ListBucketsOk = resp.json().await.map_err(GenericB2Error::from)?;
        Ok(auth_ok)
    } else {
        Err(GenericB2Error::from_response(resp).await)
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
        assert!(res.is_ok());
    }
}
