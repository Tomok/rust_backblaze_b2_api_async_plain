use std::{
    convert::TryInto,
    io::{self, BufRead, Write},
};

///! This example goes through all implemented calls creating a test bucket
use backblaze_b2_async_plain::v2::*;
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
    dbg!(test_key);
}
