use reqwest::Body;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::header_serializer::HeadersFrom;

use super::{
    errors::UploadPartError, FileId, Md5Digest, PartNumber, ServerSideEncryption,
    ServerSideEncryptionCustomerKey, Sha1Digest, Sha1DigestRef, TimeStamp, UploadPartUrlParameters,
};

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UploadPartParameters<'s> {
    #[serde(rename = "X-Bz-Part-Number")]
    /// A number from 1 to 10000. The parts uploaded for one file must have contiguous numbers, starting with 1.
    part_number: PartNumber,

    #[serde(rename = "Content-Length")]
    /// The number of bytes in the file being uploaded. Note that this header is required; you cannot leave it out and just use chunked encoding.
    /// The minimum size of every part but the last one is 5MB.
    /// When sending the SHA1 checksum at the end, the Content-Length should be set to the size of the file plus the 40 bytes of hex checksum.
    content_length: u64,

    #[serde(rename = "X-Bz-Content-Sha1")]
    /// The SHA1 checksum of the this part of the file. B2 will check this when the part is uploaded, to make sure that the data arrived correctly.
    /// The same SHA1 checksum must be passed to b2_finish_large_file.
    /// You may optionally provide the SHA1 at the end of the upload.
    content_sha1: Sha1DigestRef<'s>,

    #[builder(default, setter(strip_option))]
    #[serde(skip)] //will be serialized manually
    server_side_encryption: Option<&'s ServerSideEncryptionCustomerKey<'s>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPartOk {
    file_id: FileId,
    part_number: PartNumber,
    content_length: u64,
    content_sha1: Sha1Digest,
    content_md5: Option<Md5Digest>,
    server_side_encryption: Option<ServerSideEncryption>,
    upload_timestamp: TimeStamp,
}

impl UploadPartOk {
    /// Get a reference to the upload part ok's file id.
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Get the upload part ok's part number.
    pub fn part_number(&self) -> PartNumber {
        self.part_number
    }

    /// Get the upload part ok's content length.
    pub fn content_length(&self) -> u64 {
        self.content_length
    }

    /// Get a reference to the upload part ok's content sha1.
    pub fn content_sha1(&self) -> &Sha1Digest {
        &self.content_sha1
    }

    /// Get a reference to the upload part ok's content md5.
    pub fn content_md5(&self) -> Option<&Md5Digest> {
        self.content_md5.as_ref()
    }

    /// Get a reference to the upload part ok's server side encryption.
    pub fn server_side_encryption(&self) -> Option<&ServerSideEncryption> {
        self.server_side_encryption.as_ref()
    }

    /// Get the upload part ok's upload timestamp.
    pub fn upload_timestamp(&self) -> i64 {
        self.upload_timestamp
    }
}

pub async fn b2_upload_part<'a, T: Into<Body>>(
    uploader_params: &'a mut UploadPartUrlParameters,
    upload_part_params: &'a UploadPartParameters<'a>,
    file_contents: T,
) -> Result<UploadPartOk, UploadPartError> {
    let mut request = reqwest::Client::new()
        .post(uploader_params.upload_url().as_str())
        .header(
            "Authorization",
            uploader_params.authorization_token().as_str(),
        )
        .headers_from(upload_part_params)
        .body(file_contents);
    if let Some(ssec) = upload_part_params.server_side_encryption {
        request = ssec.add_to_request_as_header(request);
    }
    let resp = request.send().await.map_err(UploadPartError::from)?;
    if resp.status() == http::StatusCode::OK {
        Ok(resp.json().await.map_err(UploadPartError::from)?)
    } else {
        Err(UploadPartError::from_response(resp).await)
    }
}
