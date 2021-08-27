use serde::{Deserialize, Serialize};

use super::{
    errors::ListBucketsError, AccountId, ApiUrl, ApplicationKeyId, AuthorizationToken, BucketId,
    Capabilities, FileName, JsonErrorObj, KeyName, TimeStamp,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteKeyRequest<'s> {
    application_key_id: &'s ApplicationKeyId,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInformation {
    /// The name assigned when the key was created.
    key_name: KeyName,

    ///The ID of the key.
    application_key_id: ApplicationKeyId,

    capabilities: Capabilities,

    ///The account that this application key is for.
    account_id: AccountId,

    expiration_timestamp: Option<TimeStamp>,

    /// When present, restricts access to one bucket.
    bucket_id: Option<BucketId>,

    ///When present, restricts access to files whose names start with the prefix.
    name_prefix: Option<FileName>,

    /// reserved by blackblaze for future use,
    options: serde_json::Value,
}

pub async fn b2_delete_key(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    application_key_id: &ApplicationKeyId,
) -> Result<KeyInformation, ListBucketsError> {
    let request_body = DeleteKeyRequest { application_key_id };
    let url = format!("{}/b2api/v2/b2_delete_key", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&request_body)
        .send()
        .await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
