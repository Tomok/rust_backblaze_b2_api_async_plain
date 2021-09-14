use serde::{Deserialize, Serialize};

use super::{
    errors::GenericB2Error, AccountId, ApiUrl, ApplicationKeyId, ApplicationKeyIdRef,
    AuthorizationToken, BucketId, Capabilities, FileName, JsonErrorObj, KeyName, TimeStamp,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteKeyRequest<'s> {
    application_key_id: ApplicationKeyIdRef<'s>,
}

//TODO: used by list_keys as well, maybe move?
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

impl KeyInformation {
    /// Get a reference to the key information's key name.
    pub fn key_name(&self) -> &KeyName {
        &self.key_name
    }

    /// Get a reference to the key information's application key id.
    pub fn application_key_id(&self) -> super::ApplicationKeyIdRef {
        &self.application_key_id
    }

    /// Get a reference to the key information's capabilities.
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Get a reference to the key information's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the key information's expiration timestamp.
    pub fn expiration_timestamp(&self) -> Option<&TimeStamp> {
        self.expiration_timestamp.as_ref()
    }

    /// Get a reference to the key information's bucket id.
    pub fn bucket_id(&self) -> Option<&BucketId> {
        self.bucket_id.as_ref()
    }

    /// Get a reference to the key information's name prefix.
    pub fn name_prefix(&self) -> Option<&FileName> {
        self.name_prefix.as_ref()
    }

    /// Get a reference to the key information's options.
    pub fn options(&self) -> &serde_json::Value {
        &self.options
    }
}

pub async fn b2_delete_key(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    application_key_id: ApplicationKeyIdRef<'_>,
) -> Result<KeyInformation, GenericB2Error> {
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
