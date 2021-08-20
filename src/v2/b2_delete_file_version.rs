use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{ApiUrl, AuthorizationToken, Error, FileId, FileName, JsonErrorObj};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileVersionRequest {
    file_name: FileName,
    /// The ID of the file, as returned by b2_upload_file, b2_list_file_names, or b2_list_file_versions.
    file_id: FileId,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Must be specified and set to true if deleting a file version protected by File Lock governance mode retention settings. Setting the value requires the bypassGovernance application key capability.
    bypass_governance: Option<bool>,
}

#[derive(Debug)]
pub enum DeleteFileVersionError {
    BadRequest { raw_error: JsonErrorObj },
    BadBucketId { raw_error: JsonErrorObj },
    FileNotPresent { raw_error: JsonErrorObj },
    Unauthorized { raw_error: JsonErrorObj },
    BadAuthToken { raw_error: JsonErrorObj },
    ExpiredAuthToken { raw_error: JsonErrorObj },
    AccessDenied { raw_error: JsonErrorObj },

    Unexpected { raw_error: Error },
}

impl From<reqwest::Error> for DeleteFileVersionError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for DeleteFileVersionError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (400, "bad_bucket_id") => Self::BadBucketId { raw_error: e },
            (400, "file_not_present") => Self::FileNotPresent { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (401, "access_denied") => Self::AccessDenied { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteFileVersionOk {
    file_id: FileId,
    file_name: FileName,
}

/// Deletes a specific version of a file.
///
/// If the version you delete is the latest version, and there are older versions, then the most recent older version will become the current version, and be the one that you'll get when downloading by name. See the File Versions page for more details.
///
///When used on an unfinished large file, this call has the same effect as b2_cancel_large_file.
pub async fn b2_delete_file_version(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &DeleteFileVersionRequest,
) -> Result<DeleteFileVersionOk, DeleteFileVersionError> {
    let url = format!("{}/b2api/v2/b2_delete_file_version", api_url.as_str());
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(&request)
        .send()
        .await?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp.json().await?)
    } else {
        let raw_error: JsonErrorObj = resp.json().await?;
        Err(raw_error.into())
    }
}
