use anyhow::Result;
use serde::{Deserialize, Serialize};


/// Similar to the `Option` type, but the `None` variant is serialized/deserialized by `serde_json`
/// to / from an empty object `{}`.
///
/// Note that although `EmptyObjectOr<T>` adds the constraint `#[serde(deny_unknown_fields)]`, the
/// inner object `T` is not bound by this and may independently specify this this constraint if
/// needed. The behavior is important for compatibility with the jupyter messaging specification
/// which requires that for [MessageContent](crate::wire::MessageContent) objects "Both sides should
/// allow extra fields in known message types" See (
/// [Compatibility](https://jupyter-client.readthedocs.io/en/latest/messaging.html#compatibility))
///
/// > NOTE: I tried very hard to make a custom serializer/deserializer for serde that allows serde_json to
/// > encode/decode a generic `Option<T>::None` variant to and from `{}`
/// > but it turns out to be unexpectedly difficult
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
    use crate::{
        protocol::{
            Header,
            MessageType,
        },
        util::TryToJsonBytesString
    };

    #[test]
    fn test_serialize_empty_object_or() {
        let empty_object_or: EmptyObjectOr<u32> = EmptyObjectOr::EmptyObject {};
        let json = serde_json::to_string(&empty_object_or).unwrap();
        assert_eq!(json, "{}");
        let object_or = EmptyObjectOr::Object(Header {
            message_id: "a".into(),
            message_type: MessageType::CommMsg,
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
            message_type: MessageType::CommMsg,
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


    /// https://jupyter-client.readthedocs.io/en/latest/messaging.html#compatibility
    /// "Both sides should allow unexpected message types, and extra fields in known message types,
    /// so that additions to the protocol do not break existing code."
    /// 
    /// This test shows that although EmptyObjectOr<T> is marked with #[serde(deny_unknown_fields)],
    /// the inner object T can be used for json objects which may contain unknown fields.
    #[test]
    fn test_if_inner_object_can_tolerate_additional_fields(){

        #[derive(Serialize, Deserialize, Debug)]
        struct Dummy {
            a: u32,
            // b: u32,
        }

        let input_data = r#"{"a":10,"b":20}"#;
        let dummy: EmptyObjectOr<Dummy> = serde_json::from_str(input_data).unwrap();
        let dummy_inner = dummy.unwrap();
        assert_eq!(dummy_inner.a, 10);
    }
}
