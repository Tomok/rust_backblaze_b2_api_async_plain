use serde::Serialize;
use typed_builder::TypedBuilder;

use super::{
    b2_list_buckets::Bucket, buckets::LifeCycleRule, errors::CreateBucketError, AccountId, ApiUrl,
    AuthorizationToken, BucketInfo, BucketName, BucketType, JsonErrorObj, ServerSideEncryption,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CreateBucketRequest<'s> {
    /// Your account ID.
    account_id: &'s AccountId,
    /// The name to give the new bucket.
    bucket_name: &'s BucketName,
    bucket_type: BucketType, // TODO: use differnt type, that allows all private / all public only

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// User-defined information to be stored with the bucket: a JSON object mapping names to values. See Buckets.
    ///Cache-Control policies can be set here on a global level for all the files in the bucket.
    bucket_info: Option<&'s BucketInfo>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    cors_rules: Option<&'s serde_json::Value>, //TODO...

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, the boolean value specifies whether bucket is File Lock-enabled. The default value is false. Setting the value to true requires the writeBucketRetentions capability.  
    file_lock_enabled: Option<bool>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The initial list of lifecycle rules for this bucket.
    lifecycle_rules: Option<&'s Vec<LifeCycleRule>>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The default server-side encryption settings for this bucket.
    default_server_side_encryption: Option<ServerSideEncryption>,
}

pub async fn b2_create_bucket<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &'a CreateBucketRequest<'a>,
) -> Result<Bucket, CreateBucketError> {
    let url = format!("{}/b2api/v2/b2_create_bucket", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
