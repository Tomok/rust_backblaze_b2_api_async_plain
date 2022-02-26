use super::{
    errors::DownloadFileError, BucketName, DownloadAuthorizationToken, DownloadUrl, FileName,
};

use headers::{HeaderMap, HeaderMapExt};
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct DownloadFileByNameRequest<'s> {
    bucket_name: &'s BucketName,
    file_name: &'s FileName,
    #[builder(default, setter(strip_option))]
    range: Option<&'s headers::Range>,
}

/// gets a download URL for a given file in a given directory
///
/// public as it might be usefull to give this url to a different application/client/...
pub fn get_b2_download_file_by_name_url(
    download_url: &DownloadUrl,
    bucket_name: &BucketName,
    file_name: &FileName,
) -> String {
    format!(
        "{}/file/{}/{}",
        download_url.as_str(),
        bucket_name.as_str(),
        file_name.as_str()
    )
}

/// downloads a file by Name, does return a reqwest::Response object, if the server returned http status OK (200)
/// or PartialContent (206) if a range was used.
pub async fn b2_download_file_by_name<'a, 'b, AuthToken>(
    download_url: &DownloadUrl,
    authorization_token: Option<&'a AuthToken>,
    request: &DownloadFileByNameRequest<'b>,
) -> Result<reqwest::Response, DownloadFileError>
where
    AuthToken: DownloadAuthorizationToken,
{
    let url =
        get_b2_download_file_by_name_url(download_url, request.bucket_name, request.file_name);
    let mut request_builder = reqwest::Client::new().get(url);
    if let Some(auth) = authorization_token {
        request_builder = request_builder.header("Authorization", auth.download_token_as_str());
    }
    if let Some(range) = request.range {
        let mut headers = HeaderMap::with_capacity(1);
        headers.typed_insert(range.clone());
        request_builder = request_builder.headers(headers);
    }
    let resp = request_builder
        .send()
        .await
        .map_err(DownloadFileError::from)?;
    let expected_status = if request.range.is_none() {
        http::StatusCode::OK
    } else {
        http::StatusCode::PARTIAL_CONTENT
    };
    if resp.status().as_u16() == expected_status {
        Ok(resp)
    } else {
        Err(DownloadFileError::from_response(resp).await)
    }
}
