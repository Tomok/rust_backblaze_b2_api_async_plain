use std::convert::TryInto;

use textwrap::dedent;
use wiremock::{
    matchers::{any, header_exists, headers, method, path},
    Match,
};
use wiremock::{Mock, MockServer, ResponseTemplate};

use serde::Serialize;

pub struct B2MockServer {
    mock_server: MockServer,
}

pub const FAKE_APPLICATION_KEY_ID: &'static str = "applicationKeyId_value";
pub const FAKE_APPLICATION_KEY: &'static str = "applicationKey_value";

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
            dbg!(&h);
            match http_auth_basic::Credentials::from_header(h) {
                Ok(credentials) => {
                    dbg!(&credentials);
                    (credentials.user_id == self.username_expected)
                        && (credentials.password == self.password_expected)
                }
                Err(e) => {
                    dbg!(e);
                    false
                }
            }
        } else {
            dbg!("not found");
            // no auth header
            false
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
        let auth_key = base64::encode(format!(
            "Basic {}:{}",
            FAKE_APPLICATION_KEY_ID, FAKE_APPLICATION_KEY
        ));
        dbg!(&auth_key);
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

    pub fn uri(&self) -> String {
        self.mock_server.uri()
    }
}
