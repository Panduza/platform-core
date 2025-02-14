use crate::{Error, MessageCodec};
use std::fmt::Display;

///
/// Codec for a simple Boolean
///
#[derive(Clone, PartialEq, Debug)]
pub struct RawCodec {
    pub data: bytes::Bytes,
}

///
/// Allow implicit convertion
///
impl From<bytes::Bytes> for RawCodec {
    fn from(data: bytes::Bytes) -> Self {
        Self { data: data.clone() }
    }
}

impl From<&[u8]> for RawCodec {
    fn from(data: &[u8]) -> Self {
        Self {
            data: bytes::Bytes::copy_from_slice(data),
        }
    }
}

///
/// To ease display
///
impl Display for RawCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}

///
/// To apply all the required trait
///
impl MessageCodec for RawCodec {
    ///
    ///
    ///
    fn from_message_payload(data: &bytes::Bytes) -> Result<RawCodec, Error> {
        Ok(RawCodec { data: data.clone() })
    }
    ///
    ///
    ///
    fn into_message_payload(&self) -> Result<Vec<u8>, Error> {
        Ok(self.data.clone().into())
    }
    ///
    ///
    fn typee() -> String {
        "raw".to_string()
    }
}
