// use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Error, MessageCodec, StableNumber};

use std::fmt::Write;

fn format_number(number: f32, decimal_places: usize) -> String {
    let mut formatted_string = String::new();

    // Handle potential formatting errors
    if decimal_places > 10 {
        return "Invalid decimal places".to_string();
    }

    // Format the number with the specified decimal places
    write!(
        &mut formatted_string,
        "{:.1$}",
        number, decimal_places as usize
    )
    .unwrap();

    formatted_string
}

#[derive(Clone, PartialEq, Debug)]
pub struct SiCodec {
    value: String,
}

impl SiCodec {
    pub fn from_f32(value: f32, decimals: usize) -> Self {
        Self {
            value: format_number(value, decimals as usize),
        }
    }

    pub fn into_f32(&self) -> Result<f32, Error> {
        self.value
            .parse()
            .map_err(|e| Error::DeserializeError(format!("{:?}", e)))
    }

    pub fn into_stable_number(&self) -> StableNumber {
        StableNumber {
            value: self.value.clone(),
        }
    }
}

///
/// Allow implicit convertion
///
impl From<u16> for SiCodec {
    fn from(value: u16) -> SiCodec {
        SiCodec {
            value: value.to_string(),
        }
    }
}

///
/// Allow implicit convertion
///
impl From<u32> for SiCodec {
    fn from(value: u32) -> SiCodec {
        SiCodec {
            value: value.to_string(),
        }
    }
}

///
/// Allow implicit convertion
///
impl From<i32> for SiCodec {
    fn from(value: i32) -> SiCodec {
        SiCodec {
            value: value.to_string(),
        }
    }
}

// ///
// /// Allow implicit convertion
// ///
// impl Into<SiCodec> for u64 {
//     fn into(self) -> SiCodec {
//         return SiCodec {
//             value: serde_json::json!(self),
//         };
//     }
// }

// ///
// /// Allow implicit convertion
// ///
// impl Into<SiCodec> for i32 {
//     fn into(self) -> SiCodec {
//         return SiCodec {
//             value: serde_json::json!(self),
//         };
//     }
// }

// ///
// /// Do not use derive because we do not want { "value": true }
// /// But only true or false on the payload
// ///
// impl Serialize for SiCodec {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         self.value.serialize(serializer)
//     }
// }

// ///
// /// See Serialize
// ///
// impl<'de> Deserialize<'de> for SiCodec {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let value = serde_json::Value::deserialize(deserializer)?;
//         Ok(SiCodec { value })
//     }
// }

impl MessageCodec for SiCodec {
    ///
    ///
    ///
    fn from_message_payload(data: &bytes::Bytes) -> Result<Self, Error> {
        let ppp = String::from_utf8(data.to_vec()).unwrap();
        // let p: Self =
        //     serde_json::from_str(String::from_utf8(data.to_vec()).unwrap().as_str()).unwrap();
        Ok(Self { value: ppp })
    }
    ///
    ///
    ///
    fn into_message_payload(&self) -> Result<Vec<u8>, Error> {
        // let v = serde_json::to_string(self).map_err(|e| Error::SerializeFailure(e.to_string()))?;

        Ok(self.value.clone().into_bytes())
    }

    ///
    fn typee() -> String {
        "si".to_string()
    }
}
