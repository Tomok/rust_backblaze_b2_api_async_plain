use std::convert::TryInto;

use textwrap::dedent;
use wiremock::{
    matchers::{header, header_exists, method, path},
    Match,
};
use wiremock::{Mock, MockServer, ResponseTemplate};

use serde::Serialize;
use serde_json::json;

pub struct B2MockServer {
    mock_server: MockServer,
}

pub const FAKE_APPLICATION_KEY_ID: &str = "applicationKeyId_value";
pub const FAKE_APPLICATION_KEY: &str = "applicationKey_value";
pub const FAKE_AUTHORIZATION_TOKEN: &str = "authorization_token";
pub const FAKE_ACCOUNT_ID: &str = "a30f20426f0b1";
pub const FAKE_BUCKET_ID: &str = "b2f6f21365e1d29f6c580f18";

struct AuthorizationHeaderMatch {
    username_expected: String,
    password_expected: String,
}

impl AuthorizationHeaderMatch {
    fn new(username_expected: String, password_expected: String) -> Self {
        Self {
            username_expected,
            password_expected,
        }
    }
}

impl Match for AuthorizationHeaderMatch {
    fn matches(&self, request: &wiremock::Request) -> bool {
        if let Some(auth_header) = request.headers.get(&"authorization".try_into().unwrap()) {
            let h = auth_header.get(0).unwrap().to_string();
            match http_auth_basic::Credentials::from_header(h) {
                Ok(credentials) => {
                    (credentials.user_id == self.username_expected)
                        && (credentials.password == self.password_expected)
                }
                Err(_e) => false,
            }
        } else {
            // no auth header
            false
        }
    }
}

struct JsonBodyMatch {
    json_obj: serde_json::Value,
}

impl JsonBodyMatch {
    fn new(json_obj: serde_json::Value) -> Self {
        Self { json_obj }
    }
}

impl Match for JsonBodyMatch {
    fn matches(&self, request: &wiremock::Request) -> bool {
        if let Ok(body) = String::from_utf8(request.body.clone()) {
            if let Ok(input) = serde_json::from_str::<serde_json::Value>(&body) {
                let res = input == self.json_obj;
                if !res {
                    println!(
                        "input != expected, input was {:#?} \n expected was {:#?}",
                        input, self.json_obj
                    );
                }
                res
            } else {
                false //not JSON
            }
        } else {
            false // not valid utf-8
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorStructure<'a> {
    status: http_types::StatusCode,
    code: &'a str,
    message: &'a str,
}

impl<'a> ErrorStructure<'a> {
    fn new<S>(status: S, code: &'a str, message: &'a str) -> Self
    where
        S: TryInto<http_types::StatusCode>,
        <S as TryInto<http_types::StatusCode>>::Error: std::fmt::Debug,
    {
        let status_code = status
            .try_into()
            .expect("Failed to convert into status code.");
        Self {
            status: status_code,
            code,
            message,
        }
    }
}

fn error_reponse<S>(status: S, code: &str, message: &str) -> ResponseTemplate
where
    S: TryInto<http_types::StatusCode>,
    <S as TryInto<http_types::StatusCode>>::Error: std::fmt::Debug,
{
    let status_code = status
        .try_into()
        .expect("Failed to convert into status code.");
    let error_obj = ErrorStructure::new(status_code, code, message);
    ResponseTemplate::new(status_code).set_body_json(error_obj)
}

impl B2MockServer {
    pub async fn start() -> Self {
        let mock_server = MockServer::start().await;

        Self { mock_server }
    }

    pub async fn register_default_auth_handler(&self) {
        let ok_body = dedent("
        {
            \"absoluteMinimumPartSize\": 5000000,
            \"accountId\": \"YOUR_ACCOUNT_ID\",
            \"allowed\": {
              \"bucketId\": \"BUCKET_ID\",
              \"bucketName\": \"BUCKET_NAME\",
              \"capabilities\": [
                \"listBuckets\",
                \"listFiles\",
                \"readFiles\",
                \"shareFiles\",
                \"writeFiles\",
                \"deleteFiles\"
              ],
              \"namePrefix\": null
            },
            \"apiUrl\": \"https://apiNNN.backblazeb2.com\",
            \"authorizationToken\": \"4_0022623512fc8f80000000001_0186e431_d18d02_acct_tH7VW03boebOXayIc43-sxptpfA=\",
            \"downloadUrl\": \"https://f002.backblazeb2.com\",
            \"recommendedPartSize\": 100000000,
            \"s3ApiUrl\": \"https://s3.us-west-NNN.backblazeb2.com\"
        }");
        Mock::given(method("GET"))
            .and(path("/b2api/v2/b2_authorize_account"))
            .and(AuthorizationHeaderMatch::new(
                FAKE_APPLICATION_KEY_ID.into(),
                FAKE_APPLICATION_KEY.into(),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_raw(ok_body, "application/json"))
            .mount(&self.mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/b2api/v2/b2_authorize_account"))
            .and(header_exists("authorization"))
            .respond_with(error_reponse(
                401,
                "unauthorized",
                "The applicationKeyId and/or the applicationKey are wrong.",
            ))
            .mount(&self.mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/b2api/v2/b2_authorize_account"))
            .respond_with(error_reponse(
                400,
                "bad_request",
                "No authorization header found",
            ))
            .mount(&self.mock_server)
            .await
    }

    /// creates a default list bucket handler, that responds to authorized requests using [FAKE_ACCOUNT_ID] and [FAKE_AUTHORIZATION_TOKEN],
    /// Note: no other fields may be part of the request body
    pub async fn register_default_list_bucket_handler(&self) {
        let ok_body = dedent(
            "
        {
            \"buckets\": [
            {
                \"accountId\": \"30f20426f0b1\",
                \"bucketId\": \"4a48fe8875c6214145260818\",
                \"bucketInfo\": {},
                \"bucketName\" : \"Kitten-Videos\",
                \"bucketType\": \"allPrivate\",
                \"defaultServerSideEncryption\": {
                  \"isClientAuthorizedToRead\" : true,
                  \"value\": {
                    \"algorithm\" : \"AES256\",
                    \"mode\" : \"SSE-B2\"
                  }
                },
                \"fileLockConfiguration\": {
                    \"isClientAuthorizedToRead\": true,
                    \"value\": {
                      \"defaultRetention\": {
                        \"mode\": null,
                        \"period\": null
                      },
                      \"isFileLockEnabled\": true
                    }
                },
                \"lifecycleRules\": [],
                \"options\": [],
                \"revision\": 3
                
            },
            {
                \"accountId\": \"30f20426f0b1\",
                \"bucketId\" : \"5b232e8875c6214145260818\",
                \"bucketInfo\": {},
                \"bucketName\": \"Puppy-Videos\",
                \"bucketType\": \"allPublic\",
            \"defaultServerSideEncryption\": {
                  \"isClientAuthorizedToRead\" : true,
                  \"value\": {
                    \"algorithm\" : null,
                    \"mode\" : null
                  }
                },
                \"fileLockConfiguration\": {
                    \"isClientAuthorizedToRead\": true,
                    \"value\": {
                      \"defaultRetention\": {
                        \"mode\": null,
                        \"period\": null
                      },
                      \"isFileLockEnabled\": true
                    }
                },
                \"lifecycleRules\": [],
                \"options\": [],
                \"revision\": 32222222
            },
            {
                \"accountId\": \"30f20426f0b1\",
                \"bucketId\": \"87ba238875c6214145260818\",
                \"bucketInfo\": {},
                \"bucketName\": \"Vacation-Pictures\",
                \"bucketType\" : \"allPrivate\",
            \"defaultServerSideEncryption\": {
                  \"isClientAuthorizedToRead\" : true,
                  \"value\": {
                    \"algorithm\" : null,
                    \"mode\" : null
                  }
                },
                \"fileLockConfiguration\": {
                    \"isClientAuthorizedToRead\": false,
                    \"value\":  null
                },
                \"lifecycleRules\": [],
                \"options\": [],
                \"revision\": 2
            } ]
        }",
        );
        println!("ok_body = {}", &ok_body);
        let expected_input = json!({
            "accountId": "a30f20426f0b1"
        });
        Mock::given(method("POST"))
            .and(path("/b2api/v2/b2_list_buckets"))
            .and(header("Authorization", FAKE_AUTHORIZATION_TOKEN))
            .and(JsonBodyMatch::new(expected_input))
            .respond_with(ResponseTemplate::new(200).set_body_raw(ok_body, "application/json"))
            .mount(&self.mock_server)
            .await;
    }

    pub async fn register_default_list_file_names_handler(&self) {
        let ok_obj = json!({
          "files": [
            {
              "accountId": "ACCOUNT_ID",
              "action": "upload",
              "bucketId": "b2f6f21365e1d29f6c580f18",
              "contentLength": 7,
              "contentSha1": "dc724af18fbdd4e59189f5fe768a5f8311527050",
              "contentType": "text/plain",
              "fileId": "4_zb2f6f21365e1d29f6c580f18_f10904e5ca06493a1_d20180914_m223119_c002_v0001094_t0002",
              "fileInfo": {
                "src_last_modified_millis": "1536964184056"
              },
              "fileName": "testing.txt",
              "file_retention": {
                "isClientAuthorizedToRead": true,
                "value": {
                    "mode": null,
                    "retainUntilTimestamp": null,
                },
              },
              "legal_hold": {
                "isClientAuthorizedToRead": true,
                "value": null,
              },
              "serverSideEncryption": {
                "algorithm": "AES256",
                "mode": "SSE-C"
              },
              "uploadTimestamp": 1536964279000u64
            },
            {
              "accountId": "ACCOUNT_ID",
              "action": "upload",
              "bucketId": "b2f6f21365e1d29f6c580f18",
              "contentLength": 8,
              "contentSha1": "596b29ec9afea9e461a20610d150939b9c399d93",
              "contentType": "text/plain",
              "fileId": "4_zb2f6f21365e1d29f6c580f18_f10076875fe98d4af_d20180914_m223128_c002_v0001108_t0050",
              "fileInfo": {
                "src_last_modified_millis": "1536964200750"
              },
              "fileName": "testing2.txt",
              "serverSideEncryption": {
                "algorithm": "AES256",
                "mode": "SSE-B2"
              },
              "uploadTimestamp": 1536964288000u64
            }
          ],
          "nextFileName": null
        });
        println!("ok_body = {:#?}", &ok_obj);
        let expected_input = json!({
            "bucketId": "b2f6f21365e1d29f6c580f18",
        });
        Mock::given(method("POST"))
            .and(path("/b2api/v2/b2_list_file_names"))
            .and(header("Authorization", FAKE_AUTHORIZATION_TOKEN))
            .and(JsonBodyMatch::new(expected_input))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_obj))
            .mount(&self.mock_server)
            .await;
    }

    pub fn uri(&self) -> String {
        self.mock_server.uri()
    }
}
