mod boolean;
use crate::Error;
use boolean::ScpiBoolean;
use bytes::Bytes;

// use  as JsonValue;

///
///
pub fn e_query<S: Into<String>>(query: S) -> Bytes {
    Bytes::from(query.into())
}

///
///
pub fn d_boolean(data: Bytes) -> Result<bool, Error> {
    Ok(ScpiBoolean::from_bytes(data)?.value())
}

/// Return the string that represent a code snippet that generate a Bytes object
/// to request a boolean to the device
///
pub fn d_get_boolean(codec_settings: &serde_json::Value) -> String {
    // "bytes::Bytes::from(codec_settings.query)"

    "".to_string()
}
// s_get_boolean()
// d_set_boolean() => if some ask else tell
// s_set_boolean()
