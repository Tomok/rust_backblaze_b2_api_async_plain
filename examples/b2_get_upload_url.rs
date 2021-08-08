use backblaze_b2_async_plain::v2::{b2_get_upload_url, AuthorizeAccountOk, BucketId};
use std::convert::TryFrom;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "b2_get_upload_url", about = "Calls b2_get_upload_url")]
struct Params {
    #[structopt(short, long, env = "B2_AUTH_FILE")]
    /// file with the authentication data as created by the [b2_authorize_account] example with the --save option, by default will check ~/.b2_auth.yaml
    auth_file: Option<String>,

    bucket_id: String,

    #[structopt(long)]
    /// save the received upload url with the corresponding authorization token for later usage to file specified by save_file (default: ~/.b2_upload_url.yaml)
    save: bool,

    #[structopt(long, env = "B2_UPLOAD_URL_FILE")]
    /// file to store the upload url into, requires --save otherwise nothing will be saved
    save_file: Option<String>,
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();
    let save = if p.save {
        let output_path: PathBuf = match p.save_file {
            Some(path) => PathBuf::from(path),
            None => {
                let mut home = home::home_dir().expect("Could not get home directory. Please specify path for storing the value using --save-file");
                home.push(".b2_upload_url.yaml");
                home
            }
        };
        Some(output_path)
    } else {
        None
    };

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

    let bucket_id = BucketId::try_from(p.bucket_id).expect("Invalid Bucket Id");

    let response = b2_get_upload_url(
        auth_data.api_url(),
        auth_data.authorization_token(),
        &bucket_id,
    )
    .await;

    let mut stdout = io::stdout();
    writeln!(stdout, "Result: {:#?}", response).unwrap();
    stdout.flush().unwrap();

    // this is synchrounous as well ... todo: make async
    if let Some(save_file) = save {
        if let Ok(auth_data) = response {
            match std::fs::File::create(&save_file) {
                Ok(f) => {
                    serde_yaml::to_writer(&f, &auth_data).unwrap();
                }
                Err(e) => panic!(
                    "Could not open file {:#?} to save upload url data:\n {:#?}",
                    save_file, e
                ),
            }
            println!("Upload Url saved to file {:#?}", save_file);
        } else {
            println!("Retrieving Upload Url failed, will not save result");
        }
    }
}
