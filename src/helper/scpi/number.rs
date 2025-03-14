use crate::Error;
use bytes::Bytes;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_from_str_valid() {
        // Test valid numerical strings
        let valid_cases = vec![
            ("0", 0.0),
            ("123.456", 123.456),
            ("-789.012", -789.012),
            ("0.0", 0.0),
            ("1e3", 1000.0),
            ("-1.23e-4", -0.000123),
        ];

        for (input, expected) in valid_cases {
            let result = ScpiNumber::from_str(input);
            assert!(result.is_ok(), "Failed to parse valid input: {}", input);
            let number = result.unwrap();
            assert_eq!(
                number.value(),
                expected,
                "Incorrect value for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_from_str_invalid() {
        // Test invalid numerical strings
        let invalid_cases = vec!["abc", "123.45.67", "1e3.4", "-", ""];

        for input in invalid_cases {
            let result = ScpiNumber::from_str(input);
            assert!(result.is_err(), "Parsed invalid input: {}", input);
            let error = result.unwrap_err();
            match error {
                Error::CodecError(msg) => {
                    assert_eq!(msg, format!("Invalid numerical value: {:?}", input))
                }
                _ => panic!("Unexpected error type for input: {}", input),
            }
        }
    }

    #[test]
    fn test_from_str_edge_cases() {
        // Test edge cases
        let edge_cases = vec![
            ("3.4028235e38", 3.4028235e38),   // Max f32
            ("-3.4028235e38", -3.4028235e38), // Min f32
            ("1.4e-45", 1.4e-45),             // Smallest positive f32
            ("-1.4e-45", -1.4e-45),           // Smallest negative f32
        ];

        for (input, expected) in edge_cases {
            let result = ScpiNumber::from_str(input);
            assert!(result.is_ok(), "Failed to parse edge case input: {}", input);
            let number = result.unwrap();
            assert_eq!(
                number.value(),
                expected,
                "Incorrect value for edge case input: {}",
                input
            );
        }
    }
}
