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
