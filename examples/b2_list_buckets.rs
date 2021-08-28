use std::path::PathBuf;
use std::vec::Vec;
use std::{collections::HashSet, convert::TryFrom};
use structopt::StructOpt;

use backblaze_b2_async_plain::v2::{
    b2_list_buckets, AuthorizeAccountOk, BucketId, BucketName, BucketType, BucketTypes,
    ListBucketsRequest,
};

#[derive(StructOpt)]
#[structopt(name = "b2_list_buckets", about = "Calls b2_list_buckets")]
struct Params {
    #[structopt(short, long, env = "B2_AUTH_FILE")]
    /// file with the authentication data as created by the [b2_authorize_account] example with the --save option, by default will check ~/.b2_auth.yaml
    auth_file: Option<String>,

    #[structopt(long)]
    bucket_id: Option<String>,
    #[structopt(long)]
    bucket_name: Option<String>,
    #[structopt(long)]
    /// bucket types to request ["all"] by default
    bucket_types: Vec<String>,
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();
    let auth_data = {
        let auth_file = match p.auth_file {
            Some(path) => PathBuf::from(path),
            None => {
                let mut home = home::home_dir().expect("Could not get home directory. Please specify path where authentication data is stored using --auth-file");
                home.push(".b2_auth.yaml");
                home
            }
        };
        let file =
            std::fs::File::open(auth_file).expect("Could not open file with authentication data");
        let auth_data: AuthorizeAccountOk = serde_yaml::from_reader(file)
            .expect("Could not read authentication data from authentication data file");
        auth_data
    };

    let bucket_id = p
        .bucket_id
        .map(|s| BucketId::try_from(s).expect("Invalid Bucket Id"));
    let bucket_name = p
        .bucket_name
        .map(|s| BucketName::try_from(s).expect("Invalid Bucket Name"));
    let bucket_type_count = p.bucket_types.len();
    let bucket_types = if bucket_type_count > 0 {
        let mut set = HashSet::with_capacity(bucket_type_count);
        for t in p.bucket_types.iter() {
            let v = BucketType::from(t);
            if v.is_other() {
                println!("Warning: Unknown bucket type requested {:#?}, if Backblaze API was extended by this, the BucketType enum should be extended", v);
            }
            set.insert(v);
        }
        Some(BucketTypes::List(set))
    } else {
        None
    };

    //usually it is a good idea to use [ListBucketsRequest::builder] instead of new() here, but since we have all parameters
    let request_params = ListBucketsRequest::new(
        auth_data.account_id(),
        bucket_id.as_ref(),
        bucket_name.as_ref(),
        bucket_types.as_ref(),
    );

    dbg!(serde_json::to_string_pretty(&request_params).unwrap());

    let res = b2_list_buckets(
        &auth_data.api_url(),
        &auth_data.authorization_token(),
        &request_params,
    )
    .await;

    println!("Result: {:#?}", res);
}
