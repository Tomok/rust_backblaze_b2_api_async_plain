mod serde_header_serializer;

use serde::Serialize;
pub use serde_header_serializer::HeaderSerialzier;

pub trait HeadersFrom {
    fn headers_from<V: Serialize>(self, value: V) -> Self;
}

impl HeadersFrom for reqwest::RequestBuilder {
    fn headers_from<V: Serialize>(self, value: V) -> Self {
        let mut headers_serializer = HeaderSerialzier::new(self);
        // unwrap is ok, headerSerialzier already panics! on every error...
        value.serialize(&mut headers_serializer).unwrap();
        headers_serializer.done()
    }
}
