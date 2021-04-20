//! Constants for the froggi protocol
//!
//! All values are in bytes.

pub const FROGGI_VERSION_LEN: usize = 1;
pub const FROGGI_VERSION_OFFSET: usize = 0;

pub const REQUEST_RESPONSE_KIND_LEN: usize = 1;
pub const REQUEST_RESPONSE_KIND_OFFSET: usize = FROGGI_VERSION_LEN;

pub const VERSION_KIND_HEADER_LEN: usize = FROGGI_VERSION_LEN + REQUEST_RESPONSE_KIND_LEN;

// request constants

pub const REQUEST_LENGTH_LEN: usize = 2;
pub const REQUEST_LENGTH_OFFSET: usize = VERSION_KIND_HEADER_LEN;

pub const REQUEST_OFFSET: usize = REQUEST_LENGTH_OFFSET + REQUEST_LENGTH_LEN;

// response constants

pub const TOTAL_RESPONSE_LENGTH_LEN: usize = 4;
pub const TOTAL_RESPONSE_LENGTH_OFFSET: usize = VERSION_KIND_HEADER_LEN;

pub const PAGE_LENGTH_LEN: usize = 4;
pub const PAGE_LENGTH_OFFSET: usize = TOTAL_RESPONSE_LENGTH_OFFSET + TOTAL_RESPONSE_LENGTH_LEN;

pub const PAGE_OFFSET: usize = PAGE_LENGTH_LEN + PAGE_LENGTH_OFFSET;

pub const NUM_ITEMS_LEN: usize = 1;
pub const ITEM_KIND_LEN: usize = 1;
pub const ITEM_NAME_LENGTH_LEN: usize = 1;
pub const ITEM_LENGTH_LEN: usize = 4;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_kind_constants() {
        assert_eq!(1, FROGGI_VERSION_LEN);
        assert_eq!(0, FROGGI_VERSION_OFFSET);
        assert_eq!(1, REQUEST_RESPONSE_KIND_LEN);
        assert_eq!(1, REQUEST_RESPONSE_KIND_OFFSET);
    }

    #[test]
    fn request_constants() {
        assert_eq!(2, REQUEST_LENGTH_LEN);
        assert_eq!(2, REQUEST_LENGTH_OFFSET);
        assert_eq!(4, REQUEST_OFFSET);
    }

    #[test]
    fn response_constants() {
        assert_eq!(4, TOTAL_RESPONSE_LENGTH_LEN);
        assert_eq!(2, TOTAL_RESPONSE_LENGTH_OFFSET);
        assert_eq!(4, PAGE_LENGTH_LEN);
        assert_eq!(6, PAGE_LENGTH_OFFSET);
        assert_eq!(10, PAGE_OFFSET);
    }
}
