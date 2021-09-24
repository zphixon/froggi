//! Constants for the froggi protocol
//!
//! All values are in bytes.

pub const FROGGI_MAGIC: [u8; 4] = [0xf0, 0x9f, 0x90, 0xb8];

pub const FROGGI_MAGIC_LEN: usize = 4;
pub const FROGGI_MAGIC_OFFSET: usize = 0;

/// The byte length of the version of this request/response.
pub const FROGGI_VERSION_LEN: usize = 1;
/// The byte offset of the version of the request/response.
pub const FROGGI_VERSION_OFFSET: usize = FROGGI_MAGIC_LEN;

/// The byte length of the request/response kind.
pub const REQUEST_RESPONSE_KIND_LEN: usize = 1;
/// The byte offset of the request/response kind.
pub const REQUEST_RESPONSE_KIND_OFFSET: usize = FROGGI_MAGIC_LEN + FROGGI_VERSION_LEN;

/// The byte length of the request/response client ID.
pub const REQUEST_RESPONSE_UUID_LEN: usize = 16;
/// The byte offset of the client ID.
pub const REQUEST_RESPONSE_UUID_OFFSET: usize =
    FROGGI_MAGIC_LEN + REQUEST_RESPONSE_KIND_LEN + REQUEST_RESPONSE_KIND_LEN;

/// The total length of the request/response header.
pub const FROGGI_HEADER_LEN: usize =
    FROGGI_MAGIC_LEN + FROGGI_VERSION_LEN + REQUEST_RESPONSE_KIND_LEN + REQUEST_RESPONSE_UUID_LEN;

// request constants

/// The total byte length of the request string length.
pub const REQUEST_LENGTH_LEN: usize = 2;
/// The byte offset of the request string length.
pub const REQUEST_LENGTH_OFFSET: usize = FROGGI_HEADER_LEN;

/// The byte offset of the request string.
pub const REQUEST_OFFSET: usize = FROGGI_HEADER_LEN + REQUEST_LENGTH_LEN;

// response constants

/// The byte length of the total response length.
pub const TOTAL_RESPONSE_LENGTH_LEN: usize = 4;
/// The byte offset of the total response length.
pub const TOTAL_RESPONSE_LENGTH_OFFSET: usize = FROGGI_HEADER_LEN;

/// The byte length of the page length.
pub const PAGE_LENGTH_LEN: usize = 4;
/// The byte offset of the page length.
pub const PAGE_LENGTH_OFFSET: usize = FROGGI_HEADER_LEN + TOTAL_RESPONSE_LENGTH_LEN;

/// The byte offset of the page.
pub const PAGE_OFFSET: usize = FROGGI_HEADER_LEN + TOTAL_RESPONSE_LENGTH_LEN + PAGE_LENGTH_LEN;

/// The byte length of the number of items.
pub const NUM_ITEMS_LEN: usize = 1;
/// The byte length of the item kind.
pub const ITEM_KIND_LEN: usize = 1;
/// The byte length of the item name.
pub const ITEM_NAME_LENGTH_LEN: usize = 1;
/// The byte length of the item length.
pub const ITEM_LENGTH_LEN: usize = 4;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header_constants() {
        assert_eq!(4, FROGGI_MAGIC_LEN);
        assert_eq!(1, FROGGI_VERSION_LEN);
        assert_eq!(4, FROGGI_VERSION_OFFSET);
        assert_eq!(1, REQUEST_RESPONSE_KIND_LEN);
        assert_eq!(5, REQUEST_RESPONSE_KIND_OFFSET);
        assert_eq!(16, REQUEST_RESPONSE_UUID_LEN);
        assert_eq!(6, REQUEST_RESPONSE_UUID_OFFSET);

        assert_eq!(
            REQUEST_RESPONSE_UUID_LEN,
            (REQUEST_RESPONSE_UUID_OFFSET..REQUEST_LENGTH_OFFSET).len()
        );

        assert_eq!(
            REQUEST_RESPONSE_UUID_LEN,
            (REQUEST_RESPONSE_UUID_OFFSET..TOTAL_RESPONSE_LENGTH_OFFSET).len()
        );
    }

    #[test]
    fn request_constants() {
        assert_eq!(2, REQUEST_LENGTH_LEN);
        assert_eq!(22, REQUEST_LENGTH_OFFSET);
        assert_eq!(24, REQUEST_OFFSET);
    }

    #[test]
    fn response_constants() {
        assert_eq!(4, TOTAL_RESPONSE_LENGTH_LEN);
        assert_eq!(22, TOTAL_RESPONSE_LENGTH_OFFSET);
        assert_eq!(4, PAGE_LENGTH_LEN);
        assert_eq!(26, PAGE_LENGTH_OFFSET);
        assert_eq!(30, PAGE_OFFSET);
    }
}
