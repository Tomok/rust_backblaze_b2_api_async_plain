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
mod b2_download_file_by_name;
mod b2_finish_large_file;
mod b2_get_download_authorization;
mod b2_get_file_info;
mod b2_get_upload_part_url;
mod b2_get_upload_url;
mod b2_hide_file;
mod b2_list_buckets;
mod b2_list_file_names;
mod b2_list_file_versions;
mod b2_list_keys;
mod b2_list_parts;
mod b2_list_unfinished_large_files;
mod b2_start_large_file;
mod b2_update_bucket;
mod b2_update_file_legal_hold;
mod b2_update_file_retention;
mod b2_upload_file;
mod b2_upload_part;
mod buckets;
mod capabilities;
pub mod errors;
mod file;
mod file_lock;
mod file_part;
mod server_side_encryption;

pub type KeyName = String; //TODO
pub type KeyNameRef<'a> = &'a str; //TODO
pub type ApplicationKey = String; //TODO
pub type ApplicationKeyId = String; //TODO
pub type ApplicationKeyIdRef<'a> = &'a str; //TODO

pub use b2_authorize_account::AuthorizeAccountOk;
/// Authorize account function see [official documentation](https://www.backblaze.com/b2/docs/b2_authorize_account.html)
pub async fn b2_authorize_account(
    application_key_id: &str,
    application_key: &str,
) -> Result<AuthorizeAccountOk, errors::AuthorizeError> {
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
    LifeCycleRule,
};

pub use capabilities::{all_per_bucket_capabilites, Capabilities, Capability};
pub use common_structs::*;
pub use file::*;
pub use file_lock::*;
use serde::ser::SerializeSeq;
use serde::Serialize;
pub use server_side_encryption::{ServerSideEncryption, ServerSideEncryptionCustomerKey};

pub use b2_create_bucket::{b2_create_bucket, CreateBucketRequest};
pub use b2_delete_bucket::b2_delete_bucket;
pub use b2_list_buckets::b2_list_buckets;
pub use b2_list_buckets::{Bucket, ListBucketsOk, ListBucketsRequest};
pub use b2_update_bucket::{b2_update_bucket, UpdateBucketRequest};

pub use b2_get_file_info::b2_get_file_info;
pub use b2_list_file_names::{
    b2_list_file_names, ListFileNamesOk, ListFileNamesRequest, MaxFileCount,
};
pub use b2_list_file_versions::{
    b2_list_file_versions, ListFileVersionsOk, ListFileVersionsRequest,
};

pub use b2_download_file_by_id::{b2_download_file_by_id, DownloadParams};
pub use b2_download_file_by_name::{
    b2_download_file_by_name, get_b2_download_file_by_name_url, DownloadFileByNameRequest,
};
pub use b2_get_download_authorization::{
    b2_get_download_authorization, GetDownloadAuthorizationOk, GetDownloadAuthorizationRequest,
    ValidDownloadAuthorizationDurationInSeconds,
};

pub use b2_get_upload_url::{b2_get_upload_url, UploadParameters};
pub use b2_upload_file::{b2_upload_file, UploadFileParameters};

pub use b2_cancel_large_file::{b2_cancel_large_file, CancelFileOk};
pub use b2_finish_large_file::b2_finish_large_file;
pub use b2_get_upload_part_url::{b2_get_upload_part_url, UploadPartUrlParameters};
pub use b2_list_parts::{b2_list_parts, ListPartsOk, ListPartsRequest, MaxPartCount, Part};
pub use b2_list_unfinished_large_files::{
    b2_list_unfinished_large_files, ListUnfinishedLargeFilesRequest, MaxUnfinishedLargeFileCount,
};
pub use b2_start_large_file::{b2_start_large_file, StartLargeFileParameters};
pub use b2_upload_part::{b2_upload_part, UploadPartOk, UploadPartParameters};
pub use file_part::PartNumber;

pub use b2_copy_file::{b2_copy_file, CopyFileRequest, MetadataDirective};
pub use b2_copy_part::{b2_copy_part, CopyPartRequest};

pub use b2_delete_file_version::{
    b2_delete_file_version, DeleteFileVersionOk, DeleteFileVersionRequest,
};
pub use b2_hide_file::b2_hide_file;

pub use b2_create_key::{b2_create_key, CreateKeyRequest, CreatedKeyInformation};
pub use b2_delete_key::{b2_delete_key, KeyInformation};
pub use b2_list_keys::{b2_list_keys, ListKeysOk, ListKeysRequest};

pub use b2_update_file_legal_hold::{
    b2_update_file_legal_hold, UpdateFileLegalHoldOk, UpdateFileLegalHoldRequest,
};
pub use b2_update_file_retention::{
    b2_update_file_retention, UpdateFileRetentionOk, UpdateFileRetentionRequest,
};

#[cfg(test)]
mod test;

/// helper function to serialize headers::Header values
fn serialize_header<S, H>(header: &H, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    H: headers::Header + Clone,
{
    let mut temporary_storage = headers::HeaderMap::with_capacity(1);
    headers::HeaderMapExt::typed_insert(&mut temporary_storage, header.clone());
    let len = temporary_storage.len();
    match len {
        0 => serializer.serialize_none(),
        1 => {
            let entry = temporary_storage.into_iter().next().unwrap();

            let (_, value) = entry;
            //try to serialize as string first, if that does not work, use bytes
            if let Ok(s) = value.to_str() {
                s.serialize(serializer)
            } else {
                value.as_bytes().serialize(serializer)
            }
        }
        _ => {
            // not sure this could happen, but better safe than sorry
            let mut seq = serializer.serialize_seq(Some(len))?;
            for (_, value) in temporary_storage.into_iter() {
                if let Ok(s) = value.to_str() {
                    seq.serialize_element(s)?;
                } else {
                    seq.serialize_element(value.as_bytes())?;
                }
            }
            seq.end()
        }
    }
}

/// helper function to serialize Options of headers::Header values
fn serialize_header_option<S, H>(header: &Option<&H>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    H: headers::Header + Clone,
{
    match header {
        Some(h) => serialize_header(*h, serializer),
        None => serializer.serialize_none(),
    }
}
