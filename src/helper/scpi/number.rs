use crate::Error;
use bytes::Bytes;

/// SCPI Number type
///
pub struct ScpiNumber {
    /// Internal representation of the numerical value
    ///
    value: f32,
}

impl ScpiNumber {
    /// Create a new ScpiNumber from a f32
    ///
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    /// Create a new ScpiNumber from bytes
    ///
    pub fn from_bytes(b: Bytes) -> Result<Self, Error> {
        Self::from_slice(&b)
    }

    /// Create a new ScpiNumber from a string
    ///
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.parse::<f32>() {
            Ok(value) => Ok(Self::new(value)),
            Err(_) => Err(Error::CodecError(format!(
                "Invalid numerical value: {:?}",
                s
            ))),
        }
    }

    /// Create a new ScpiNumber from a byte slice
    ///
    pub fn from_slice(s: &[u8]) -> Result<Self, Error> {
        match std::str::from_utf8(s) {
            Ok(r) => Self::from_str(r),
            Err(e) => Err(Error::CodecError(format!(
                "Invalid utf8 decoding: {:?} / {:?}",
                e, s
            ))),
        }
    }

    /// Get the value of the ScpiNumber
    ///
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get the string representation of the number
    ///
    pub fn to_str(&self) -> String {
        self.value.to_string()
    }
}

/// Implicit conversion from f32 to ScpiNumber
///
impl From<f32> for ScpiNumber {
    fn from(value: f32) -> Self {
        Self { value }
    }
}

/// Implicit conversion from ScpiNumber to f32
///
impl From<ScpiNumber> for f32 {
    fn from(number: ScpiNumber) -> Self {
        number.value
    }
}
