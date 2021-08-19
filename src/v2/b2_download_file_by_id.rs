use super::{
    AuthorizationToken, DownloadUrl, Error, FileId, JsonErrorObj, ServerSideEncryptionCustomerKey,
};

use http_range::HttpRange;
use http_types::Url;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct DownloadParams<'s> {
    #[builder(default, setter(strip_option))]
    authorization_token: Option<&'s AuthorizationToken>,
    #[builder(default, setter(strip_option))]
    range: Option<&'s HttpRange>,
    //TODO: b2* header ...
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey>,
}

#[derive(Debug)]
pub enum DownloadFileError {
    // TODO: update acc. to documentation
    BadRequest {
        raw_error: JsonErrorObj,
    },
    Unauthorized {
        raw_error: JsonErrorObj,
    },
    /// not listed in the api in <https://www.backblaze.com/b2/docs/b2_list_buckets.html> but I assume this could happen as well
    TransactionCapExceeded {
        raw_error: JsonErrorObj,
    },
    BadAuthToken {
        raw_error: JsonErrorObj,
    },
    ExpiredAuthToken {
        raw_error: JsonErrorObj,
    },
    Unexpected {
        raw_error: Error,
    },
}

impl From<reqwest::Error> for DownloadFileError {
    fn from(e: reqwest::Error) -> Self {
        //TODO separate error for network / timeouts??
        Self::Unexpected {
            raw_error: Error::ReqwestError(e),
        }
    }
}

impl From<JsonErrorObj> for DownloadFileError {
    fn from(e: JsonErrorObj) -> Self {
        match (e.status as usize, e.code.as_str()) {
            (400, "bad_request") => Self::BadRequest { raw_error: e },
            (401, "unauthorized") => Self::Unauthorized { raw_error: e },
            (401, "bad_auth_token") => Self::BadAuthToken { raw_error: e },
            (401, "expired_auth_token") => Self::ExpiredAuthToken { raw_error: e },
            (403, "transaction_cap_exceeded") => Self::TransactionCapExceeded { raw_error: e },
            _ => Self::Unexpected {
                raw_error: Error::JsonError(e),
            },
        }
    }
}
/// downloads a file by ID, does return a reqwest::Response object, if the server returned http status OK (200).
pub async fn b2_download_file_by_id(
    download_url: &DownloadUrl,
    file_id: &FileId,
    params: &DownloadParams<'_>,
) -> Result<reqwest::Response, DownloadFileError> {
    let url_base_str = format!("{}/b2api/v2/b2_download_file_by_id", download_url.as_str());
    let url = Url::parse_with_params(&url_base_str, &[("fileId", file_id.as_str())]).unwrap();
    let mut request_builder = reqwest::Client::new().get(url);
    if let Some(auth) = params.authorization_token {
        request_builder = request_builder.header("Authorization", auth.as_str());
    }
    if let Some(range) = params.range {
        request_builder = request_builder.header(
            "Range",
            format!("bytes={}-{}", range.start, range.start + range.length),
        );
    }
    // todo b2* headers
    if let Some(_sse) = params.server_side_encryption {
        todo!();
    }
    let resp = request_builder
        .send()
        .await
        .map_err(DownloadFileError::from)?;
    if resp.status().as_u16() == http_types::StatusCode::Ok as u16 {
        Ok(resp)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(DownloadFileError::from)?;
        Err(raw_error.into())
    }
}
