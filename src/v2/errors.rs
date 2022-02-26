use super::JsonErrorObj;

macro_rules! error_enum{
    ($enum_name:ident {
        $(($variant_code:literal, $variant_text:literal, $variant_name:ident)),* $(,)?
    }) => {
        #[derive(Debug)]
        pub enum $enum_name {
            $($variant_name {
                raw_error: crate::v2::JsonErrorObj,
                /// Retry delay in seconds as returned by the retry-after header, if any
                retry: Option<usize>,
            },)*
            /// Error reported from reqwest & not in the data of the request or response
            RequestError{error: reqwest::Error},
            Unexpected {
                raw_error: crate::v2::Error,
            },
        }

        impl $enum_name {
            fn from_json_error_obj(raw_error: crate::v2::JsonErrorObj, retry: Option<usize>) -> Self {
                match (raw_error.status.as_u16(), raw_error.code.as_str()) {
                    $(($variant_code, $variant_text) => Self::$variant_name { raw_error, retry },)*
                    _ => Self::Unexpected {
                        raw_error: crate::v2::Error::JsonError(raw_error),
                    },
                }
            }

            fn header_to_usize(header: &headers::HeaderValue) -> Result<usize, crate::v2::Error> {
                let s = header
                    .to_str()
                    .map_err(|_| crate::v2::Error::InvalidRetryAfterHeader {
                        header: header.clone(),
                    })?;
                let v: usize = s
                    .parse()
                    .map_err(|_| crate::v2::Error::InvalidRetryAfterHeader {
                        header: header.clone(),
                    })?;
                Ok(v)
            }

            pub async fn from_response(response: reqwest::Response) -> Self {
                let retry_after = {
                    if let Some(retry_after_header) = response.headers().get("retry-after") {
                        match Self::header_to_usize(retry_after_header) {
                            Ok(retry_after) => Some(retry_after),
                            Err(e) => return Self::Unexpected{raw_error: e},
                        }
                    } else {
                        None
                    }
                };

                let res: Result<JsonErrorObj, _> = response.json().await;
                match res {
                    Ok(raw_error) => Self::from_json_error_obj(raw_error, retry_after),
                    Err(e) => e.into(),
                }
            }

            /// number of seconds from the RetryAfter Header, if any
            pub fn retry_after(&self) -> Option<usize> {
                match self {
                    $( Self::$variant_name { raw_error: _, retry } => *retry,)*
                    Self::RequestError { error: _ } => None,
                    Self::Unexpected { raw_error: _ } => None,
                }
            }
        }


        impl From<reqwest::Error> for $enum_name {
            fn from(e: reqwest::Error) -> Self {
                if e.is_timeout() || e.is_connect() {
                    Self::RequestError {
                        error: e
                    }
                } else {
                    Self::Unexpected {
                        raw_error: crate::v2::Error::ReqwestError(e),
                    }
                }
            }
        }
    }
}

error_enum!(AuthorizeError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "unsupported", Unsupported),
    (403, "transaction_cap_exceeded", TransactionCapExceeded),
});

error_enum!(CopyError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (403, "access_denied", AccessDenied),
    (403, "cap_exceeded", CapExceeded),
    (404, "not_found", NotFound),
    (408, "request_timeout", RequestTimeout),
    (416, "range_not_satisfiable", RangeNotSatisfiable),
});

error_enum!(CreateBucketError {
    (400, "bad_request", BadRequest),
    (400, "too_many_buckets", TooManyBuckets),
    (400, "duplicate_bucket_name", DuplicateBucketName),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
});

error_enum!(DeleteFileVersionError {
    (400, "bad_request", BadRequest),
    (400, "bad_bucket_id", BadBucketId),
    (400, "file_not_present", FileNotPresent),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (401, "access_denied", AccessDenied),
});

error_enum!(DownloadFileError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (403, "access_denied", AccessDenied),
    (403, "download_cap_exceeded", DownloadCapExceeded),
    (404, "not_found", NotFound),
    (416, "range_not_satisfiable", RangeNotSatisfiable),
});

error_enum!(GetDownloadAuthorizationError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (503, "service_unavailable", ServiceUnavailable),
});

error_enum!(GetFileInfoError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (404, "not_found", NotFound),
});

error_enum!(GetUploadUrlError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (403, "storage_cap_exceeded", StorageCapExceeded),
    (503, "service_unavailable", ServiceUnavaliabe),
});

//Generic Error caused by backblaze, error value for multiple functions
error_enum!(GenericB2Error {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
});

error_enum!(ListFileNamesError {
    (400, "bad_request", BadRequest),
    (400, "invalid_bucket_id", InvalidBucketId),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (403, "transaction_cap_exceeded", TransactionCapExceeded),
    (503, "bad_request", BadRequestTimeout),
});

error_enum!(LargeFileError {
    (400, "bad_request", BadRequest),
    (400, "bad_bucket_id", BadBucketId),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
});

error_enum!(UpdateBucketError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (409, "conflict", Conflict),
});

error_enum!(UpdateFileLockError {
    (400, "bad_request", BadRequest),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (401, "access_denied", AccessDenied),
    (403, "cap_exceeded", CapExceeded),
    (405, "method_not_allowed", MethodNotAllowed),
});

error_enum!(UploadFileError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (403, "cap_exceeded", CapExceeded),
    (408, "request_timeout", RequestTimeout),
    (503, "service_unavailable", ServiceUnavailable),
});

error_enum!(UploadPartError {
    (400, "bad_request", BadRequest),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (408, "request_timeout", RequestTimeout),
    (503, "service_unavailable", ServiceUnavailable),
});

error_enum!(ListFileVersionsError {
    (400, "out_of_range",	OutOfRange),
    (400, "invalid_file_id", InvalidFileId),
    (401, "unauthorized", Unauthorized),
    (401, "bad_auth_token", BadAuthToken),
    (401, "expired_auth_token", ExpiredAuthToken),
    (503, "bad_request", BadRequest),
});

#[cfg(test)]
mod test {

    use crate::v2::JsonErrorObj;
    use std::convert::TryInto;

    error_enum!(TestEnum {
            (400, "bad_request", BadRequest),
            (401, "unauthorized", Unauthorized)
    });

    #[test]
    fn test_bad_request() {
        let json_err = JsonErrorObj {
            status: 400u16.try_into().unwrap(),
            code: "bad_request".to_owned(),
            message: "message".to_owned(),
        };
        let err = TestEnum::from_json_error_obj(json_err.clone(), None);
        match err {
            TestEnum::BadRequest {
                raw_error,
                retry: None,
            } => assert_eq!(json_err, raw_error),
            _ => panic!("Expected BadRequest, found {:#?}", err),
        }
    }

    #[test]
    fn test_invalid_response() {
        let json_err = JsonErrorObj {
            status: 400u16.try_into().unwrap(),
            code: "does_not_exist".to_owned(),
            message: "message".to_owned(),
        };
        let err = TestEnum::from_json_error_obj(json_err.clone(), None);
        match err {
            TestEnum::Unexpected {
                raw_error: crate::v2::Error::JsonError(raw_error),
            } => {
                assert_eq!(json_err, raw_error)
            }
            _ => panic!("Expected Unexpected, found {:#?}", err),
        }
    }
}
