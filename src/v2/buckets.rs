///! types for B2 buckets, based on https://www.backblaze.com/b2/docs/buckets.html
use std::hash::{Hash, Hasher};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    error::Error,
    fmt::Display,
};

use serde::{
    de,
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Serialize,
};

use super::InvalidData;

#[derive(Debug)]
pub struct InvalidCharacterError {
    ///an invalid character found - does not mean there were not others
    pub invalid_character: char,
}

impl InvalidCharacterError {
    pub fn new(invalid_character: char) -> Self {
        Self { invalid_character }
    }
}

impl Error for InvalidCharacterError {}

impl Display for InvalidCharacterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid character: {:#?}", self.invalid_character)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq)]
pub struct BucketName(String);

impl PartialEq for BucketName {
    /// custom Eq function, since case is ignored for bucketNames
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase().eq(&other.0.to_lowercase())
    }
}

impl TryFrom<String> for BucketName {
    type Error = InvalidCharacterError;

    /// Ensures all characters in string are valid, i.e. characters, numbers or '-'
    fn try_from(value: String) -> Result<Self, Self::Error> {
        for c in value.chars() {
            match c {
                'a'..='z' => {}
                'A'..='Z' => {}
                '0'..='9' => {}
                '-' => {}
                _ => {
                    return Err(Self::Error::new(c));
                }
            };
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum BucketType {
    AllPublic,
    AllPrivate,
    Snapshot,

    Other { name: String },
}

impl BucketType {
    /// Returns `true` if the bucket_type is [`Other`], i.e. this library does not know it
    pub fn is_other(&self) -> bool {
        matches!(self, Self::Other { .. })
    }
}

impl<S> From<S> for BucketType
where
    S: Into<String> + PartialEq<&'static str>,
{
    fn from(s: S) -> Self {
        // since s is generic we cannot use match here (?), so compare manually
        if s == "allPublic" {
            BucketType::AllPublic
        } else if s == "allPrivate" {
            BucketType::AllPrivate
        } else if s == "snapshot" {
            BucketType::Snapshot
        } else {
            BucketType::Other { name: s.into() }
        }
    }
}

impl Serialize for BucketType {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let name = match self {
            BucketType::AllPublic => "allPublic",
            BucketType::AllPrivate => "allPrivate",
            BucketType::Snapshot => "snapshot",
            BucketType::Other { name } => name,
        };
        // variant_index serialization is not supported, so always set it to 0
        s.serialize_str(name)
    }
}

struct BucketTypeVisitor;
impl<'de> de::Visitor<'de> for BucketTypeVisitor {
    type Value = BucketType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A String contianing a BucketType")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}

impl<'de> Deserialize<'de> for BucketType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BucketTypeVisitor)
    }
}

#[derive(Debug)]
pub enum BucketTypes {
    All,
    List(HashSet<BucketType>),
}

//custom serializer to support the "All"-Value
impl Serialize for BucketTypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BucketTypes::All => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element("all")?;
                seq.end()
            }
            BucketTypes::List(elems) => {
                let mut seq = serializer.serialize_seq(Some(elems.len()))?;
                for elem in elems.iter() {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
        }
    }
}

#[derive(Debug, Eq, Serialize, Deserialize)]
pub struct BucketInfoKey(String);

impl TryFrom<String> for BucketInfoKey {
    type Error = InvalidData;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.bytes().len() > 50 {
            return Err(InvalidData::new(
                "Keys for Bucket Info may be at most 50 characters long".into(),
            ));
        }
        let lower_value = value.to_lowercase();
        /* there are limits on valid b2- values, but as some values are allowed, do not exclude them right now
        if lower_value.starts_with("b2-") {
            return Err(InvalidData::new("Names starting with \"b2-\" are reserved and not allowed as keys for Bucket Info".into()));
        }
        */
        //validate characters used
        for c in lower_value.chars() {
            match c {
                'a'..='z'
                | '-'
                | '_'
                | '.'
                | '`'
                | '~'
                | '!'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '\''
                | '|'
                | '+' => {}
                _ => {
                    return Err(InvalidData::new(format!(
                        "Invalid Character found: {:#?}",
                        c
                    )))
                }
            }
        }
        Ok(Self(value))
    }
}

impl PartialEq for BucketInfoKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase().eq(&other.0.to_lowercase())
    }
}

impl Hash for BucketInfoKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

#[derive(Debug, Serialize)]
pub struct BucketInfoValue(Vec<u8>);

impl<'de> Deserialize<'de> for BucketInfoValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = Vec::deserialize(deserializer)?;
        if inner.len() > 10000 {
            Err(de::Error::invalid_length(
                inner.len(),
                &"At most 10000 Bytes",
            ))
        } else {
            Ok(Self(inner))
        }
    }
}

impl BucketInfoValue {
    pub fn new(data: Vec<u8>) -> Result<Self, InvalidData> {
        if data.len() > 10000 {
            Err(InvalidData::new(format!(
                "Too many Bytes in Bucket Info, at most 10000 are allowed, but {} were attemted",
                data.len()
            )))
        } else {
            Ok(Self(data))
        }
    }

    pub fn data(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<&str> for BucketInfoValue {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().into())
    }
}

#[derive(Debug)]
pub struct BucketInfo {
    key_values: HashMap<BucketInfoKey, BucketInfoValue>,
}

impl BucketInfo {
    pub fn new() -> Self {
        Self {
            key_values: HashMap::with_capacity(10),
        }
    }

    pub fn get(&self, key: &BucketInfoKey) -> Option<&BucketInfoValue> {
        self.key_values.get(key)
    }

    pub fn set(
        &mut self,
        key: BucketInfoKey,
        value: BucketInfoValue,
    ) -> Result<(), TooManyEntriesError> {
        let len = self.key_values.len();
        // only check length if it might be broken by this operation, if it is, check if action is just replacing an element
        if len >= 10 && !self.key_values.contains_key(&key) {
            return Err(TooManyEntriesError::new(len + 1));
        }
        self.key_values.insert(key, value);
        assert!(self.key_values.len() <= 10);
        Ok(())
    }
}

impl Serialize for BucketInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.key_values.len()))?;
        for (key, value) in self.key_values.iter() {
            s.serialize_entry(key, value)?;
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for BucketInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = HashMap::deserialize(deserializer)?;
        Ok(Self::try_from(inner).map_err(|e| {
            de::Error::invalid_length(e.number_of_entries(), &"At most 10 elements")
        })?)
    }
}

#[derive(Debug)]
pub struct TooManyEntriesError {
    number_of_entries: usize,
}

impl TooManyEntriesError {
    pub fn new(number_of_entries: usize) -> Self {
        Self { number_of_entries }
    }

    /// Number of entries that were attemted to create
    pub fn number_of_entries(&self) -> usize {
        self.number_of_entries
    }
}

impl Display for TooManyEntriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bucket Info may only contain up to 10 key/value pairs, but it was attemted to insert {}", self.number_of_entries)
    }
}

impl Error for TooManyEntriesError {}

impl TryFrom<HashMap<BucketInfoKey, BucketInfoValue>> for BucketInfo {
    type Error = TooManyEntriesError;

    fn try_from(value: HashMap<BucketInfoKey, BucketInfoValue>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > 10 {
            Err(TooManyEntriesError::new(len))
        } else {
            Ok(Self { key_values: value })
        }
    }
}

/// sadly the documentation does not state what datatype is to be used,
/// however it seems like this could get big very quickly, so u128 seems like the safest bet
/// unless BigUint or something similar is used
pub type BucketRevision = u128;
