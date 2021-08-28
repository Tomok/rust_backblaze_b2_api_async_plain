use std::io::{self, BufRead, Write};

///! This example goes through all implemented calls creating a test bucket
use backblaze_b2_async_plain::v2::*;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "all_b2_calls",
    about = "Goes through all calls implemented in the libary"
)]
struct Params {
    #[structopt(short, long, env = "B2_TEST_BUCKET_NAME")]
    /// name of the test bucket to be created / deleted.
    /// WARNING if it exists, this will be deleted.
    /// by default "rust_backblaze_b2_api_async_plain_test_bucket" is used
    test_bucket_name: Option<String>,
    
    #[structopt(short, long, env = "B2_TEST_KEY_NAME")]
    /// name of the test key to be created / deleted.
    /// WARNING if it exists, this will be deleted.
    /// by default "rust_backblaze_b2_api_async_plain_test_key" is used
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

async fn delete_existing_test_keys(auth_token: &AuthorizationToken, test_key_name: &str) {
    let mut start_key = None;
    loop {
        let key_listing = b2_list_keys
    }
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();

    let test_bucket_name = p.test_bucket_name.unwrap_or_else(|| "rust_backblaze_b2_api_async_plain_test_bucket".to_owned());
    let test_bucket_key = p.test_key_name.unwrap_or_else(|| "rust_backblaze_b2_api_async_plain_test_key".to_owned());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "Please enter application key id: ").unwrap();
    let mut stdout = io::stdout();
    stdout.flush().unwrap();
    let application_key_id = readline(&stdin);

    write!(stdout, "Please enter the application key: ").unwrap();
    stdout.flush().unwrap();
    let application_key = readline(&stdin);

    let root_authorization_data = b2_authorize_account(&application_key_id, &application_key).await.expect("Authorization failed.");
    
}
