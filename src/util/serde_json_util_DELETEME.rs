// serializer.rs

use std::marker::PhantomData;

use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;

pub fn serialize_none_as_empty_object<S, T>(
    option: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    match option {
        Some(value) => value.serialize(serializer),
        None => {
            let map = serializer.serialize_map(Some(0))?;
            map.end()
        }
    }
}

pub fn deserialize_empty_object_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct EmptyObjectVisitor<T>{
        marker:PhantomData<fn() -> Option<T>>
    }

    impl<'de, T> Visitor<'de> for EmptyObjectVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an object or an empty object")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            // If the map has entries, deserialize as T; otherwise return None
            println!("map size: {:?}", map.size_hint().unwrap_or(0usize));
            println!("first key {:?}", map.next_key::<String>());
            let size_hint = map.size_hint().unwrap_or(0usize);
            let map_deser = serde::de::value::MapAccessDeserializer::new(map);
            match T::deserialize(map_deser).map(Some){
                Ok(res)=>match size_hint{
                    0=>Ok(None),
                    _=>Ok(res)
                },
                err=>err
            }
        }
    }
    
    deserializer.deserialize_map(EmptyObjectVisitor{
        marker:PhantomData
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json;

    #[test]
    fn test_serialization() {
        #[derive(Serialize)]
        struct MyStruct {
            #[serde(serialize_with = "serialize_none_as_empty_object")]
            optional_field: Option<String>,
        }

        let my_struct = MyStruct {
            optional_field: None,
        };

        let json = serde_json::to_string(&my_struct).unwrap();
        assert_eq!(json, r#"{"optional_field":{}}"#);

        let my_struct = MyStruct {
            optional_field: Some("some".to_string()),
        };

        let json = serde_json::to_string(&my_struct).unwrap();
        assert_eq!(json, r#"{"optional_field":"some"}"#);
    }

    #[test]
    fn test_deserialization() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Pox {
            a: u8,
            b: u8,
        }

        #[derive(Deserialize)]
        struct MyStruct {
            #[serde(deserialize_with = "deserialize_empty_object_as_none")]
            optional_field: Option<Pox>,
        }

        // let json = r#"{"optional_field":{}}"#;
        // let my_struct: MyStruct = serde_json::from_str(json).unwrap();
        // assert_eq!(my_struct.optional_field, None);

        let json = r#"{"optional_field":{"a":1,"b":2}}"#;
        let my_struct: MyStruct = serde_json::from_str(json).unwrap();
        assert_eq!(my_struct.optional_field, Some(Pox { a: 1, b: 2 }));
    }
}
