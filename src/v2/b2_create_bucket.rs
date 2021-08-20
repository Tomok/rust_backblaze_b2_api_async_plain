use serde::Serialize;
use typed_builder::TypedBuilder;

use super::{
    b2_list_buckets::Bucket, buckets::LifeCycleRule, AccountId, ApiUrl, AuthorizationToken,
    BucketInfo, BucketName, BucketType, Error, JsonErrorObj, ServerSideEncryption,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CreateBucketRequest {
    /// Your account ID.
    account_id: AccountId,
    /// The name to give the new bucket.
    bucket_name: BucketName,
    bucket_type: BucketType, // TODO: use differnt type, that allows all private / all public only

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// User-defined information to be stored with the bucket: a JSON object mapping names to values. See Buckets.
    ///Cache-Control policies can be set here on a global level for all the files in the bucket.
    bucket_info: Option<BucketInfo>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    cors_rules: Option<serde_json::Value>, //TODO...

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, the boolean value specifies whether bucket is File Lock-enabled. The default value is false. Setting the value to true requires the writeBucketRetentions capability.  
    file_lock_enabled: Option<bool>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The initial list of lifecycle rules for this bucket.
    lifecycle_rules: Option<Vec<LifeCycleRule>>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The default server-side encryption settings for this bucket.
    default_server_side_encryption: Option<ServerSideEncryption>,
}

#[derive(Debug)]
pub enum CreateBucketError {
    BadRequest { raw_error: JsonErrorObj },
    TooManyBuckets { raw_error: JsonErrorObj },
    DuplicateBucketName { raw_error: JsonErrorObj },
    Unauthorized { raw_error: JsonErrorObj },
    BadAuthToken { raw_error: JsonErrorObj },
    ExpiredAuthToken { raw_error: JsonErrorObj },
    Unexpected { raw_error: Error },
}

impl From<reqwest::Error> for CreateBucketError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

impl From<JsonErrorObj> for CreateBucketError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (400, "too_many_buckets") => Self::TooManyBuckets { raw_error: e },
            (400, "duplicate_bucket_name") => Self::DuplicateBucketName { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_create_bucket(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &CreateBucketRequest,
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
