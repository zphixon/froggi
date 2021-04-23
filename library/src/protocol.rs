//! Constants for the froggi protocol
//!
//! All values are in bytes.

/// The byte length of the version of this request/response.
pub const FROGGI_VERSION_LEN: usize = 1;
/// The byte offset of the version of the request/response.
pub const FROGGI_VERSION_OFFSET: usize = 0;

/// The byte length of the request/response kind.
pub const REQUEST_RESPONSE_KIND_LEN: usize = 1;
/// The byte offset of the request/response kind.
pub const REQUEST_RESPONSE_KIND_OFFSET: usize = FROGGI_VERSION_LEN;

/// The byte length of the request/response client ID.
pub const REQUEST_RESPONSE_UUID_LEN: usize = 16;
/// The byte offset of the client ID.
pub const REQUEST_RESPONSE_UUID_OFFSET: usize =
    REQUEST_RESPONSE_KIND_OFFSET + REQUEST_RESPONSE_KIND_OFFSET;

/// The total length of the request/response header.
pub const FROGGI_HEADER_LEN: usize =
    FROGGI_VERSION_LEN + REQUEST_RESPONSE_KIND_LEN + REQUEST_RESPONSE_UUID_LEN;

// request constants

/// The total byte length of the request string length.
pub const REQUEST_LENGTH_LEN: usize = 2;
/// The byte offset of the request string length.
pub const REQUEST_LENGTH_OFFSET: usize = FROGGI_HEADER_LEN;

/// The byte offset of the request string.
pub const REQUEST_OFFSET: usize = REQUEST_LENGTH_OFFSET + REQUEST_LENGTH_LEN;

// response constants

/// The byte length of the total response length.
pub const TOTAL_RESPONSE_LENGTH_LEN: usize = 4;
/// The byte offset of the total response length.
pub const TOTAL_RESPONSE_LENGTH_OFFSET: usize = FROGGI_HEADER_LEN;

/// The byte length of the page length.
pub const PAGE_LENGTH_LEN: usize = 4;
/// The byte offset of the page length.
pub const PAGE_LENGTH_OFFSET: usize = TOTAL_RESPONSE_LENGTH_OFFSET + TOTAL_RESPONSE_LENGTH_LEN;

/// The byte offset of the page.
pub const PAGE_OFFSET: usize = PAGE_LENGTH_LEN + PAGE_LENGTH_OFFSET;

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
        assert_eq!(1, FROGGI_VERSION_LEN);
        assert_eq!(0, FROGGI_VERSION_OFFSET);
        assert_eq!(1, REQUEST_RESPONSE_KIND_LEN);
        assert_eq!(1, REQUEST_RESPONSE_KIND_OFFSET);
        assert_eq!(16, REQUEST_RESPONSE_UUID_LEN);
        assert_eq!(2, REQUEST_RESPONSE_UUID_OFFSET);

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
        assert_eq!(18, REQUEST_LENGTH_OFFSET);
        assert_eq!(20, REQUEST_OFFSET);
    }

    #[test]
    fn response_constants() {
        assert_eq!(4, TOTAL_RESPONSE_LENGTH_LEN);
        assert_eq!(18, TOTAL_RESPONSE_LENGTH_OFFSET);
        assert_eq!(4, PAGE_LENGTH_LEN);
        assert_eq!(22, PAGE_LENGTH_OFFSET);
        assert_eq!(26, PAGE_OFFSET);
    }
}
