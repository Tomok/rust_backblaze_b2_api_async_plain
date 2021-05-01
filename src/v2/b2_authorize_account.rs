use super::{AccountId, ApiUrl, AuthorizationToken, DownloadUrl, Error, JsonErrorObj};
use http_types::StatusCode;
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
    pub capabilities: Vec<String>, //TODO: use enum instead ??
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<String>,
}

#[derive(Debug)]
pub enum AuthorizeError {
    BadRequest { raw_error: JsonErrorObj },
    Unauthorized { raw_error: JsonErrorObj },
    Unsupported { raw_error: JsonErrorObj },
    TransactionCapExceeded { raw_error: JsonErrorObj },
    Unexpected { raw_error: Error },
}

impl From<reqwest::Error> for AuthorizeError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        AuthorizeError::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

pub async fn b2_authorize_account(
    basic_uri: &str,
    application_key_id: &str,
    application_key: &str,
) -> Result<AuthorizeAccountOk, AuthorizeError> {
    let url = format!("{}/b2api/v2/b2_authorize_account", basic_uri);
    dbg!(&url);
    //https://api.backblazeb2.com
    let resp = reqwest::Client::new()
        .get(url)
        .basic_auth(application_key_id, Some(application_key))
        .send()
        .await
        .map_err(AuthorizeError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: AuthorizeAccountOk = resp.json().await.map_err(AuthorizeError::from)?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(AuthorizeError::from)?;
        let auth_error = match (raw_error.status, raw_error.code.as_str()) {
            (StatusCode::BadRequest, "bad_request") => AuthorizeError::BadRequest { raw_error },
            (StatusCode::Unauthorized, "unauthorized") => {
                AuthorizeError::Unauthorized { raw_error }
            }
            (StatusCode::Unauthorized, "unsupported") => AuthorizeError::Unsupported { raw_error },
            (StatusCode::Forbidden, "transaction_cap_exceeded") => {
                AuthorizeError::TransactionCapExceeded { raw_error }
            }
            _ => AuthorizeError::Unexpected {
                raw_error: Error::JsonError(raw_error),
            },
        };
        Err(auth_error)
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
        let res =
            b2_authorize_account(&mock.uri(), FAKE_APPLICATION_KEY_ID, FAKE_APPLICATION_KEY).await;
        dbg!(&res);
        assert_eq!(true, res.is_ok());
    }

    #[tokio::test]
    async fn test_b2_authorize_account_account_invalid_password() {
        let mock = B2MockServer::start().await;
        mock.register_default_auth_handler().await;
        let res = b2_authorize_account(&mock.uri(), FAKE_APPLICATION_KEY_ID, "Invalid Key").await;
        let err = res.unwrap_err();
        assert!(matches!(err, AuthorizeError::Unauthorized { .. }));
    }
}
