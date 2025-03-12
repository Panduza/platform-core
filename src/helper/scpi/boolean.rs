use std::collections::HashMap;

use crate::Error;
use bytes::Bytes;

/// SCPI Boolean type
///
pub struct ScpiBoolean {
    /// Internal representation of the boolean value
    ///
    value: bool,
}

impl ScpiBoolean {
    /// Create a new ScpiBoolean from a bool
    ///
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    ///
    ///
    pub fn from_bytes(b: Bytes) -> Result<Self, Error> {
        Self::from_slice(&b)
    }

    /// Create a new ScpiBoolean from a string
    ///
    pub fn from_str_case_insensitive(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "on" | "1" => Ok(Self::new(true)),
            "off" | "0" => Ok(Self::new(false)),
            _ => Err(Error::CodecError(format!("Invalid boolean value: {:?}", s))),
        }
    }

    ///
    ///
    pub fn from_slice(s: &[u8]) -> Result<Self, Error> {
        match std::str::from_utf8(s) {
            Ok(r) => Self::from_str_case_insensitive(r),
            Err(e) => Err(Error::CodecError(format!(
                "Invalid utf8 decoding: {:?} / {:?}",
                e, s
            ))),
        }
    }

    ///
    ///
    pub fn from_bytes_and_map(s: Bytes, map: HashMap<&str, bool>) -> Result<Self, Error> {
        match std::str::from_utf8(&s) {
            Ok(r) => {
                match map.get(r) {
                    Some(value) => Ok(Self::new(*value)),
                    None => return Err(Error::CodecError(format!("value not mapped '{}'", r))),
                }
                // if r == true_str {
                //     return Ok(Self::new(true));
                // } else if r == false_str {
                //     return Ok(Self::new(false));
                // } else {
                //     return Err(Error::CodecError(format!("value not mapped '{}'", r)));
                // }
            }
            Err(e) => Err(Error::CodecError(format!(
                "Invalid utf8 decoding: {:?} / {:?}",
                e, s
            ))),
        }
    }

    /// Get the value of the ScpiBoolean
    ///
    pub fn value(&self) -> bool {
        self.value
    }

    /// Get the string representation of the Boolean
    ///
    pub fn to_str(&self) -> &'static str {
        match self.value {
            true => "ON",
            false => "OFF",
        }
    }

    /// Get the string representation of the Boolean
    ///
    pub fn to_digital_str(&self) -> &'static str {
        match self.value {
            true => "1",
            false => "0",
        }
    }
}

/// Implicit conversion from bool to ScpiBoolean
///
impl From<bool> for ScpiBoolean {
    fn from(b: bool) -> Self {
        Self { value: b }
    }
}

/// Implicit conversion from ScpiBoolean to bool
///
impl From<ScpiBoolean> for bool {
    fn from(b: ScpiBoolean) -> Self {
        b.value
    }
}
