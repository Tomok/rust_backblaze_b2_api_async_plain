use std::{convert::TryFrom, path::PathBuf};

use backblaze_b2_async_plain::v2::{
    b2_download_file_by_id, AuthorizeAccountOk, DownloadParams, FileId,
};
use reqwest::Response;
use structopt::StructOpt;
use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
};
#[derive(StructOpt)]
#[structopt(
    name = "b2_download_file_by_id",
    about = "Calls b2_download_file_by_id and downloads the file"
)]
struct Params {
    #[structopt(short, long, env = "B2_AUTH_FILE")]
    /// file with the authentication data as created by the [b2_authorize_account] example with the --save option, by default will check ~/.b2_auth.yaml
    auth_file: Option<String>,

    /// ID of the file to be downloaded
    file_id: String,
    #[structopt(long)]
    /// file to save data to, if not set, download will be printed to standard out
    out: Option<String>,
}

async fn save_download_response<O>(r: &mut Response, output: &mut O)
where
    O: Unpin + AsyncWriteExt,
{
    while let Some(chunk) = r.chunk().await.expect("Failed during download") {
        output
            .write_all(&chunk)
            .await
            .expect("Failed while writing / printing downloaded data");
    }
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
    let file_id = FileId::try_from(p.file_id).expect("Invalid file Id");
    let download_params = DownloadParams::builder()
        .authorization_token(auth_data.authorization_token())
        .build();
    let mut resp =
        b2_download_file_by_id(auth_data.download_url(), &file_id, &download_params).await;
    match resp {
        Ok(ref mut r) => {
            match p.out {
                None => save_download_response(r, &mut io::stdout()).await,
                Some(file_name) => {
                    let mut f = File::create(file_name)
                        .await
                        .expect("Could not create output file");
                    save_download_response(r, &mut f).await;
                }
            };
        }
        Err(e) => {
            panic!("Download error: {:#?}", e);
        }
    }
}
