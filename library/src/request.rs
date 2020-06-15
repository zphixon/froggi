use crate::{serialize_to_bytes, AddMsg, ErrorKind, FroggiError};

use std::io::Read;

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
        let version = crate::FROGGI_VERSION;

        if path.len() > u16::MAX as usize {
            Err(FroggiError::new(ErrorKind::RequestFormatError).msg_str("The path is too large."))
        } else {
            Ok(Request { version, path })
        }
    }

    pub fn from_bytes(bytes: &mut impl Read) -> Result<Self, FroggiError> {
        // request header
        let mut header = [0u8; 3];
        bytes.read_exact(&mut header)?;

        // consists of version and path length
        let version = header[0];
        let path_len = crate::deserialize_bytes(&header[1..]);

        // Vec::with_capacity doesn't work here for some reason
        let mut path_buf = vec![0; path_len];
        bytes.read_exact(&mut path_buf)?;

        let path = String::from_utf8(path_buf)?;

        Ok(Request { version, path })
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

impl Into<Vec<u8>> for Request {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.version);
        let (low, high) = serialize_to_bytes(self.path.len());
        data.push(low);
        data.push(high);
        data.extend(self.path.bytes());
        data
    }
}
