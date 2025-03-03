mod boolean;
use crate::Error;
use boolean::ScpiBoolean;
use bytes::Bytes;

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
