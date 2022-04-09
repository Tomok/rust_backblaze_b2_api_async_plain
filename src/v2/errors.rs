use super::JsonErrorObj;

/// How this error should be handled acc. to [https://www.backblaze.com/b2/docs/calling.html#error_handling] and [https://www.backblaze.com/b2/docs/integration_checklist.html]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RecommendedReaction<'s> {
    /// Something in the request or api is wrong, call/cap limit reached, ... no chance of recovery
    Raise,
    /// Retry after some time
    Retry {
        /// delay returned by Backblaze, if any. If none was returned, use an exponential backoff starting with 1 Sec
        delay: &'s Option<usize>,
    },
    /// Authorization expired, reauthenticate
    Reauthenticate,
    /// get a new upload url
    GetNewUploadUrl,
}
trait B2Error {
    fn recommended_action(&self) -> RecommendedReaction<'_>;
}

macro_rules! reactionIdentToRecommendedReaction {
    (Raise, $retry:ident) => {
        RecommendedReaction::Raise
    };
    (Retry, $retry:ident) => {
        RecommendedReaction::Retry { delay: &$retry }
    };
    (Reauthenticate, $retry:ident) => {
        RecommendedReaction::Reauthenticate
    };
    (GetNewUploadUrl, $retry:ident) => {
        RecommendedReaction::GetNewUploadUrl
    };
}

macro_rules! error_enum{
    ($enum_name:ident {
        $(($variant_code:literal, $variant_text:literal, $variant_name:ident, $reaction:ident)),* $(,)?
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

        impl crate::v2::errors::B2Error for $enum_name {
            #[allow(unused_variables)]
            fn recommended_action(&self) -> RecommendedReaction<'_> {
                match self {
                    $(
                        Self::$variant_name { raw_error: _, retry } => {reactionIdentToRecommendedReaction!($reaction, retry)}
                    ,)*

                    Self::RequestError{ error: _ } => {
                        todo!()
                    },
                    Self::Unexpected { raw_error: _ } => RecommendedReaction::Raise,

                }
            }
        }
    }
}

error_enum!(AuthorizeError {
    (401, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "unsupported", Unsupported, Raise),
    (403, "transaction_cap_exceeded", TransactionCapExceeded, Raise),
});

error_enum!(CopyError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (403, "access_denied", AccessDenied, Raise),
    (403, "cap_exceeded", CapExceeded, Raise),
    (404, "not_found", NotFound, Raise),
    (408, "request_timeout", RequestTimeout, Retry),
    (416, "range_not_satisfiable", RangeNotSatisfiable, Raise),
});

error_enum!(CreateBucketError {
    (400, "bad_request", BadRequest, Raise),
    (400, "too_many_buckets", TooManyBuckets, Raise),
    (400, "duplicate_bucket_name", DuplicateBucketName, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
});

error_enum!(DeleteFileVersionError {
    (400, "bad_request", BadRequest, Raise),
    (400, "bad_bucket_id", BadBucketId, Raise),
    (400, "file_not_present", FileNotPresent, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (401, "access_denied", AccessDenied, Raise),
});

error_enum!(DownloadFileError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (403, "access_denied", AccessDenied, Raise),
    (403, "download_cap_exceeded", DownloadCapExceeded, Raise),
    (404, "not_found", NotFound, Raise),
    (416, "range_not_satisfiable", RangeNotSatisfiable, Raise),
});

error_enum!(GetDownloadAuthorizationError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (503, "service_unavailable", ServiceUnavailable, Retry),
});

error_enum!(GetFileInfoError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (404, "not_found", NotFound, Raise),
});

error_enum!(GetUploadUrlError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (403, "storage_cap_exceeded", StorageCapExceeded, Raise),
    (503, "service_unavailable", ServiceUnavaliabe, Retry),
});

//Generic Error caused by backblaze, error value for multiple functions
error_enum!(GenericB2Error {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
});

error_enum!(ListFileNamesError {
    (400, "bad_request", BadRequest, Raise),
    (400, "invalid_bucket_id", InvalidBucketId, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (403, "transaction_cap_exceeded", TransactionCapExceeded, Raise),
    (503, "bad_request", BadRequestTimeout, Retry),
});

error_enum!(LargeFileError {
    (400, "bad_request", BadRequest, Raise),
    (400, "bad_bucket_id", BadBucketId, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
});

error_enum!(UpdateBucketError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (409, "conflict", Conflict, Raise),
});

error_enum!(UpdateFileLockError {
    (400, "bad_request", BadRequest, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (401, "access_denied", AccessDenied, Raise),
    (403, "cap_exceeded", CapExceeded, Raise),
    (405, "method_not_allowed", MethodNotAllowed, Raise),
});

error_enum!(UploadFileError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (403, "cap_exceeded", CapExceeded, Raise),
    (408, "request_timeout", RequestTimeout, GetNewUploadUrl),
    (503, "service_unavailable", ServiceUnavailable, GetNewUploadUrl),
});

error_enum!(UploadPartError {
    (400, "bad_request", BadRequest, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (408, "request_timeout", RequestTimeout, GetNewUploadUrl),
    (503, "service_unavailable", ServiceUnavailable, GetNewUploadUrl),
});

error_enum!(ListFileVersionsError {
    (400, "out_of_range",	OutOfRange, Raise),
    (400, "invalid_file_id", InvalidFileId, Raise),
    (401, "unauthorized", Unauthorized, Raise),
    (401, "bad_auth_token", BadAuthToken, Reauthenticate),
    (401, "expired_auth_token", ExpiredAuthToken, Reauthenticate),
    (503, "bad_request", BadRequest, Retry),
});

#[cfg(test)]
mod test {

    use super::*;
    use crate::v2::JsonErrorObj;
    use std::convert::TryInto;

    error_enum!(TestEnum {
            (400, "bad_request", BadRequest, Raise),
            (401, "unauthorized", Unauthorized, Raise),
            (408, "request_timeout", RequestTimeoutInServer, Retry),
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
