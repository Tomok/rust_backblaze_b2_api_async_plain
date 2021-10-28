use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors::GenericB2Error, AccountId, ApiUrl, ApplicationKey, ApplicationKeyId,
    ApplicationKeyIdRef, AuthorizationToken, BucketId, Capabilities, FileName, JsonErrorObj,
    KeyName, KeyNameRef, TimeStamp,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyRequest<'s> {
    account_id: &'s AccountId,
    capabilities: &'s Capabilities,
    /// A name for this key. There is no requirement that the name be unique. The name cannot be used to look up the key. Names can contain letters, numbers, and "-", and are limited to 100 characters.
    key_name: KeyNameRef<'s>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// When provided, the key will expire after the given number of seconds, and will have expirationTimestamp set. Value must be a positive integer, and must be less than 1000 days (in seconds).
    valid_duration_in_seconds: Option<u64>, //todo

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// When present, the new key can only access this bucket. When set, only these capabilities can be specified: listAllBucketNames, listBuckets, readBuckets, readBucketEncryption, writeBucketEncryption, readBucketRetentions, writeBucketRetentions, listFiles, readFiles, shareFiles, writeFiles, deleteFiles, readFileLegalHolds, writeFileLegalHolds, readFileRetentions, writeFileRetentions, and bypassGovernance.
    bucket_id: Option<&'s BucketId>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    ///When present, restricts access to files whose names start with the prefix. You must set bucketId when setting this.
    name_prefix: Option<&'s FileName>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedKeyInformation {
    /// The name assigned when the key was created.
    key_name: KeyName,

    ///The ID of the newly created key.
    application_key_id: ApplicationKeyId,

    ///The secret part of the key. This is the only time it will be returned, so you need to keep it. This is not returned when you list the keys in your account.
    application_key: ApplicationKey,

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

impl CreatedKeyInformation {
    /// Get a reference to the created key information's key name.
    pub fn key_name(&self) -> &KeyName {
        &self.key_name
    }

    /// Get a reference to the created key information's application key id.
    pub fn application_key_id(&self) -> ApplicationKeyIdRef {
        &self.application_key_id
    }

    /// Get a reference to the created key information's application key.
    pub fn application_key(&self) -> &ApplicationKey {
        &self.application_key
    }

    /// Get a reference to the created key information's capabilities.
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Get a reference to the created key information's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the created key information's expiration timestamp.
    pub fn expiration_timestamp(&self) -> Option<&TimeStamp> {
        self.expiration_timestamp.as_ref()
    }

    /// Get a reference to the created key information's bucket id.
    pub fn bucket_id(&self) -> Option<&BucketId> {
        self.bucket_id.as_ref()
    }

    /// Get a reference to the created key information's name prefix.
    pub fn name_prefix(&self) -> Option<&FileName> {
        self.name_prefix.as_ref()
    }
}

pub async fn b2_create_key<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &'a AuthorizationToken,
    request: &'a CreateKeyRequest<'a>,
) -> Result<CreatedKeyInformation, GenericB2Error> {
    let url = format!("{}/b2api/v2/b2_create_key", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
