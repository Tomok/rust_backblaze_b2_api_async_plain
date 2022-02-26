use serde::Serialize;

use super::{
    b2_list_buckets::Bucket, errors::GenericB2Error, AccountId, ApiUrl, AuthorizationToken,
    BucketId,
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
) -> Result<Bucket, GenericB2Error> {
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
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(GenericB2Error::from_response(resp).await)
    }
}
