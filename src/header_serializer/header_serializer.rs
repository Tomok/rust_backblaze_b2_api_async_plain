use std::{convert::TryFrom, fmt::Display};

use reqwest::{header::HeaderValue, RequestBuilder};
use serde::{
    ser::{self},
    Serialize,
};
use typed_builder::TypedBuilder;

#[derive(Debug)]
pub enum HeaderSerialzierError {
    Custom { msg: String },
}

impl ser::Error for HeaderSerialzierError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom {
            msg: format!("{}", msg),
        }
    }
}

impl std::error::Error for HeaderSerialzierError {}

impl Display for HeaderSerialzierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HeaderSerializer failed.")
    }
}

/// Serializes into http headers for a [RequestBuilder], panics if an invalid struct was inserted
#[derive(TypedBuilder)]
pub struct HeaderSerialzier {
    #[builder(setter(strip_option))]
    request_builder: Option<RequestBuilder>,
    #[builder(default = "true")]
    bool_true: &'static str,
    #[builder(default = "false")]
    bool_false: &'static str,
    #[builder(default, setter(skip))]
    current_field_name: Option<&'static str>,
}

impl HeaderSerialzier {
    pub fn new(request_builder: RequestBuilder) -> Self {
        Self::builder().request_builder(request_builder).build()
    }

    fn try_serialize_field_value<V>(&mut self, value: V) -> Result<(), HeaderSerialzierError>
    where
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        if let Some(name) = self.current_field_name {
            let builder = self.request_builder.take().unwrap();
            self.request_builder = Some(builder.header(name, value));
            self.current_field_name = None;
            Ok(())
        } else {
            panic!("Serialized value, but field name not known");
        }
    }

    fn try_serialize_field_value_as_string<V: ToString>(
        &mut self,
        value: V,
    ) -> Result<(), HeaderSerialzierError> {
        self.try_serialize_field_value(value.to_string())
    }

    pub fn done(mut self) -> RequestBuilder {
        self.request_builder.take().unwrap()
    }
}

impl<'a> ser::Serializer for &'a mut HeaderSerialzier {
    type Ok = ();
    type Error = HeaderSerialzierError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let v_str = if v { self.bool_true } else { self.bool_false };
        self.try_serialize_field_value(v_str)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value_as_string(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value_as_string(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value_as_string(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value_as_string(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value_as_string(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.try_serialize_field_value(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // translate to header not included
        self.current_field_name = None;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        panic!("Not supported");
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        panic!("Not supported");
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        panic!("Not supported");
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        panic!("Not supported");
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        panic!("Not supported");
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // ignore struct name & continue
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        panic!("Not supported");
    } //TODO
}

impl<'a> ser::SerializeStruct for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;

    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_field_name = Some(key);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        panic!("Not supported");
    }
}

impl<'a> ser::SerializeTuple for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        panic!("Not supported");
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        panic!("Not supported");
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        panic!("Not supported");
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut HeaderSerialzier {
    type Ok = <&'a mut HeaderSerialzier as ser::Serializer>::Ok;
    type Error = <&'a mut HeaderSerialzier as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        panic!("Not supported");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        panic!("Not supported");
    }
}

#[cfg(test)]
mod test {
    use wiremock::{
        matchers::{header, method},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;

    #[derive(Debug, Serialize)]
    struct ExampleHeaderStruct {
        a: u8,
        b: bool,
        #[serde(rename = "c_renamed")]
        c: String,
        d: u32,
        o: Option<f32>,
        p: Option<bool>,
    }

    impl ExampleHeaderStruct {
        fn new(a: u8, b: bool, c: String, d: u32, o: Option<f32>, p: Option<bool>) -> Self {
            Self { a, b, c, d, o, p }
        }
    }

    impl Default for ExampleHeaderStruct {
        fn default() -> Self {
            Self::new(1u8, false, "STRING".into(), 2u32, None, Some(true))
        }
    }

    #[tokio::test]
    async fn test_example_header_struct_serialize() {
        let response = {
            let mock_server = MockServer::start().await;
            let _mock = Mock::given(method("GET"))
                .and(header("a", "1"))
                .and(header("b", "false"))
                .and(header("c_renamed", "STRING"))
                .and(header("d", "2"))
                .and(header("p", "true"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&mock_server)
                .await;

            let request = reqwest::Client::new().get(mock_server.uri());
            let mut serializer = HeaderSerialzier::new(request);
            let test_data = ExampleHeaderStruct::default();
            test_data.serialize(&mut serializer).unwrap();
            let res = serializer.done();
            res.send().await.unwrap()
        };
        assert_eq!(200, response.status());
    }
}
