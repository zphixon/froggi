use crate::{FroggiError, ErrorKind, AddMsg, serialize_to_bytes};

/// Represents a froggi request to a server.
///
/// The first byte is the froggi version number, the next two bytes are the request path length,
/// and the next bytes are the request path.
pub struct Request {
    version: u8,
    path: String,
}

impl Request {
    pub fn new(path: impl ToString) -> Result<Self, FroggiError> {
        let path = path.to_string();
        if path.len() > u16::MAX as usize {
            Err(FroggiError::new(ErrorKind::RequestFormatError).msg_str("The path is too large."))
        } else {
            Ok(Request {
                version: crate::FROGGI_VERSION,
                path: path.to_string(),
            })
        }
    }
}

impl Into<Vec<u8>> for Request {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(self.version);
        let (low, high) = serialize_to_bytes(self.path.len());
        v.push(low);
        v.push(high);
        v.extend(self.path.bytes());
        v
    }
}
