use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};

#[derive(Debug, EnumSetType, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[enumset(serialize_as_list)]
pub enum Capability {
    ListKeys,
    WriteKeys,
    DeleteKeys,
    ListAllBucketNames,
    ListBuckets,
    ReadBuckets,
    WriteBuckets,
    DeleteBuckets,
    ReadBucketRetentions,
    WriteBucketRetentions,
    ReadBucketEncryption,
    WriteBucketEncryption,
    ListFiles,
    ReadFiles,
    ShareFiles,
    WriteFiles,
    DeleteFiles,
    ReadFileLegalHolds,
    WriteFileLegalHolds,
    ReadFileRetentions,
    WriteFileRetentions,
    BypassGovernance,
}

pub type Capabilities = EnumSet<Capability>;

/// all capabilities that are allowed for a key that is limited to a bucket
/// (from b2_create_key documentation)
pub fn all_per_bucket_capabilites() -> Capabilities {
    Capability::ListAllBucketNames
        | Capability::ListBuckets
        | Capability::ReadBuckets
        | Capability::ReadBucketEncryption
        | Capability::WriteBucketEncryption
        | Capability::ReadBucketRetentions
        | Capability::WriteBucketRetentions
        | Capability::ListFiles
        | Capability::ReadFiles
        | Capability::ShareFiles
        | Capability::WriteFiles
        | Capability::DeleteFiles
        | Capability::ReadFileLegalHolds
        | Capability::WriteFileLegalHolds
        | Capability::ReadFileRetentions
        | Capability::WriteFileRetentions
        | Capability::BypassGovernance
}
