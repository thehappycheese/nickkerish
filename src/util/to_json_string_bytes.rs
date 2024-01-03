use anyhow::Result;
use bytes::Bytes;
use serde::Serialize;

pub trait TryToJsonBytesString {
    fn try_to_json_bytes(&self) -> Result<Bytes>;
    fn try_to_json_string(&self) -> Result<String>;
}

impl<'de, T> TryToJsonBytesString for T
where
    T: Serialize,
{
    fn try_to_json_bytes(&self) -> Result<Bytes> {
        Ok(Bytes::from(serde_json::to_string(self)?))
    }

    fn try_to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub trait TryFromJsonBytesString
where
    Self: Sized,
{
    fn try_from_json_bytes(bytes: &Bytes) -> Result<Self>;
    fn try_from_json_string(string: &str) -> Result<Self>;
}

impl<T> TryFromJsonBytesString for T
where
    T: for<'de> serde::Deserialize<'de>,
{
    fn try_from_json_bytes(bytes: &Bytes) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    fn try_from_json_string(string: &str) -> Result<Self> {
        Ok(serde_json::from_str(string)?)
    }
}
