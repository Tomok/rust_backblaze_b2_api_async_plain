use headers::Range;
use serde::Serialize;
use typed_builder::TypedBuilder;

use super::{
    errors, serialize_header_option, ApiUrl, AuthorizationToken, BucketId, ContentTypeRef, FileId,
    FileInformation, FileName, FileRetention, LegalHoldOnOff, ServerSideEncryptionCustomerKey,
};

#[cfg(feature = "b2_unstable")]
use super::FileInfo;

#[derive(Debug, Serialize)]
pub enum MetadataDirective {
    COPY,
    REPLACE,
}

#[derive(Debug, Serialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CopyFileRequest<'s> {
    ///The ID of the source file being copied.
    source_file_id: &'s FileId,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The ID of the bucket where the copied file will be stored. If this is not set, the copied file will be added to the same bucket as the source file.
    /// Note that the bucket containing the source file and the destination bucket must belong to the same account.
    destination_bucket_id: Option<&'s BucketId>,

    /// The name of the new file being created.
    file_name: &'s FileName,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    /// The range of bytes to copy. If not provided, the whole source file will be copied.
    range: Option<&'s Range>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The strategy for how to populate metadata for the new file. If COPY is the indicated strategy, then supplying the contentType or fileInfo param is an error.
    metadata_directive: Option<MetadataDirective>,

    #[builder(default, setter(strip_option))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_header_option"
    )]
    /// Must only be supplied if the metadataDirective is REPLACE.
    /// The MIME type of the content of the file, which will be returned in the Content-Type header when downloading the file.
    content_type: Option<ContentTypeRef<'s>>,

    #[cfg(feature = "b2_unstable")]
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Must only be supplied if the metadataDirective is REPLACE.
    /// This field stores the metadata that will be stored with the file.
    /// It follows the same rules that are applied to b2_upload_file
    file_info: Option<&'s FileInfo>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the File Lock retention settings for the new file. Setting the value requires the writeFileRetentions capability and that the destination bucket is File Lock-enabled.
    file_retention: Option<&'s FileRetention>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the File Lock legal hold status for the new file. Setting the value requires the writeFileLegalHolds capability and that the destination bucket is File Lock-enabled.
    legal_hold: Option<&'s LegalHoldOnOff>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the parameters for Backblaze B2 to use for accessing the source file data using Server-Side Encryption. This parameter is required if and only if the source file has been encrypted using Server-Side Encryption with Customer-Managed Keys (SSE-C), and the provided encryption key must match the one with which the source file was encrypted.
    source_server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If present, specifies the parameters for Backblaze B2 to use for encrypting the copied data before storing the destination file using Server-Side Encryption.
    destination_server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
}

pub async fn b2_copy_file(
    api_url: &ApiUrl,
    authorization_token: &AuthorizationToken,
    request: &CopyFileRequest<'_>,
) -> Result<FileInformation, errors::CopyError> {
    let url = format!("{}/b2api/v2/b2_copy_file", api_url.as_str());
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
