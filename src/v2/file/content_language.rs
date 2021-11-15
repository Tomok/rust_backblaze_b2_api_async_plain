use headers::{Header, HeaderName, HeaderValue};
use http::header::InvalidHeaderValue;

/// Content Language value acc. to RFC 2616.
/// Defined as trait to allow usage of headers::ContentLanguage should that be added back in,
/// without breaking existing code using the [ContentLanguageType] defined as a workaround in this library.
#[derive(Debug, Clone)]
pub struct ContentLanguage {
    value: HeaderValue,
}

impl Header for ContentLanguage {
    fn name() -> &'static HeaderName {
        &http::header::CONTENT_LANGUAGE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?.clone();
        Ok(Self { value })
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        values.extend(std::iter::once(self.value.clone()));
    }
}

impl TryFrom<&str> for ContentLanguage {
    type Error = InvalidHeaderValue;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            value: HeaderValue::from_str(value)?,
        })
    }
}
