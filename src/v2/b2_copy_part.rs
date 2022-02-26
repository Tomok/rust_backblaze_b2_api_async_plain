use headers::Range;
use serde::Serialize;
use typed_builder::TypedBuilder;

use super::{
    errors, serialize_header_option, ApiUrl, AuthorizationToken, FileId, PartNumber,
    ServerSideEncryptionCustomerKey, UploadPartOk,
};

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CopyPartRequest<'s> {
    ///The ID of the source file being copied.
    source_file_id: &'s FileId,

    ///The ID of the large file the part will belong to, as returned by b2_start_large_file.
    large_file_id: &'s FileId,

    ///A number from 1 to 10000. The parts uploaded for one file must have contiguous numbers, starting with 1.
    part_number: PartNumber,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    /// The range of bytes to copy. If not provided, the whole source file will be copied.
    range: Option<&'s Range>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the parameters for Backblaze B2 to use for accessing the source file data using Server-Side Encryption. This parameter is required if and only if the source file has been encrypted using Server-Side Encryption with Customer-Managed Keys (SSE-C), and the provided encryption key must match the one with which the source file was encrypted.
    source_server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the parameters for Backblaze B2 to use for encrypting the copied data before storing the destination file using Server-Side Encryption.
    destination_server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
}

pub async fn b2_copy_part<'a>(
    api_url: &'a ApiUrl,
    authorization_token: &'a AuthorizationToken,
    request: &'a CopyPartRequest<'a>,
) -> Result<UploadPartOk, errors::CopyError> {
    let url = format!("{}/b2api/v2/b2_copy_part", api_url.as_str());
    let request = reqwest::Client::new()
        .post(url)
        .header("Authorization", authorization_token.as_str())
        .json(request);
    let resp = request.send().await?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await?)
    } else {
        Err(errors::CopyError::from_response(resp).await)
    }
}
