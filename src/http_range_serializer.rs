use http_range::HttpRange;
use serde::Serializer;

pub fn range_as_string(range: &HttpRange) -> String {
    format!("bytes={}-{}", range.start, range.start + range.length)
}

#[allow(dead_code)]
pub fn range_serializer<S>(range: &HttpRange, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&range_as_string(range))
}

#[allow(dead_code)]
pub fn range_option_serializer<S>(
    range_opt: &Option<&HttpRange>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match range_opt {
        None => serializer.serialize_none(),
        Some(range) => serializer.serialize_some(&range_as_string(range)),
    }
}
