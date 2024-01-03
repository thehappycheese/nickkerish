use serde::{Deserialize, Serialize};

/// Similar to the Option type, but
/// the None variant is serialized/deserialized by `serde_json` to/from an empty object `{}`
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum EmptyObjectOr<T> {
    EmptyObject{},
    Object(T)
}

impl<T> Default for EmptyObjectOr<T>{
    fn default() -> Self {
        Self::EmptyObject{}
    }
}

impl<T> From<T> for EmptyObjectOr<T> {
    fn from(t: T) -> Self {
        Self::Object(t)
    }
}

impl<T> Into<Option<T>> for EmptyObjectOr<T> {
    fn into(self) -> Option<T> {
        match self {
            Self::EmptyObject{} => None,
            Self::Object(t) => Some(t)
        }
    }
}

impl<T> From<Option<T>> for EmptyObjectOr<T> {
    fn from(t: Option<T>) -> Self {
        match t {
            None => Self::EmptyObject{},
            Some(t) => Self::Object(t)
        }
    }
}

impl<T> EmptyObjectOr<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> EmptyObjectOr<U> {
        match self {
            Self::EmptyObject{} => EmptyObjectOr::<U>::EmptyObject{},
            Self::Object(t) => EmptyObjectOr::<U>::Object(f(t))
        }
    }
}