use super::{
    errors::DownloadFileError, AuthorizationToken, DownloadUrl, FileId, JsonErrorObj,
    ServerSideEncryptionCustomerKey,
};

use http_range::HttpRange;
use http_types::Url;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct DownloadParams<'s> {
    #[builder(default, setter(strip_option))]
    range: Option<&'s HttpRange>,
    //TODO: b2* header ...
    #[builder(default, setter(strip_option))]
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey>,
}

/// downloads a file by ID, does return a reqwest::Response object, if the server returned http status OK (200)
/// or PartialContent (206) if a range was used.
pub async fn b2_download_file_by_id(
    download_url: &DownloadUrl,
    authorization_token: Option<&AuthorizationToken>,
    file_id: &FileId,
    params: &DownloadParams<'_>,
) -> Result<reqwest::Response, DownloadFileError> {
    let url_base_str = format!("{}/b2api/v2/b2_download_file_by_id", download_url.as_str());
    let url = Url::parse_with_params(&url_base_str, &[("fileId", file_id.as_str())]).unwrap();
    let mut request_builder = reqwest::Client::new().get(url);
    if let Some(auth) = authorization_token {
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
    let expected_status = if params.range.is_none() {
        http_types::StatusCode::Ok
    } else {
        http_types::StatusCode::PartialContent
    };
    if resp.status().as_u16() == expected_status as u16 {
        Ok(resp)
    } else {
        let raw_error: JsonErrorObj = resp.json().await.map_err(DownloadFileError::from)?;
        Err(raw_error.into())
    }
}
