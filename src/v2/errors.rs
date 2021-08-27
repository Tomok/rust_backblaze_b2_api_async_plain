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
