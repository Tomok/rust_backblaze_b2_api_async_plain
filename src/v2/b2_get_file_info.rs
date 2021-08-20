use serde::Serialize;

use super::{ApiUrl, AuthorizationToken, Error, FileId, FileInformation, JsonErrorObj};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetFileInfoRequest<'a> {
    file_id: &'a FileId,
}
#[derive(Debug)]
pub enum GetFileInfoError {
    ///The request had the wrong fields or illegal values. The message returned with the error will describe the problem.
    BadRequest {
        raw_error: JsonErrorObj,
    },

    ///The auth token used is valid, but does not authorize this call with these parameters. The capabilities of an auth token are determined by the application key used with b2_authorize_account.
    Unauthorized {
        raw_error: JsonErrorObj,
    },
    ///The auth token used is not valid. Call b2_authorize_account again to either get a new one, or an error message describing the problem.
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ///The auth token used has expired. Call b2_authorize_account again to get a new one.
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    ///File is not in B2 Cloud Storage.
    NotFound {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

impl From<reqwest::Error> for GetFileInfoError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for GetFileInfoError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (404, "not_found") => Self::NotFound { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_get_file_info(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    file_id: &FileId,
) -> Result<FileInformation, GetFileInfoError> {
    let url = format!("{}/b2api/v2/b2_get_file_info", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&GetFileInfoRequest { file_id })
        .send()
        .await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
