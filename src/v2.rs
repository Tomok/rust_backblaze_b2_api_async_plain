//! Backblaze API Level V2
mod common_structs;

mod b2_authorize_account;
mod b2_download_file_by_id;
mod b2_get_upload_url;
mod b2_list_buckets;
mod b2_list_file_names;
mod buckets;
mod file;
mod server_side_encryption;

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

pub use common_structs::*;
pub use file::{FileId, FileName};
pub use server_side_encryption::{ServerSideEncryption, ServerSideEncryptionCustomerKey};

pub use b2_list_buckets::b2_list_buckets;
pub use b2_list_buckets::{ListBucketsError, ListBucketsOk, ListBucketsRequest};

pub use b2_list_file_names::{
    b2_list_file_names, ListFileNamesError, ListFileNamesOk, ListFileNamesRequest, MaxFileCount,
};

pub use b2_download_file_by_id::{b2_download_file_by_id, DownloadParams, DownloadFileError};

pub use b2_get_upload_url::{b2_get_upload_url, UploadParameters, GetUploadUrlError};

#[cfg(test)]
mod test;
