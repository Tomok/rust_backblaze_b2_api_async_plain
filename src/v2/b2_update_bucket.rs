use serde::Serialize;
use typed_builder::TypedBuilder;

use super::{
    b2_list_buckets::Bucket,
    buckets::{BucketRevision, LifeCycleRule},
    errors::UpdateBucketError,
    AccountId, ApiUrl, AuthorizationToken, BucketId, BucketInfo, BucketType, DefaultFileRetention,
    ServerSideEncryption,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBucketRequest<'s> {
    /// Your account ID.
    account_id: &'s AccountId,
    /// The name to give the new bucket.
    bucket_id: &'s BucketId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_type: Option<&'s BucketType>, // TODO: use differnt type, that allows all private / all public only

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// User-defined information to be stored with the bucket: a JSON object mapping names to values. See Buckets.
    ///Cache-Control policies can be set here on a global level for all the files in the bucket.
    bucket_info: Option<&'s BucketInfo>,

    #[cfg(feature = "b2_unstable")]
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    cors_rules: Option<&'s serde_json::Value>, //TODO...

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The default File Lock retention settings for this bucket.
    ///
    ///If specified, the existing default bucket retention settings will be replaced with the new settings. If not specified, setting will remain unchanged. Setting the value requires the writeBucketRetentions capability and that the bucket is File Lock-enabled.  
    default_retention: Option<&'s DefaultFileRetention>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The default server-side encryption settings for this bucket.
    default_server_side_encryption: Option<&'s ServerSideEncryption>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The initial list of lifecycle rules for this bucket.
    lifecycle_rules: Option<&'s [LifeCycleRule]>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// When set, the update will only happen if the revision number stored in the B2 service matches the one passed in. This can be used to avoid having simultaneous updates make conflicting changes.
    if_revision_is: Option<&'s BucketRevision>,
}

pub async fn b2_update_bucket(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &UpdateBucketRequest<'_>,
) -> Result<Bucket, UpdateBucketError> {
    let url = format!("{}/b2api/v2/b2_update_bucket", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(UpdateBucketError::from_response(resp).await)
    }
}
