use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Similar to the `Option` type, but the `None` variant is serialized/deserialized by `serde_json`
/// to / from an empty object `{}`.
///
/// Introduces the constraint `#[serde(deny_unknown_fields)]` which means that for the `Object(T)`
/// type `T` cannot be used for json objects which may contain unknown fields.
///
/// TODO: This is a problem for compatibility
/// https://jupyter-client.readthedocs.io/en/latest/messaging.html#compatibility
///
/// > I tried very hard to make a custom serializer/deserializer for serde that allows serde_json to
/// > convert a generic `Option<T>`'s `None` variant to and from `{}` but it turns out not to be
/// > possible for the generic case. For one thing it seems impossible to check the number of keys
/// > in a map before serializing it, and even if you could you might end up having to write a
/// > custom serializer/deserializer for each type `T`.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum EmptyObjectOr<T> {
    EmptyObject {},
    Object(T),
}

impl<T> Default for EmptyObjectOr<T> {
    fn default() -> Self {
        Self::EmptyObject {}
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
            Self::EmptyObject {} => None,
            Self::Object(t) => Some(t),
        }
    }
}

impl<T> From<Option<T>> for EmptyObjectOr<T> {
    fn from(t: Option<T>) -> Self {
        match t {
            None => Self::EmptyObject {},
            Some(t) => Self::Object(t),
        }
    }
}

impl<T> EmptyObjectOr<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> EmptyObjectOr<U> {
        match self {
            Self::EmptyObject {} => EmptyObjectOr::<U>::EmptyObject {},
            Self::Object(t) => EmptyObjectOr::<U>::Object(f(t)),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::EmptyObject {} => {
                panic!("EmptyObjectOr::unwrap called on EmptyObjectOr::EmptyObject{{}}")
            }
            Self::Object(t) => t,
        }
    }

    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::EmptyObject {} => Err(err),
            Self::Object(t) => Ok(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{wire::Header, util::TryToJsonBytesString};

    #[test]
    fn test_serialize_empty_object_or() {
        let empty_object_or: EmptyObjectOr<u32> = EmptyObjectOr::EmptyObject {};
        let json = serde_json::to_string(&empty_object_or).unwrap();
        assert_eq!(json, "{}");
        let object_or = EmptyObjectOr::Object(Header {
            message_id: "a".into(),
            message_type: crate::wire::MessageType::CommMsg,
            username: "b".into(),
            session: "c".into(),
            date: "d".into(),
            version: "e".into(),
        });
        let json = serde_json::to_string(&object_or).unwrap();
        assert_eq!(
            json,
            r#"{"msg_id":"a","msg_type":"comm_msg","username":"b","session":"c","date":"d","version":"e"}"#
        );
    }

    #[test]
    fn test_try_to_json_string(){
        let empty_object_or: EmptyObjectOr<u32> = EmptyObjectOr::EmptyObject {};
        let json = empty_object_or.try_to_json_string().unwrap();
        assert_eq!(json, "{}");
        let object_or = EmptyObjectOr::Object(Header {
            message_id: "a".into(),
            message_type: crate::wire::MessageType::CommMsg,
            username: "b".into(),
            session: "c".into(),
            date: "d".into(),
            version: "e".into(),
        });
        let json = object_or.try_to_json_string().unwrap();
        assert_eq!(
            json,
            r#"{"msg_id":"a","msg_type":"comm_msg","username":"b","session":"c","date":"d","version":"e"}"#
        );
    }
}
