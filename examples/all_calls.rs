///! This example goes through all implemented calls creating a test bucket
use backblaze_b2_async_plain::v2::*;
use lazy_static::lazy_static;
use std::{
    convert::{TryFrom, TryInto},
    io::{self, BufRead, Write},
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "all_b2_calls",
    about = "Goes through all calls implemented in the libary"
)]
struct Params {
    #[structopt(short = "b", long, env = "B2_TEST_BUCKET_NAME")]
    /// name of the test bucket to be created / deleted.
    /// WARNING if it exists, this will be deleted.
    /// by default "rust-backblaze-b2-api-async-plain-test-bucket" is used
    test_bucket_name: Option<String>,

    #[structopt(short = "k", long, env = "B2_TEST_KEY_NAME")]
    /// name of the test key to be created / deleted.
    /// WARNING if it exists, this will be deleted.
    /// by default "rust-backblaze-b2-api-async-plain-test-key" is used
    test_key_name: Option<String>,
}

/// reads a single line, fails with error messages if that does not work
fn readline(stdin: &io::Stdin) -> String {
    let res = stdin
        .lock()
        .lines()
        .next()
        .expect("No input detected")
        .expect("Error reading input");
    println!(""); //insert line break
    res
}

async fn delete_test_keys(auth_data: &AuthorizeAccountOk, test_key_name: &str) {
    let mut start_key = None;
    loop {
        println!("Listing application keys ...");
        let list_key_params = ListKeysRequest::new(
            auth_data.account_id(),
            Some(1000u16.try_into().unwrap()), //1000 is the max number of keys requestable, without it counting like a second attempt
            start_key.as_ref(),
        );

        let key_listing = b2_list_keys(
            auth_data.api_url(),
            auth_data.authorization_token(),
            &list_key_params,
        )
        .await
        .expect("Listing Keys failed");

        for key_info in key_listing.keys() {
            if key_info.key_name() == test_key_name {
                print!("Deleting test key ... ");
                b2_delete_key(
                    auth_data.api_url(),
                    auth_data.authorization_token(),
                    key_info.application_key_id(),
                )
                .await
                .expect("Deleting Key failed");
                println!("done");
            }
        }

        if let Some(next_key) = key_listing.next_application_key_id() {
            start_key = Some(next_key.to_owned());
        } else {
            break;
        }
    }
    println!("Listing application keys ... done");
}

async fn delete_test_bucket(auth_data: &AuthorizeAccountOk, test_bucket_name: &BucketName) {
    println!("Listing test bucket ...");
    let list_bucket_params = ListBucketsRequest::builder()
        .account_id(auth_data.account_id())
        .bucket_name(test_bucket_name)
        .build();
    let buckets = b2_list_buckets(
        auth_data.api_url(),
        auth_data.authorization_token(),
        &list_bucket_params,
    )
    .await
    .expect("Listing test bucket failed");
    for bucket in buckets.buckets() {
        print!("Deleting files in test bucket ... ");
        delete_all_files_in_bucket(auth_data, bucket).await;
        println!("Done");
        print!("Deleting test bucket ... ");
        b2_delete_bucket(
            auth_data.api_url(),
            auth_data.authorization_token(),
            auth_data.account_id(),
            bucket.bucket_id(),
        )
        .await
        .expect("Could not delete test bucket");
        println!("done");
    }
    println!("Listing test bucket ... done");
}

async fn delete_all_files_in_bucket(auth_data: &AuthorizeAccountOk, bucket: &Bucket) -> () {
    let mut start_file_name = None;
    let mut start_file_id = None;
    loop {
        //loops until all files were deleted
        let file_version_request = ListFileVersionsRequest::new(
            bucket.bucket_id(),
            start_file_name.as_ref(),
            start_file_id.as_ref(),
            Some(MaxFileCount::try_from(1000).unwrap()),
            None,
            None,
        );
        let files = b2_list_file_versions(
            auth_data.api_url(),
            auth_data.authorization_token(),
            &file_version_request,
        )
        .await
        .expect("Could not list files");
        for file in files.files() {
            if let Some(file_id) = file.file_id() {
                dbg!(file.file_name());
                dbg!(file.action());
                let delete_request =
                    DeleteFileVersionRequest::new(file.file_name(), file_id, Some(true));
                b2_delete_file_version(
                    auth_data.api_url(),
                    auth_data.authorization_token(),
                    &delete_request,
                )
                .await
                .expect("Could not delete file version");
            }
        }
        if let Some(filename) = files.next_file_name() {
            start_file_name = Some(filename.to_owned());
            start_file_id = files.next_file_id().map(|f| f.clone());
        } else {
            break;
        }
    }
}

//cleanup after the test / before creating keys
async fn clean_up(
    root_authorization_data: &AuthorizeAccountOk,
    test_bucket_name: &BucketName,
    test_key_name: &str,
) {
    delete_test_keys(root_authorization_data, test_key_name).await;
    delete_test_bucket(root_authorization_data, test_bucket_name).await;
}

async fn create_test_bucket(
    root_authorization_data: &AuthorizeAccountOk,
    test_bucket_name: &BucketName,
) -> Bucket {
    print!("Creating test bucket ... ");
    let params = CreateBucketRequest::builder()
        .account_id(root_authorization_data.account_id())
        .bucket_name(test_bucket_name)
        .bucket_type(BucketType::AllPrivate)
        .build();
    let res = b2_create_bucket(
        root_authorization_data.api_url(),
        root_authorization_data.authorization_token(),
        &params,
    )
    .await
    .expect("Could not create test bucket");
    println!("done");
    res
}

async fn create_test_key(
    root_authorization_data: &AuthorizeAccountOk,
    test_bucket: &Bucket,
    test_key_name: &str,
) -> CreatedKeyInformation {
    print!("Creating test key ... ");
    let capabilities = all_per_bucket_capabilites();
    let params = CreateKeyRequest::builder()
        .account_id(root_authorization_data.account_id())
        .capabilities(&capabilities)
        .key_name(test_key_name)
        .valid_duration_in_seconds(60 * 60) //one hour should be more than sufficient to run all these steps
        .bucket_id(test_bucket.bucket_id())
        .build();
    let res = b2_create_key(
        root_authorization_data.api_url(),
        root_authorization_data.authorization_token(),
        &params,
    )
    .await
    .expect("Creating test key failed");
    println!("done");
    res
}

fn sha1sum(data: &[u8]) -> String {
    let mut h = sha1::Sha1::new();
    h.update(data);
    h.digest().to_string()
}

const UPLOAD_FILE_CONTENTS: &'static [u8] = &[42u8; 4096];
lazy_static! {
    static ref UPLOAD_FILE_NAME: FileName = "UploadedFile".to_owned().try_into().unwrap();
    static ref UPLOAD_FILE_CONTENTS_SHA1: String = sha1sum(UPLOAD_FILE_CONTENTS);
}

async fn upload_file(test_key_auth: &AuthorizeAccountOk, test_bucket: &Bucket) -> FileInformation {
    print!("Uploading test file ... ");
    let mut upload_params = b2_get_upload_url(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        test_bucket.bucket_id(),
    )
    .await
    .expect("Could not get upload url");

    let upload_file_params = UploadFileParameters::builder()
        .file_name(&UPLOAD_FILE_NAME)
        .content_length(UPLOAD_FILE_CONTENTS.len() as u64)
        .content_sha1(&UPLOAD_FILE_CONTENTS_SHA1)
        .build();

    let res = b2_upload_file(
        &mut upload_params,
        &upload_file_params,
        UPLOAD_FILE_CONTENTS,
    )
    .await
    .expect("Uploading test file failed");
    println!("done");
    res
}

lazy_static! {
    static ref LARGE_UPLOAD_FILE_NAME: FileName =
        "UploadedLargeFile".to_owned().try_into().unwrap();
}
const LARGE_FILE_PART1_SIZE: usize = 5 * 1024 * 1024;
/// builds a large file using various calls related to parted file upload, copy,...
async fn build_large_file(
    test_key_auth: &AuthorizeAccountOk,
    test_bucket: &Bucket,
    test_copy_source_file: &FileInformation,
) -> FileInformation {
    println!("Building a file from parts ... ");
    let file = {
        let params = StartLargeFileParameters::builder()
            .bucket_id(test_bucket.bucket_id())
            .file_name(&LARGE_UPLOAD_FILE_NAME)
            .build();
        b2_start_large_file(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &params,
        )
        .await
        .expect("Could not start large file")
    };
    let file_id = file
        .file_id()
        .expect("Created large file did not have a file id");
    let mut upload_url = b2_get_upload_part_url(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        file_id,
    )
    .await
    .expect("Could not get file upload parts url");

    let sha1_part1: String = {
        let upload_data = Vec::from([0u8; LARGE_FILE_PART1_SIZE]);
        let sha1 = sha1sum(upload_data.as_ref());
        let params = UploadPartParameters::builder()
            .part_number(1.try_into().unwrap())
            .content_length(upload_data.len() as u64)
            .content_sha1(&sha1)
            .build();

        b2_upload_part(&mut upload_url, &params, upload_data)
            .await
            .expect("Uploading first part failed");
        sha1
    };
    {
        let source_file_id = test_copy_source_file
            .file_id()
            .expect("Copy source file had no id");
        let params = CopyPartRequest::builder()
            .source_file_id(source_file_id)
            .large_file_id(file_id)
            .part_number(2.try_into().unwrap())
            .build();
        b2_copy_part(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &params,
        )
        .await
        .expect("Could not copy parts");
    }

    //validate that partial file exists
    {
        let params = ListUnfinishedLargeFilesRequest::builder()
            .bucket_id(test_bucket.bucket_id())
            .build();
        let unfinished_files = b2_list_unfinished_large_files(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &params,
        )
        .await
        .expect("Could not list unfinished files");
        assert_eq!(&None, unfinished_files.next_file_name(), "although only one unfinished file should exist, unfinished files returned a next file name");
        assert_eq!(
            1,
            unfinished_files.files().len(),
            "Only one unfinished file should exist"
        );
        assert_eq!(
            &*LARGE_UPLOAD_FILE_NAME,
            unfinished_files.files()[0].file_name(),
            "Unfinished file did not have the expected name"
        );
    }
    // validate the partial file has the right parts
    {
        let params = ListPartsRequest::builder().file_id(file_id).build();
        let parts = b2_list_parts(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &params,
        )
        .await
        .expect("Could not list file parts");
        dbg!(&parts);
        assert_eq!(
            None,
            parts.next_part_number(),
            "Only two parts should exit, but response contained a next_part_number"
        );
        assert_eq!(2, parts.parts().len(), "Partial file should have two parts");

        assert_eq!(
            LARGE_FILE_PART1_SIZE as u64,
            *parts.parts()[0].content_length()
        );
        assert_eq!(&sha1_part1, parts.parts()[0].content_sha1());

        assert_eq!(
            UPLOAD_FILE_CONTENTS.len() as u64,
            *parts.parts()[1].content_length()
        );
        assert_eq!(&*UPLOAD_FILE_CONTENTS_SHA1, parts.parts()[1].content_sha1());
    }

    let sha1_part2_ref: &String = &UPLOAD_FILE_CONTENTS_SHA1;
    let res = b2_finish_large_file(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        file_id,
        &[&sha1_part1, sha1_part2_ref],
    )
    .await
    .expect("Finishing large file failed");
    println!("Building a file from parts ...  Done");
    res
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();

    let test_bucket_name: BucketName = p
        .test_bucket_name
        .unwrap_or_else(|| "rust-backblaze-b2-api-async-plain-test-bucket".to_owned())
        .try_into()
        .unwrap();
    let test_key_name = p
        .test_key_name
        .unwrap_or_else(|| "rust-backblaze-b2-api-async-plain-test-key".to_owned());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "Please enter application key id: ").unwrap();
    let mut stdout = io::stdout();
    stdout.flush().unwrap();
    let application_key_id = readline(&stdin);

    write!(stdout, "Please enter the application key: ").unwrap();
    stdout.flush().unwrap();
    let application_key = readline(&stdin);

    let root_authorization_data = b2_authorize_account(&application_key_id, &application_key)
        .await
        .expect("Authorization failed.");

    clean_up(&root_authorization_data, &test_bucket_name, &test_key_name).await;

    let test_bucket = create_test_bucket(&root_authorization_data, &test_bucket_name).await;
    let test_key = create_test_key(&root_authorization_data, &test_bucket, &test_key_name).await;

    let test_key_auth =
        b2_authorize_account(test_key.application_key_id(), test_key.application_key())
            .await
            .expect("Could not login with test key");

    let uploaded_file = upload_file(&test_key_auth, &test_bucket).await;
    let large_uploaded_file = build_large_file(&test_key_auth, &test_bucket, &uploaded_file).await;
    dbg!(large_uploaded_file);
}
