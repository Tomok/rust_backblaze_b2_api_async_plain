use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{ApiUrl, AuthorizationToken, Error, FileId, FileName, JsonErrorObj, LegalHoldOnOff};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileLegalHoldRequest {
    file_name: FileName,
    file_id: FileId,
    legal_hold: LegalHoldOnOff,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileLegalHoldOk {
    file_name: FileName,
    file_id: FileId,
    legal_hold: LegalHoldOnOff,
}

#[derive(Debug)]
pub enum UpdateFileLegalHoldError {
    BadRequest {
        raw_error: JsonErrorObj,
    },
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    AccessDenied {
        raw_error: JsonErrorObj,
    },

    CapExceeded {
        raw_error: JsonErrorObj,
    },
    ///This operation is not allowed because the specified file is a "hide marker."
    MethodNotAllowed {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

impl From<reqwest::Error> for UpdateFileLegalHoldError {
    fn from(err: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(err),
        }
    }
}

impl From<JsonErrorObj> for UpdateFileLegalHoldError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (401, "access_denied") => Self::AccessDenied { raw_error: e },
            (403, "cap_exceeded") => Self::CapExceeded { raw_error: e },
            (405, "method_not_allowed") => Self::MethodNotAllowed { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

pub async fn b2_update_file_legal_hold(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &UpdateFileLegalHoldRequest,
) -> Result<UpdateFileLegalHoldOk, UpdateFileLegalHoldError> {
    let url = format!("{}/b2api/v2/b2_update_file_legal_hold", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
