use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    errors::GenericB2Error, AccountId, ApiUrl, ApplicationKeyId, ApplicationKeyIdRef,
    AuthorizationToken, KeyInformation,
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
    start_application_key_id: Option<ApplicationKeyIdRef<'s>>,
}

impl<'s> ListKeysRequest<'s> {
    pub fn new(
        account_id: &'s AccountId,
        max_key_count: Option<NonZeroU16>,
        start_application_key_id: Option<ApplicationKeyIdRef<'s>>,
    ) -> Self {
        Self {
            account_id,
            max_key_count,
            start_application_key_id,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListKeysOk {
    keys: Vec<KeyInformation>,

    next_application_key_id: Option<ApplicationKeyId>,
}

impl ListKeysOk {
    /// Get a reference to the list keys ok's next application key id.
    pub fn next_application_key_id(&self) -> Option<ApplicationKeyIdRef<'_>> {
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
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(GenericB2Error::from_response(resp).await)
    }
}
