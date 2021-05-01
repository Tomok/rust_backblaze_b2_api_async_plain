use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};
use structopt::StructOpt;

use backblaze_b2_async_plain::v2::{
    b2_list_file_names, AuthorizeAccountOk, FileName, ListFileNamesRequest, MaxFileCount,
};

#[derive(StructOpt)]
#[structopt(name = "b2_list_file_names", about = "Calls b2_list_file_names")]
struct Params {
    #[structopt(short, long, env = "B2_AUTH_FILE")]
    /// file with the authentication data as created by the [b2_authorize_account] example with the --save option, by default will check ~/.b2_auth.yaml
    auth_file: Option<String>,

    #[structopt(long)]
    bucket_id: String,
    #[structopt(long)]
    start_file_name: Option<String>,
    #[structopt(long)]
    max_file_count: Option<u16>,
    #[structopt(long)]
    prefix: Option<String>,
    #[structopt(long)]
    delimiter: Option<String>,
    #[structopt(long)]
    /// if set, further calls will be made, if the returned struct contained a "next_file_name"
    continue_requests: bool,
}

#[tokio::main]
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

    let bucket_id = p.bucket_id.try_into().expect("Invalid Bucket Id");
    let start_file_name = p
        .start_file_name
        .map(|s| FileName::try_from(s).expect("Invalid start file name"));
    let max_file_count = p.max_file_count.map(|m| {
        MaxFileCount::try_from(m).unwrap_or_else(|e| panic!("Invalid Maximum count: {:#?}", e))
    });
    let prefix = p.prefix;
    let delimiter = p.delimiter;

    //usually it is a good idea to use [ListBucketsRequest::builder] instead of new() here, but since we have all parameters
    let mut request_params = ListFileNamesRequest::new(
        bucket_id,
        start_file_name,
        max_file_count,
        prefix,
        delimiter,
    );

    dbg!(serde_json::to_string_pretty(&request_params).unwrap());
    let mut request_counter = 1usize;
    loop {
        let res = b2_list_file_names(
            &auth_data.api_url(),
            &auth_data.authorization_token(),
            &request_params,
        )
        .await;

        if !p.continue_requests {
            println!("Result:  {:#?}", res);
            break;
        } else {
            println!("Result Request No {}: {:#?}", request_counter, res);

            let next_file_name = match res {
                std::result::Result::Ok(r) => (*r.next_file_name()).clone(),
                std::result::Result::Err(_) => None,
            };
            if next_file_name.is_none() {
                break;
            } else {
                request_params.set_start_file_name(next_file_name);
                request_counter += 1;
            }
        }
    }
}
