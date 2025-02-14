use crate::{Error, MessageCodec};
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

///
/// Codec for a simple Boolean
///
#[derive(Clone, PartialEq, Debug)]
pub struct NumberListCodec {
    pub list: Vec<serde_json::Value>,
}

// ///
// /// Implicit conversion from bool
// ///
// impl Into<NumberListCodec> for Vec<String> {
//     fn into(self) -> NumberListCodec {
//         return NumberListCodec { list: self };
//     }
// }

///
/// To ease display
///
impl Display for NumberListCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.list.len()))
    }
}

///
/// Do not use derive because we do not want { "value": true }
/// But only true or false on the payload
///
impl Serialize for NumberListCodec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for element in &self.list {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

///
/// See Serialize
///
impl<'de> Deserialize<'de> for NumberListCodec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let list = Vec::<serde_json::Value>::deserialize(deserializer)?;
        Ok(NumberListCodec { list })
    }
}

///
/// To apply all the required trait
///
impl MessageCodec for NumberListCodec {
    ///
    ///
    ///
    fn from_message_payload(data: &bytes::Bytes) -> Result<NumberListCodec, Error> {
        let p: NumberListCodec =
            serde_json::from_str(String::from_utf8(data.to_vec()).unwrap().as_str()).unwrap();
        Ok(p)
    }
    ///
    ///
    ///
    fn into_message_payload(&self) -> Result<Vec<u8>, Error> {
        let v = serde_json::to_string(self).map_err(|e| Error::SerializeFailure(e.to_string()))?;
        Ok(v.into_bytes())
    }

    ///
    fn typee() -> String {
        "number_list".to_string()
    }
}
