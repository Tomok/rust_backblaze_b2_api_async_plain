///! This example goes through all implemented calls creating a test bucket
use backblaze_b2_async_plain::v2::*;
use headers::CacheControl;
use lazy_static::lazy_static;
use std::{
    convert::{TryFrom, TryInto},
    io::{self, BufRead, Write},
    time::Duration,
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

    /// application key id, will be requested from stdin if not given
    application_key_id: Option<String>,
}

/// reads a single line, fails with error messages if that does not work
fn readline(stdin: &io::Stdin) -> String {
    let res = stdin
        .lock()
        .lines()
        .next()
        .expect("No input detected")
        .expect("Error reading input");
    println!(); //insert line break
    res
}

async fn delete_test_keys(auth_data: &AuthorizeAccountOk, test_key_name: &KeyName) {
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

async fn delete_all_files_in_bucket(auth_data: &AuthorizeAccountOk, bucket: &Bucket) {
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
            start_file_id = files.next_file_id().cloned();
        } else {
            break;
        }
    }
}

//cleanup after the test / before creating keys
async fn clean_up<'a>(
    root_authorization_data: &'a AuthorizeAccountOk,
    test_bucket_name: &'a BucketName,
    test_key_name: KeyNameRef<'a>,
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
        .file_lock_enabled(true) // used for [update_file_legal_hold]
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
    test_key_name: &KeyName,
) -> CreatedKeyInformation {
    print!("Creating test key ... ");
    let capabilities = all_per_bucket_capabilites();
    let params = CreateKeyRequest::builder()
        .account_id(root_authorization_data.account_id())
        .capabilities(&capabilities)
        .key_name(test_key_name)
        .valid_duration_in_seconds(Duration::from_secs(60 * 60).try_into().unwrap()) //one hour should be more than sufficient to run all these steps
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

fn sha1sum(data: &[u8]) -> Sha1Digest {
    let mut h = sha1::Sha1::new();
    h.update(data);
    h.digest().into()
}

const UPLOAD_FILE_CONTENTS: &[u8] = &[42u8; 4096];
// output key of `openssl enc -nosalt -aes-256-cbc -k hello-aes -P` .. use a save key in real usecases
const CUSTOMER_KEY_FOR_SSE_C: [u8; 32] = [
    0xE8, 0xB6, 0xC0, 0x0C, 0x9A, 0xDC, 0x5E, 0x75, 0xBB, 0x65, 0x6E, 0xCD, 0x42, 0x9C, 0xB1, 0x64,
    0x3A, 0x25, 0xB1, 0x11, 0xFC, 0xD2, 0x2C, 0x66, 0x22, 0xD5, 0x3E, 0x07, 0x22, 0x43, 0x99, 0x93,
];
lazy_static! {
    static ref UPLOAD_FILE_NAME: FileName = "UploadedFile".to_owned().try_into().unwrap();
    static ref UPLOAD_FILE_NAME_SSEC: FileName = "UploadedFileSSEC".to_owned().try_into().unwrap();
    static ref COPY_FILE_NAME: FileName = "CopiedFile".to_owned().try_into().unwrap();
    static ref COPY_FILE_NAME_SSEC: FileName = "CopiedFileSSEC".to_owned().try_into().unwrap();
    static ref UPLOAD_FILE_CONTENTS_SHA1: Sha1Digest = sha1sum(UPLOAD_FILE_CONTENTS);
    static ref CUSTOMER_KEY_FOR_SSE_C_MD5: Md5Digest = md5::compute(CUSTOMER_KEY_FOR_SSE_C).into();
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

async fn start_large_file(
    test_key_auth: &AuthorizeAccountOk,
    test_bucket: &Bucket,
) -> FileInformation {
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
}

/// builds a large file using various calls related to parted file upload, copy,...
async fn build_large_file(
    test_key_auth: &AuthorizeAccountOk,
    test_bucket: &Bucket,
    test_copy_source_file: &FileInformation,
) -> FileInformation {
    println!("Building a file from parts ... ");
    let file = start_large_file(test_key_auth, test_bucket).await;
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

    let sha1_part1: Sha1Digest = {
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

    let sha1_part2_ref: Sha1DigestRef = &UPLOAD_FILE_CONTENTS_SHA1;
    let res = b2_finish_large_file(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        file_id,
        &[&sha1_part1, sha1_part2_ref],
    )
    .await
    .expect("Finishing large file failed");
    println!("Done");
    res
}

async fn cancel_large_file(test_key_auth: &AuthorizeAccountOk, test_bucket: &Bucket) {
    print!("Starting large file ...");
    let file = start_large_file(test_key_auth, test_bucket).await;
    print!(" cancelling that file ... ");
    b2_cancel_large_file(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        file.file_id()
            .expect("Started large file did not have a FileId"),
    )
    .await
    .expect("Could not cancel file");
    println!("done");
}

async fn copy_file(
    test_key_auth: &AuthorizeAccountOk,
    source_file: &FileInformation,
    test_bucket: &Bucket,
) -> FileInformation {
    print!("Copying file ... ");
    let params = CopyFileRequest::builder()
        .source_file_id(
            source_file
                .file_id()
                .expect("Source File did not have an FileId"),
        )
        .destination_bucket_id(test_bucket.bucket_id())
        .file_name(&*COPY_FILE_NAME)
        .build();
    let res = b2_copy_file(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        &params,
    )
    .await
    .expect("Could not copy file");
    println!("done");
    res
}

async fn hide_file(test_key_auth: &AuthorizeAccountOk, copied_file: &FileInformation) {
    print!("Hiding file ... ");
    b2_hide_file(
        test_key_auth.api_url(),
        test_key_auth.authorization_token(),
        copied_file.bucket_id(),
        copied_file.file_name(),
    )
    .await
    .expect("Could not hide file");
    println!("done");
}

/// downloads part2 of the large file (the one that is copied in [build_large_file])
async fn download_file_by_id(
    test_key_auth: &AuthorizeAccountOk,
    large_uploaded_file: &FileInformation,
) {
    let part2_range = headers::Range::bytes(
        LARGE_FILE_PART1_SIZE as u64..(LARGE_FILE_PART1_SIZE + UPLOAD_FILE_CONTENTS.len()) as u64,
    )
    .unwrap();
    let content_type = ContentType::html(); //"text/html"
    let cache_control = CacheControl::new().with_no_cache();
    let content_language = ContentLanguage::try_from("en-US").unwrap();
    let params = DownloadParams::builder()
        .file_id(
            large_uploaded_file
                .file_id()
                .expect("Large file did not have a file id"),
        )
        .range(&part2_range)
        .b2_cache_control(&cache_control)
        .b2_content_type(&content_type)
        .b2_content_language(&content_language)
        .build();
    let resp = b2_download_file_by_id(
        test_key_auth.download_url(),
        Some(test_key_auth.authorization_token()),
        &params,
    )
    .await
    .expect("Downloading file by id failed");
    let received_content_type = resp
        .headers()
        .get("Content-Type")
        .expect("Content-Type header not received, even though it was sent");
    assert_eq!("text/html", received_content_type);
    let received_content_language = resp
        .headers()
        .get("Content-Language")
        .expect("Content-Languange header not received, even though it was sent");
    assert_eq!("en-US", received_content_language);

    let received_cache_control = resp
        .headers()
        .get("Cache-Control")
        .expect("Cache Control header missing");
    assert_eq!("no-cache", received_cache_control);
    let data = resp
        .bytes()
        .await
        .expect("Could not get bytes for downloaded file");
    assert_eq!(UPLOAD_FILE_CONTENTS, data);
}
/// downloads part2 of the large file (the one that is copied in [build_large_file])
async fn download_file_by_name(
    test_key_auth: &AuthorizeAccountOk,
    test_bucket: &Bucket,
    large_uploaded_file: &FileInformation,
) {
    print!("Creating Download Authorization ... ");
    let download_auth = {
        let file_name_prefix: FileNamePrefix = "".to_string().try_into().unwrap();
        let req = GetDownloadAuthorizationRequest::builder()
            .bucket_id(test_bucket.bucket_id())
            .file_name_prefix(&file_name_prefix)
            .valid_duration_in_seconds(600.try_into().unwrap())
            .build();
        b2_get_download_authorization(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &req,
        )
        .await
        .expect("Could not get download authorization")
    };
    println!("done");

    print!("Downloading file by name ... ");
    {
        let part2_range = headers::Range::bytes(
            LARGE_FILE_PART1_SIZE as u64
                ..(LARGE_FILE_PART1_SIZE + UPLOAD_FILE_CONTENTS.len()) as u64,
        )
        .unwrap();

        let req = DownloadFileByNameRequest::builder()
            .bucket_name(test_bucket.bucket_name())
            .file_name(large_uploaded_file.file_name())
            .range(&part2_range)
            .authorization(download_auth.authorization_token())
            .build();
        let resp = b2_download_file_by_name(test_key_auth.download_url(), &req)
            .await
            .expect("Downloading file by name failed");

        let data = resp
            .bytes()
            .await
            .expect("Could not get bytes for downloaded file");
        assert_eq!(UPLOAD_FILE_CONTENTS, data);
    }

    println!("done")
}

/// sets and unsets file legal hold
async fn update_file_legal_hold(test_key_auth: &AuthorizeAccountOk, file: &FileInformation) {
    let file_id = file.file_id().expect("File did not have a fileId");
    print!("Turning file legal hold on ... ");
    {
        let request = UpdateFileLegalHoldRequest::builder()
            .file_name(file.file_name())
            .file_id(file_id)
            .legal_hold(LegalHoldOnOff::On)
            .build();
        b2_update_file_legal_hold(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect("Turning legal hold on failed");
    }
    println!("done");
    print!("Triyng to delete file ...");
    {
        let request = DeleteFileVersionRequest::builder()
            .file_name(file.file_name())
            .file_id(file_id)
            .build();
        let err = b2_delete_file_version(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect_err("Attempt to delete locked file did not fail");
        assert!(
            matches!(err, errors::DeleteFileVersionError::AccessDenied { .. }),
            "Expected AccessDenied error, but got: {:#?}",
            err
        );
    }
    println!("failed as expected - done");

    print!("Turning file legal hold off ... ");
    {
        let request = UpdateFileLegalHoldRequest::builder()
            .file_name(file.file_name())
            .file_id(file_id)
            .legal_hold(LegalHoldOnOff::Off)
            .build();
        b2_update_file_legal_hold(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect("Turning legal hold off failed");
    }
    println!("done");
}

const FILE_RETENTION_DURATION: TimeStamp = 1000 * 60 * 60; //1h after upload
/// sets and unsets file legal hold
async fn update_file_retention(test_key_auth: &AuthorizeAccountOk, file: &FileInformation) {
    let file_id = file.file_id().expect("File did not have a fileId");
    print!("Setting file retention to gouvernance ...");
    {
        let new_retention = FileRetention::new(
            FileRetentionMode::Governance,
            file.upload_timestamp() + FILE_RETENTION_DURATION,
        );
        let request = UpdateFileRetentionRequest::builder()
            .file_id(file_id)
            .file_name(file.file_name())
            .file_retention(&new_retention)
            .build();
        let resp = b2_update_file_retention(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect("Setting file retention failed");
        assert_eq!(file.file_name(), resp.file_name());
        assert_eq!(&new_retention, resp.file_retention());
    }
    println!("done");
    print!("Triyng to delete file ...");
    {
        let request = DeleteFileVersionRequest::builder()
            .file_name(file.file_name())
            .file_id(file_id)
            .build();
        let err = b2_delete_file_version(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect_err("Attempt to delete goverened file did not fail");
        assert!(
            matches!(err, errors::DeleteFileVersionError::AccessDenied { .. }),
            "Expected AccessDenied error, but got: {:#?}",
            err
        );
    }
    println!("failed as expected - done");
    print!("Setting file retention to None ...");
    {
        let new_retention = FileRetention::disabled();
        let request = UpdateFileRetentionRequest::builder()
            .file_id(file_id)
            .file_name(file.file_name())
            .file_retention(&new_retention)
            .bypass_governance(true)
            .build();
        let resp = b2_update_file_retention(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect("Setting file retention failed");
        assert_eq!(file.file_name(), resp.file_name());
        assert_eq!(&new_retention, resp.file_retention());
    }
    println!("done");
}

// does not work with the test_key as that one may not write_buckets
async fn update_bucket_life_cycle_rules(main_key_auth: &AuthorizeAccountOk, bucket: &Bucket) {
    print!("Updating bucket life cycle rules ...");
    let life_cycle_rule = LifeCycleRule::builder()
        .days_from_hiding_to_deleting(1.try_into().unwrap())
        .file_name_prefix("".to_owned().try_into().unwrap())
        .build();
    let life_cycle_rules = vec![life_cycle_rule];
    let request = UpdateBucketRequest::builder()
        .account_id(bucket.account_id())
        .bucket_id(bucket.bucket_id())
        .lifecycle_rules(&life_cycle_rules)
        .build();
    let resp = b2_update_bucket(
        main_key_auth.api_url(),
        main_key_auth.authorization_token(),
        &request,
    )
    .await
    .expect("Could not update bucket");
    assert_eq!(bucket.bucket_id(), resp.bucket_id());
    assert_eq!(life_cycle_rules.as_slice(), resp.lifecycle_rules());
    println!("done");
}

async fn server_side_encryption_c_calls(test_key_auth: &AuthorizeAccountOk, test_bucket: &Bucket) {
    let server_side_encryption = ServerSideEncryptionCustomerKey::SseC {
        customer_key: &CUSTOMER_KEY_FOR_SSE_C,
        customer_key_md5: &CUSTOMER_KEY_FOR_SSE_C_MD5,
    };

    print!("Uploading test file with SSE-C... ");
    let uploaded_file = {
        let mut upload_params = b2_get_upload_url(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            test_bucket.bucket_id(),
        )
        .await
        .expect("Could not get upload url");

        let upload_file_params = UploadFileParameters::builder()
            .file_name(&UPLOAD_FILE_NAME_SSEC)
            .content_length(UPLOAD_FILE_CONTENTS.len() as u64)
            .content_sha1(&UPLOAD_FILE_CONTENTS_SHA1)
            .server_side_encryption(&server_side_encryption)
            .build();

        b2_upload_file(
            &mut upload_params,
            &upload_file_params,
            UPLOAD_FILE_CONTENTS,
        )
        .await
        .expect("Uploading test file with SSE-C failed")
    };
    let uploaded_file_id = uploaded_file
        .file_id()
        .expect("File uploaded with SSE-C did not have a file id");
    println!("done");

    print!("Copying file with SSE-C ...");
    let copied_file = {
        let parameters = CopyFileRequest::builder()
            .source_file_id(uploaded_file_id)
            .file_name(&COPY_FILE_NAME_SSEC)
            .source_server_side_encryption(&server_side_encryption)
            .destination_server_side_encryption(&server_side_encryption)
            .build();
        b2_copy_file(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &parameters,
        )
        .await
        .expect("Copiing file with SSE-C failed")
    };
    println!("done");

    print!("Checking file with SSE-C ...");
    {
        let download_params = DownloadParams::builder()
            .file_id(
                copied_file
                    .file_id()
                    .expect("File copied with SSE-C did not have a file_id"),
            )
            .server_side_encryption(&server_side_encryption)
            .build();
        let download = b2_download_file_by_id(
            test_key_auth.download_url(),
            Some(test_key_auth.authorization_token()),
            &download_params,
        )
        .await
        .expect("Downloading file with SSE-C failed");
        assert_eq!(
            UPLOAD_FILE_CONTENTS,
            download
                .bytes()
                .await
                .expect("Getting data of file with SSE-C failed")
        );
    }
    println!("done");
    print!("Checking named file download with SSE-C ...");
    {
        let download_params = DownloadFileByNameRequest::builder()
            .bucket_name(test_bucket.bucket_name())
            .file_name(copied_file.file_name())
            .authorization(test_key_auth.authorization_token())
            .server_side_encryption(&server_side_encryption)
            .build();
        let download = b2_download_file_by_name(test_key_auth.download_url(), &download_params)
            .await
            .expect("Downloading file by name for SSE-C file failed");
        assert_eq!(
            UPLOAD_FILE_CONTENTS,
            download
                .bytes()
                .await
                .expect("Getting data of file with SSE-C failed")
        );
    }
    println!("done");
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
    let test_key_name: KeyName = p
        .test_key_name
        .unwrap_or_else(|| "rust-backblaze-b2-api-async-plain-test-key".to_owned())
        .try_into()
        .expect("Invalid test key name");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let application_key_id_string = if let Some(key_id) = p.application_key_id {
        key_id
    } else {
        write!(stdout, "Please enter application key id: ").unwrap();
        let mut stdout = io::stdout();
        stdout.flush().unwrap();
        readline(&stdin)
    };
    let application_key_id = application_key_id_string
        .try_into()
        .expect("Invalid application key id");
    write!(stdout, "Please enter the application key: ").unwrap();
    stdout.flush().unwrap();
    let application_key_string = readline(&stdin);
    let application_key = application_key_string
        .try_into()
        .expect("Invalid application key");

    let root_authorization_data = b2_authorize_account(&application_key_id, &application_key)
        .await
        .expect("Authorization failed.");

    clean_up(&root_authorization_data, &test_bucket_name, &test_key_name).await;

    let test_bucket = create_test_bucket(&root_authorization_data, &test_bucket_name).await;
    update_bucket_life_cycle_rules(&root_authorization_data, &test_bucket).await;

    let test_key = create_test_key(&root_authorization_data, &test_bucket, &test_key_name).await;

    let test_key_auth =
        b2_authorize_account(test_key.application_key_id(), test_key.application_key())
            .await
            .expect("Could not login with test key");

    let uploaded_file = upload_file(&test_key_auth, &test_bucket).await;
    let large_uploaded_file = build_large_file(&test_key_auth, &test_bucket, &uploaded_file).await;

    download_file_by_id(&test_key_auth, &large_uploaded_file).await;
    download_file_by_name(&test_key_auth, &test_bucket, &large_uploaded_file).await;

    cancel_large_file(&test_key_auth, &test_bucket).await;
    let copied_file = copy_file(&test_key_auth, &uploaded_file, &test_bucket).await;
    {
        let copied_file_info = b2_get_file_info(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            copied_file
                .file_id()
                .expect("Copied file did not have a FileId"),
        )
        .await
        .expect("Getting file information failed");
        assert_eq!(copied_file.file_name(), copied_file_info.file_name());
    }
    hide_file(&test_key_auth, &copied_file).await;

    update_file_legal_hold(&test_key_auth, &uploaded_file).await;
    update_file_retention(&test_key_auth, &uploaded_file).await;

    server_side_encryption_c_calls(&test_key_auth, &test_bucket).await;

    print!("Listing file names ... ");
    {
        let request = ListFileNamesRequest::builder()
            .bucket_id(test_bucket.bucket_id())
            .build();
        let resp = b2_list_file_names(
            test_key_auth.api_url(),
            test_key_auth.authorization_token(),
            &request,
        )
        .await
        .expect("Listing file names failed");
        assert_eq!(&None, resp.next_file_name(), "Test bucket contained so many files, that paging needed to be used ... but those were not created");
        assert!(
            resp.files().len() >= 2,
            "Expected more than two files, but file_list was \n {:#?}",
            resp
        );
    }
    println!("done");
}
