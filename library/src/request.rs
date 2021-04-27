//! Types for dealing with a froggi protocol request.

use crate::{protocol::*, serialize_to_bytes, Result, Uuid};
use crate::{Encoding, FromEncoding, RequestFormat, IO};

use snafu::ResultExt;

use std::convert::TryInto;
use std::io::Read;

// TODO proc macro
// /// Kinds of requests.
// ///
// /// PageOnly - Plain old page. Don't send me any items or additional page expressions.
// /// I won't be talking to you again.
// /// PageItems - Page with items, but no interactions. Don't send me any additional page
// /// expressions. I won't be talking to you again.
// /// Page - Give me everything. I'll be in touch again for those extra page
// /// expressions.
// /// Put - Here's some data. I'm eagerly awaiting your response.
crate::u8enum! { RequestKind {
    PageOnly = 0,
    PageItems = 1,
    Page = 2,
    Put = 14,
    Unknown = 15,
} }

/// Represents a froggi request to a server.
#[derive(Debug)]
pub struct Request {
    version: u8,
    kind: RequestKind,
    id: Uuid,
    request: String,
}

impl Request {
    /// Create a new request with no client ID.
    pub fn new(request: impl ToString, kind: RequestKind) -> Result<Self> {
        let request = request.to_string();
        let version = crate::FROGGI_VERSION;

        if request.len() > u16::MAX as usize {
            RequestFormat.fail()
            //Err(FroggiError::new(ErrorKind::RequestFormatError).msg_str("The path is too large."))
        } else {
            Ok(Request {
                version,
                kind,
                id: Uuid::nil(),
                request,
            })
        }
    }

    /// Create a new request with a client ID.
    pub fn new_with_id(request: impl ToString, id: Uuid, kind: RequestKind) -> Result<Self> {
        let request = request.to_string();
        let version = crate::FROGGI_VERSION;

        if request.len() > u16::MAX as usize {
            RequestFormat.fail()
            //Err(FroggiError::new(ErrorKind::RequestFormatError).msg_str("The path is too large."))
        } else {
            Ok(Request {
                version,
                kind,
                id,
                request,
            })
        }
    }

    /// Read a request from a source of bytes.
    pub fn from_bytes(bytes: &mut impl Read) -> Result<Self> {
        // request header, twenty bytes
        let mut header = [0u8; REQUEST_OFFSET];
        bytes.read_exact(&mut header).context(IO)?;

        // first byte is version
        let version = header[FROGGI_VERSION_OFFSET];

        // next byte is request kind
        let kind = header[REQUEST_RESPONSE_KIND_OFFSET].into();

        // next 16 bytes are client id - unwrap is OK since the slice length is definitely 16 bytes
        let id = Uuid::from_bytes(
            header[REQUEST_RESPONSE_UUID_OFFSET..REQUEST_LENGTH_OFFSET]
                .try_into()
                .unwrap(),
        );

        // next two bytes are request length
        let request_length = crate::deserialize_bytes(&header[REQUEST_LENGTH_OFFSET..])?;

        // remaining bytes are the request itself
        let mut request_buf = vec![0; request_length];
        bytes.read_exact(&mut request_buf).context(IO)?;

        let request = String::from_utf8(request_buf).context(FromEncoding)?;

        Ok(Request {
            version,
            kind,
            id,
            request,
        })
    }

    /// Get the version of this request
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get the client ID supplied by the client
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the client ID supplied by the client
    pub fn kind(&self) -> RequestKind {
        self.kind
    }

    /// Get the request string
    pub fn request(&self) -> &str {
        &self.request
    }

    /// Convert the request into bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

impl Into<Vec<u8>> for Request {
    fn into(self) -> Vec<u8> {
        // first byte is version
        let mut data = Vec::new();
        data.push(self.version);

        // second byte is request kind
        data.push(self.kind.into());

        // next 16 bytes are client ID
        data.extend(self.id.as_bytes());

        // next two bytes are request length
        // unwrap safety - we check the path length before request construction
        let (low, high) = serialize_to_bytes(self.request.len()).unwrap();
        data.push(low);
        data.push(high);

        // remainder of request is the path
        data.extend(self.request.bytes());

        data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[rustfmt::skip]
    const REQUEST_BYTES: &[u8] = &[
        0x00,                                                       // version
        0x00,                                                       // kind
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // UUID
        0x09, 0x00,                                                 // request length
        0x69, 0x6e, 0x64, 0x65, 0x78, 0x2e, 0x66, 0x6d, 0x6c,       // request path
    ];

    #[test]
    fn from_bytes() {
        let mut bytes = REQUEST_BYTES.clone();
        let request = Request::from_bytes(&mut bytes).unwrap();
        assert_eq!(request.version, crate::FROGGI_VERSION);
        assert_eq!(&request.request, "index.fml");
    }

    #[test]
    fn to_bytes() {
        let request = Request::new("index.fml", RequestKind::PageOnly).unwrap();
        let data_test = request.into_bytes();

        assert_eq!(data_test.len(), REQUEST_BYTES.len());

        crate::test::test_bytes(REQUEST_BYTES, &data_test).unwrap();
    }
}
