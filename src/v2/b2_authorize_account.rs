use super::{
    errors, AccountId, ApiUrl, ApplicationKeyIdRef, ApplicationKeyRef, AuthorizationToken,
    Capabilities, DownloadUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeAccountOk {
    absolute_minimum_part_size: u32,
    account_id: AccountId,
    allowed: AuthorizeAccountAllowed,
    api_url: ApiUrl,
    authorization_token: AuthorizationToken,
    download_url: DownloadUrl,
    recommended_part_size: u32,
}

impl AuthorizeAccountOk {
    /// Get a reference to the authorize account ok's absolute minimum part size.
    pub fn absolute_minimum_part_size(&self) -> u32 {
        self.absolute_minimum_part_size
    }

    /// Get a reference to the authorize account ok's account id.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Get a reference to the authorize account ok's allowed.
    pub fn allowed(&self) -> &AuthorizeAccountAllowed {
        &self.allowed
    }

    /// Get a reference to the authorize account ok's api url.
    pub fn api_url(&self) -> &ApiUrl {
        &self.api_url
    }

    /// Get a reference to the authorize account ok's authorization token.
    pub fn authorization_token(&self) -> &AuthorizationToken {
        &self.authorization_token
    }

    /// Get a reference to the authorize account ok's download url.
    pub fn download_url(&self) -> &DownloadUrl {
        &self.download_url
    }

    /// Get a reference to the authorize account ok's recommended part size.
    pub fn recommended_part_size(&self) -> u32 {
        self.recommended_part_size
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeAccountAllowed {
    pub capabilities: Capabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<String>,
}

pub async fn b2_authorize_account<'a>(
    basic_uri: &str,
    application_key_id: ApplicationKeyIdRef<'a>,
    application_key: ApplicationKeyRef<'a>,
) -> Result<AuthorizeAccountOk, errors::AuthorizeError> {
    let url = format!("{}/b2api/v2/b2_authorize_account", basic_uri);
    //https://api.backblazeb2.com
    let resp = reqwest::Client::new()
        .get(url)
        .basic_auth(application_key_id.as_str(), Some(application_key.as_str()))
        .send()
        .await?;
    if resp.status() == http::StatusCode::OK {
        let auth_ok: AuthorizeAccountOk = resp.json().await?;
        Ok(auth_ok)
    } else {
        Err(errors::AuthorizeError::from_response(resp).await)
    }
}
#[cfg(test)]
mod test {
    use super::*;

    use crate::v2::test::mock_server::*;

    #[tokio::test]
    async fn test_b2_authorize_account() {
        let mock = B2MockServer::start().await;
        mock.register_default_auth_handler().await;
        let res = b2_authorize_account(
            &mock.uri(),
            &FAKE_APPLICATION_KEY_ID.to_owned().try_into().unwrap(),
            &FAKE_APPLICATION_KEY.to_owned().try_into().unwrap(),
        )
        .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_b2_authorize_account_account_invalid_password() {
        let mock = B2MockServer::start().await;
        mock.register_default_auth_handler().await;
        let res = b2_authorize_account(
            &mock.uri(),
            &FAKE_APPLICATION_KEY_ID.to_owned().try_into().unwrap(),
            &"Invalid Key".to_owned().try_into().unwrap(),
        )
        .await;
        let err = res.unwrap_err();
        assert!(matches!(err, errors::AuthorizeError::Unauthorized { .. }));
    }
}
