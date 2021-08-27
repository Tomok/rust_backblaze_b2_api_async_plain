macro_rules! error_enum{
    ($enum_name:ident {
        $(($variant_code:literal, $variant_text:literal, $variant_name:ident)),* $(,)?
    }) => {
        #[derive(Debug)]
        pub enum $enum_name {
            $($variant_name { raw_error: crate::v2::JsonErrorObj },)*
            Unexpected {
                raw_error: crate::v2::Error,
            },
        }

        impl From<crate::v2::JsonErrorObj> for $enum_name {
            fn from(raw_error: crate::v2::JsonErrorObj) -> Self {
                match (raw_error.status as u16, raw_error.code.as_str()) {
                    $(($variant_code, $variant_text) => Self::$variant_name { raw_error },)*
                    _ => Self::Unexpected {
                        raw_error: crate::v2::Error::JsonError(raw_error),
                    },
                }
            }
        }


        impl From<reqwest::Error> for $enum_name {
            fn from(e: reqwest::Error) -> Self {
                //TODO separate error for network / timeouts??
                Self::Unexpected {
                    raw_error: crate::v2::Error::ReqwestError(e),
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

error_enum!(CopyFileError {
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
    (403, "transaction_cap_exceeded", TransactionCapExceeded),
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

error_enum!(ListBucketsError {
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

error_enum!(StartLargeFileError {
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

error_enum!(UpdateFileLegalHoldError {
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
        let err: TestEnum = json_err.clone().into();
        match err {
            TestEnum::BadRequest { raw_error } => assert_eq!(json_err, raw_error),
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
        let err: TestEnum = json_err.clone().into();
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
