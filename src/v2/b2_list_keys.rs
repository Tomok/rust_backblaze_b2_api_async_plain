use std::num::NonZeroU16;

use http_types::cookies::Key;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors::GenericB2Error, AccountId, ApiUrl, ApplicationKeyId, AuthorizationToken, JsonErrorObj,
    KeyInformation,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ListKeysRequest<'s> {
    account_id: &'s AccountId,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_key_count: Option<NonZeroU16>, //using NonZeroU16, acc. to the documentation there is a limit of 1000, but it is only relevant for billing

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start_application_key_id: Option<&'s ApplicationKeyId>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListKeysOk {
    keys: Vec<KeyInformation>,

    next_application_key_id: Option<ApplicationKeyId>,
}

impl ListKeysOk {
    /// Get a reference to the list keys ok's next application key id.
    pub fn next_application_key_id(&self) -> Option<&ApplicationKeyId> {
        self.next_application_key_id.as_ref()
    }

    /// Get a reference to the list keys ok's keys.
    pub fn keys(&self) -> &[KeyInformation] {
        self.keys.as_slice()
    }
}

pub async fn b2_list_keys<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &'a AuthorizationToken,
    params: &'a ListKeysRequest<'a>,
) -> Result<ListKeysOk, GenericB2Error> {
    let url = format!("{}/b2api/v2/b2_list_keys", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(params);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
