use std::{convert::TryInto, num::NonZeroU16, path::PathBuf};

use backblaze_b2_async_plain::v2::{
    b2_finish_large_file, b2_get_upload_part_url, b2_start_large_file, b2_upload_part,
    AuthorizeAccountOk, FileInformation, PartNumber, StartLargeFileParameters,
    UploadPartParameters,
};
use num_integer::{div_ceil, div_floor};
use reqwest::Body;
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(StructOpt)]
#[structopt(
    name = "b2_upload_file_parts",
    about = "Uploads a file in multiple parts.",
    long_about = "Uploads a file in multiple parts. \n Contrary to other examples, this will perform calls to multiple api endpoints"
)]
struct Params {
    #[structopt(short, long, env = "B2_AUTH_FILE")]
    /// file with the authentication data as created by the [b2_authorize_account] example with the --save option, by default will check ~/.b2_auth.yaml
    auth_file: Option<String>,

    bucket_id: String,

    #[structopt(parse(from_os_str))]
    /// file to upload
    file_to_upload: PathBuf,

    #[structopt(short, long)]
    /// filename to use in b2 storage, if not set, will be the filename (without any path) from [file_to_upload]
    target_filename: Option<String>,

    #[structopt(short, long)]
    /// in how many chunks should the file be split?
    chunk_count: Option<NonZeroU16>,
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();

    let target_filename = p.target_filename.unwrap_or(
        p.file_to_upload
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .expect("Filename could not be determined"),
    );
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

    let mut file_to_upload = File::open(p.file_to_upload)
        .await
        .expect("Could not open file to be uploaded");
    let file_size = file_to_upload
        .metadata()
        .await
        .expect("Could not get file size for file to be uploaded")
        .len();

    let (chunk_count, normal_chunk_size) =
        determine_chunk_count_and_size(file_size, &p.chunk_count);

    //check that chunk_count and normal_chunk_size are ok...
    assert!(chunk_count.get() <= PartNumber::max_part_number());
    if chunk_count.get() > 1 {
        assert!(normal_chunk_size >= MIN_NOT_LAST_CHUNK_SIZE);
    }
    let last_chunk_size = file_size - (chunk_count.get() as u64 - 1) * normal_chunk_size;
    assert!(
        last_chunk_size <= normal_chunk_size,
        "Last chunk size {} is larger than normal chunk size {}",
        last_chunk_size,
        normal_chunk_size
    );
    let chunk_sum_size = normal_chunk_size * (chunk_count.get() as u64 - 1) + last_chunk_size;
    assert_eq!(chunk_sum_size, file_size);

    let file_information = start_large_file(&auth_data, p.bucket_id.clone(), target_filename).await;

    //a real programm might use multiple upload urls to push data in parallel, but this is just an example...
    let mut sha1sums = Vec::with_capacity(chunk_count.get() as usize);
    let mut upload_url_params = b2_get_upload_part_url(
        auth_data.api_url(),
        auth_data.authorization_token(),
        file_information.file_id().expect("No file id received"),
    )
    .await
    .expect("Could not get upload url");
    {
        for chunk_id in 1..=chunk_count.get() {
            // creating a buffer here instead of reusing the same buffer again and again seems bad...,
            // but since [Body::from] takes ownership of the vector I currently do not see an easy way around this
            let size = if chunk_id != chunk_count.get() {
                normal_chunk_size
            } else {
                last_chunk_size
            };
            let mut buf = Vec::with_capacity(size as usize);
            let mut segmet_reader = file_to_upload.take(size);
            segmet_reader
                .read_to_end(&mut buf)
                .await
                .expect("Reading input file failed");
            //give reader back
            file_to_upload = segmet_reader.into_inner();
            let sha1 = {
                let mut hasher = sha1::Sha1::new();
                hasher.update(buf.as_slice());
                hasher.digest().to_string()
            };
            sha1sums.push(sha1.clone());
            let upload_params = UploadPartParameters::builder()
                .part_number(chunk_id.try_into().unwrap())
                .content_length(size)
                .content_sha1(sha1)
                .build();
            {
                let body = Body::from(buf);

                let _part_upload_resp =
                    b2_upload_part(&mut upload_url_params, &upload_params, body)
                        .await
                        .expect("Uploading part failed");
            }
        }
    }
    let res = b2_finish_large_file(
        auth_data.api_url(),
        auth_data.authorization_token(),
        file_information.file_id().unwrap(),
        sha1sums.as_slice(),
    )
    .await;
    println!("finish large file result: {:#?}", res);
}

const DEFAULT_CHUNK_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
const MIN_NOT_LAST_CHUNK_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
fn determine_chunk_count_and_size(
    file_size: u64,
    chunk_count: &Option<NonZeroU16>,
) -> (NonZeroU16, u64) {
    match chunk_count {
        Some(cnt) => {
            if *cnt > 1.try_into().unwrap() {
                let normal_chunk_size = div_floor(file_size - 1, (cnt.get() - 1) as u64); //decrease file size by 1 to have last chunk not empty
                if normal_chunk_size < MIN_NOT_LAST_CHUNK_SIZE {
                    panic!("All but the last chunk need to be 5MB, but with the chunk count given, chunks will only be {} bytes", normal_chunk_size);
                }
                (*cnt, normal_chunk_size)
            } else {
                (1u16.try_into().unwrap(), file_size)
            }
        }
        None => {
            let default_sized_chunk_cnt = div_ceil(file_size, DEFAULT_CHUNK_SIZE);
            if default_sized_chunk_cnt > PartNumber::max_part_number() as u64 {
                //file is to large for 10 MB chunks, split into 10000 chunks
                (
                    PartNumber::max_part_number().try_into().unwrap(),
                    div_floor(file_size, PartNumber::max_part_number() as u64),
                )
            } else {
                if default_sized_chunk_cnt > 1 {
                    (
                        (default_sized_chunk_cnt as u16).try_into().unwrap(),
                        DEFAULT_CHUNK_SIZE,
                    )
                } else {
                    (1u16.try_into().unwrap(), file_size)
                }
            }
        }
    }
}

async fn start_large_file(
    auth_data: &AuthorizeAccountOk,
    bucket_id: String,
    filename: String,
) -> FileInformation {
    let params = StartLargeFileParameters::builder()
        .bucket_id(bucket_id.try_into().expect("Invalid bucket id"))
        .file_name(filename.try_into().expect("Invalid filename"))
        .build();
    b2_start_large_file(
        auth_data.api_url(),
        auth_data.authorization_token(),
        &params,
    )
    .await
    .expect("Could not start large file")
}
