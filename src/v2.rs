//! Backblaze API Level V2
mod common_structs;

mod b2_authorize_account;
mod b2_cancel_large_file;
mod b2_copy_file;
mod b2_copy_part;
mod b2_create_bucket;
mod b2_create_key;
mod b2_delete_bucket;
mod b2_delete_file_version;
mod b2_delete_key;
mod b2_download_file_by_id;
mod b2_finish_large_file;
mod b2_get_upload_part_url;
mod b2_get_upload_url;
mod b2_list_buckets;
mod b2_list_file_names;
mod b2_list_parts;
mod b2_list_unfinished_large_files;
mod b2_start_large_file;
mod b2_upload_file;
mod b2_upload_part;
mod buckets;
mod capabilities;
mod file;
mod file_lock;
mod file_part;
mod server_side_encryption;

pub type KeyName = String; //TODO
pub type ApplicationKey = String; //TODO
pub type ApplicationKeyId = String; //TODO

pub use b2_authorize_account::{AuthorizeAccountOk, AuthorizeError};
/// Authorize account function see [official documentation](https://www.backblaze.com/b2/docs/b2_authorize_account.html)
pub async fn b2_authorize_account(
    application_key_id: &str,
    application_key: &str,
) -> Result<AuthorizeAccountOk, AuthorizeError> {
    // call the real function with the basic_uri filled in (needes to be changeable for testing)
    b2_authorize_account::b2_authorize_account(
        "https://api.backblazeb2.com",
        application_key_id,
        application_key,
    )
    .await
}

pub use buckets::{
    BucketId, BucketInfo, BucketInfoKey, BucketInfoValue, BucketName, BucketType, BucketTypes,
};

pub use capabilities::{Capabilities, Capability};
pub use common_structs::*;
pub use file::*;
pub use file_lock::*;
pub use server_side_encryption::{ServerSideEncryption, ServerSideEncryptionCustomerKey};

pub use b2_create_bucket::{b2_create_bucket, CreateBucketError, CreateBucketRequest};
pub use b2_delete_bucket::b2_delete_bucket;
pub use b2_list_buckets::b2_list_buckets;
pub use b2_list_buckets::{ListBucketsError, ListBucketsOk, ListBucketsRequest};

pub use b2_list_file_names::{
    b2_list_file_names, ListFileNamesError, ListFileNamesOk, ListFileNamesRequest, MaxFileCount,
};

pub use b2_download_file_by_id::{b2_download_file_by_id, DownloadFileError, DownloadParams};

pub use b2_get_upload_url::{b2_get_upload_url, GetUploadUrlError, UploadParameters};
pub use b2_upload_file::{b2_upload_file, UploadFileParameters};

pub use b2_cancel_large_file::{b2_cancel_large_file, CancelFileOk};
pub use b2_finish_large_file::b2_finish_large_file;
pub use b2_get_upload_part_url::{b2_get_upload_part_url, UploadPartUrlParameters};
pub use b2_list_parts::{b2_list_parts, ListPartsOk, ListPartsRequest, MaxPartCount, Part};
pub use b2_list_unfinished_large_files::{
    b2_list_unfinished_large_files, ListUnfinishedLargeFilesRequest, MaxUnfinishedLargeFileCount,
};
pub use b2_start_large_file::{b2_start_large_file, StartLargeFileError, StartLargeFileParameters};
pub use b2_upload_part::{b2_upload_part, UploadPartError, UploadPartOk, UploadPartParameters};
pub use file_part::PartNumber;

pub use b2_copy_file::{b2_copy_file, CopyFileError, CopyFileRequest, MetadataDirective, Range};
pub use b2_copy_part::{b2_copy_part, CopyPartRequest};

pub use b2_delete_file_version::{
    b2_delete_file_version, DeleteFileVersionError, DeleteFileVersionOk, DeleteFileVersionRequest,
};

pub use b2_create_key::{b2_create_key, CreateKeyRequest, CreatedKeyInformation};
pub use b2_delete_key::{b2_delete_key, KeyInformation};

#[cfg(test)]
mod test;
