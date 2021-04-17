use super::{Error, JsonErrorObj, MachineReadableJsonErrorObj};
use http_types::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeAccountOk {
    absolute_minimum_part_size: u32,
    account_id: String,
    allowed: AuthorizeAccountAllowed,
    api_url: String,
    authorization_token: String,
    download_url: String,
    recommended_part_size: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeAccountAllowed {
    pub capabilities: Vec<String>, //TODO: use enum instead ??
    #[serde(default)]
    pub bucket_id: Option<String>,
    #[serde(default)]
    pub bucket_name: Option<String>,
    #[serde(default)]
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
        .map_err(|e| AuthorizeError::from(e))?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        let auth_ok: AuthorizeAccountOk = resp.json().await.map_err(|e| AuthorizeError::from(e))?;
        Ok(auth_ok)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(|e| AuthorizeError::from(e))?;
        let auth_error = match raw_error.machine_readable() {
            MachineReadableJsonErrorObj {
                status: StatusCode::BadRequest,
                code: "bad_request",
            } => AuthorizeError::BadRequest { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Unauthorized,
                code: "unauthorized",
            } => AuthorizeError::Unauthorized { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Unauthorized,
                code: "unsupported",
            } => AuthorizeError::Unsupported { raw_error },
            MachineReadableJsonErrorObj {
                status: StatusCode::Forbidden,
                code: "transaction_cap_exceeded",
            } => AuthorizeError::TransactionCapExceeded { raw_error },
            _ => AuthorizeError::Unexpected {
                raw_error: Error::JSONError(raw_error),
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
