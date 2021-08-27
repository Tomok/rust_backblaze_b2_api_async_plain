use serde::Serialize;

use super::{
    b2_list_buckets::Bucket, errors::ListBucketsError, AccountId, ApiUrl, AuthorizationToken,
    BucketId, JsonErrorObj,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteBucketRequest<'s> {
    account_id: &'s AccountId,
    bucket_id: &'s BucketId,
}

pub async fn b2_delete_bucket(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    account_id: &AccountId,
    bucket_id: &BucketId,
) -> Result<Bucket, ListBucketsError> {
    let url = format!("{}/b2api/v2/b2_delete_bucket", api_url.as_str());
    let delete_bucket_request = DeleteBucketRequest {
        account_id,
        bucket_id,
    };
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&delete_bucket_request);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
