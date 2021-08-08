use std::{
    convert::TryInto,
    io::{self, SeekFrom, Write},
    path::PathBuf,
};

use reqwest::Body;
use structopt::StructOpt;

use backblaze_b2_async_plain::v2::{b2_upload_file, UploadFileParameters, UploadParameters};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

#[derive(StructOpt)]
#[structopt(
    name = "b2_upload_file",
    about = "Calls b2_upload_file, should not be used for files larger than 200MB"
)]
struct Params {
    #[structopt(short, long, env = "B2_UPLOAD_URL_FILE")]
    /// file with the upload url as created by the [b2_get_upload_url] example with the --save option, by default will check ~/.b2_upload_url.yaml
    upload_url_file: Option<String>,

    #[structopt(parse(from_os_str))]
    /// file to upload
    file_to_upload: PathBuf,

    #[structopt(short, long)]
    /// filename to use in b2 storage, if not set, will be the filename (without any path) from [file_to_upload]
    target_filename: Option<String>,
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();
    let mut upload_url = {
        let upload_url_file: PathBuf = match p.upload_url_file {
            Some(path) => PathBuf::from(path),
            None => {
                let mut home = home::home_dir().expect("Could not get home directory. Please specify path for upload_url using --upload_url_file");
                home.push(".b2_upload_url.yaml");
                home
            }
        };
        let file =
            std::fs::File::open(upload_url_file).expect("Could not open file with upload url");

        let url: UploadParameters =
            serde_yaml::from_reader(file).expect("Could not read upload url data from file");
        url
    };
    let target_filename = p.target_filename.unwrap_or(
        p.file_to_upload
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .expect("Filename could not be determined"),
    );

    let mut file = tokio::fs::File::open(p.file_to_upload)
        .await
        .expect("Could not open file to upload");
    //determine sha1 and length first
    file.seek(SeekFrom::Start(0))
        .await
        .expect("Could not go to start of upload file");
    let mut hasher = sha1::Sha1::new();
    let mut file_len = 0u64;
    let mut buf = [0u8; 4096];
    loop {
        let bytes_read = file
            .read(&mut buf)
            .await
            .expect("Reading file for sha1 hashing failed");
        if bytes_read > 0 {
            file_len += bytes_read as u64;
            hasher.update(&buf[..bytes_read]);
        } else {
            break; // 0 bytes were read -> file end was reached
        }
    }
    let sha1 = hasher.digest();
    file.seek(SeekFrom::Start(0))
        .await
        .expect("Could not go to start of upload file");

    dbg!(&sha1);
    let upload_params = UploadFileParameters::builder()
        .file_name(
            target_filename
                .try_into()
                .expect("Filename not b2 compatible"),
        )
        .content_length(file_len)
        .content_sha1(sha1.to_string())
        .build();

    let res = b2_upload_file(
        &mut upload_url,
        &upload_params,
        Body::wrap_stream(ReaderStream::new(file)),
    )
    .await;

    let mut stdout = io::stdout();
    writeln!(stdout, "Result: {:#?}", res).unwrap();
    stdout.flush().unwrap();
}
