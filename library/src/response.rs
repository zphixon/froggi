//! Types for dealing with a froggi protocol response.

use crate::{protocol::*, AddMsg, ErrorKind, FroggiError, Uuid};

use crate::ErrorKind::ResponseFormatError;
use std::convert::TryInto;
use std::io::Read;

// TODO proc macro
crate::u8enum! { ResponseKind {
    Page = 0,
    PageNoItems = 1,
    Unknown = 15,
} }

// TODO proc macro
crate::u8enum! { ItemKind {
    Png = 0,
    Jpg = 1,
    Gif = 2,
    Unknown = 15,
} }

/// An extra item that may appear at the end of a page.
pub struct Item {
    name: String,
    kind: ItemKind,
    data: Vec<u8>,
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{ name: {}, data: ... }}", self.name)
    }
}

impl Item {
    /// Create a new item
    pub fn new(name: String, kind: ItemKind, data: Vec<u8>) -> Item {
        Item { name, kind, data }
    }

    /// Get the name of the item
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the kind of the item
    pub fn kind(&self) -> ItemKind {
        self.kind
    }

    /// Get the data of the item
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Represents a response from a froggi server.
#[derive(Debug)]
pub struct Response {
    version: u8,
    kind: ResponseKind,
    id: Uuid,
    page: String,
    items: Vec<Item>,
}

fn check_page_and_items(page: &str, items: &[Item]) -> Result<(), FroggiError> {
    if items.len() > u8::MAX as usize {
        return Err(
            FroggiError::new(ErrorKind::ResponseFormatError).msg_str("There are too many items.")
        );
    }

    for item in items.iter() {
        if item.data.len() > u32::MAX as usize {
            return Err(FroggiError::new(ErrorKind::ResponseFormatError)
                .msg(format!("The item {} is too long.", item.name)));
        }

        if item.name.len() > u8::MAX as usize {
            return Err(FroggiError::new(ErrorKind::ResponseFormatError)
                .msg(format!("The item name {} is too long.", item.name)));
        }
    }

    if page.len() > u32::MAX as usize {
        return Err(
            FroggiError::new(ErrorKind::ResponseFormatError).msg_str("The page is too long.")
        );
    }

    if FROGGI_HEADER_LEN
        + items
            .iter()
            .map(|item| {
                item.data.len()
                    + item.name.len()
                    + ITEM_KIND_LEN
                    + ITEM_KIND_LEN
                    + ITEM_NAME_LENGTH_LEN
            })
            .sum::<usize>()
        + PAGE_LENGTH_LEN
        + page.len()
        > (u32::MAX as usize)
    {
        return Err(
            FroggiError::new(ResponseFormatError).msg_str("The page and items are too large.")
        );
    }

    Ok(())
}

impl Response {
    /// Create a new response with a random ID.
    pub fn new(kind: ResponseKind, page: String, items: Vec<Item>) -> Result<Self, FroggiError> {
        check_page_and_items(&page, &items)?;

        Ok(Self {
            version: crate::FROGGI_VERSION,
            kind,
            id: Uuid::new_v4(),
            page,
            items,
        })
    }

    /// Create a new response with a client ID.
    pub fn new_with_id(
        kind: ResponseKind,
        id: Uuid,
        page: String,
        items: Vec<Item>,
    ) -> Result<Self, FroggiError> {
        check_page_and_items(&page, &items)?;

        Ok(Self {
            version: crate::FROGGI_VERSION,
            kind,
            id,
            page,
            items,
        })
    }

    /// Parse the response into a page. Zero-copy.
    pub fn parse(&self) -> Result<crate::markup::Page<'_>, Vec<FroggiError>> {
        crate::markup::parse::parse(&self.page)
    }

    /// Read a response from a source of bytes.
    pub fn from_bytes(bytes: &mut impl Read) -> Result<Self, FroggiError> {
        // response header, 26 bytes long
        let mut header = [0u8; PAGE_OFFSET];
        bytes.read_exact(&mut header)?;

        // version and kind are first two bytes
        let version = header[FROGGI_VERSION_OFFSET];
        let kind = header[REQUEST_RESPONSE_KIND_OFFSET].into();

        // next 16 is client ID
        let id = Uuid::from_bytes(
            header[REQUEST_RESPONSE_UUID_OFFSET..TOTAL_RESPONSE_LENGTH_OFFSET]
                .try_into()
                .unwrap(),
        );

        // next four bytes is response length
        let _total_response_length = crate::deserialize_four_bytes(
            &header[TOTAL_RESPONSE_LENGTH_OFFSET..PAGE_LENGTH_OFFSET],
        );

        // next four bytes is page length
        let page_len = crate::deserialize_four_bytes(&header[PAGE_LENGTH_OFFSET..PAGE_OFFSET])?;

        // read page
        let mut page_buf = vec![0; page_len];
        bytes.read_exact(&mut page_buf)?;
        let page = String::from_utf8(page_buf)?;

        // number of items, one byte
        let mut num_items = [0u8; NUM_ITEMS_LEN];
        bytes.read_exact(&mut num_items)?;
        let num_items = num_items[0] as usize;

        // read items
        let mut items = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            // item kind, 1 byte
            let mut item_kind = [0u8; ITEM_KIND_LEN];
            bytes.read_exact(&mut item_kind)?;
            let kind = item_kind[0].into();

            // length of the item's name, 1 byte
            let mut item_name_len = [0u8; ITEM_NAME_LENGTH_LEN];
            bytes.read_exact(&mut item_name_len)?;
            let item_name_len = item_name_len[0] as usize;

            // item name
            let mut name_buf = vec![0; item_name_len];
            bytes.read_exact(&mut name_buf)?;
            let name = String::from_utf8(name_buf)?;

            // item length, 4 bytes
            let mut item_len = [0u8; ITEM_LENGTH_LEN];
            bytes.read_exact(&mut item_len)?;
            let item_len = crate::deserialize_four_bytes(&item_len)?;

            // item
            let mut data = vec![0; item_len];
            bytes.read_exact(&mut data)?;

            items.push(Item { name, kind, data });
        }

        Ok(Self {
            version,
            kind,
            id,
            page,
            items,
        })
    }

    /// Get the version of the response
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get the kind of the response
    pub fn kind(&self) -> ResponseKind {
        self.kind
    }

    /// Get the client ID provided by the server
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the page of the response, un-parsed
    pub fn page(&self) -> &str {
        &self.page
    }

    /// Get the extra items of a page
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Convert the page into bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        // first byte: version number
        data.push(self.version);

        // next byte: response kind
        data.push(self.kind.into());

        // next 16 bytes: client ID
        data.extend_from_slice(self.id.as_bytes());

        // next four bytes: total response length
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // next four bytes: page length
        // unwrap safety - we checked that the page fits in a u32
        let page_len = crate::serialize_to_four_bytes(self.page.len()).unwrap();
        data.push(page_len[0]);
        data.push(page_len[1]);
        data.push(page_len[2]);
        data.push(page_len[3]);

        // next string: page
        data.extend_from_slice(self.page.as_bytes());

        // next byte: number of items
        // overflow safety - we checked the number of items fits in a u8
        data.push(self.items.len() as u8);

        for item in self.items.iter() {
            // next byte: item kind
            data.push(item.kind.into());

            // next byte: item name length
            // overflow safety - we checked the item name length fits in a u8
            data.push(item.name.len() as u8);

            // next string: item name
            data.extend_from_slice(item.name.as_bytes());

            // next four bytes: item length
            // unwrap safety - we checked the item size fits in a u32
            let item_len = crate::serialize_to_four_bytes(item.data.len()).unwrap();
            data.push(item_len[0]);
            data.push(item_len[1]);
            data.push(item_len[2]);
            data.push(item_len[3]);

            // next string: item data
            data.extend_from_slice(&item.data);
        }

        // unwrap safety - we checked that the size of everything can fit in a u32
        let total_len = crate::serialize_to_four_bytes(data.len()).unwrap();
        data[TOTAL_RESPONSE_LENGTH_OFFSET] = total_len[0];
        data[TOTAL_RESPONSE_LENGTH_OFFSET + 1] = total_len[1];
        data[TOTAL_RESPONSE_LENGTH_OFFSET + 2] = total_len[2];
        data[TOTAL_RESPONSE_LENGTH_OFFSET + 3] = total_len[3];

        data
    }
}

#[rustfmt::skip]
#[allow(dead_code)]
const DATA_REAL: &[u8] = &[
    0,                                                                                      // version
    0,                                                                                      // response kind
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                                         // client ID
    101, 1, 0, 0,                                                                           // total length
    60, 0, 0, 0,                                                                            // page length
    40, 105, 109, 103, 32, 34, 119, 104, 105, 116, 101, 46, 112, 110, 103, 34, 41, 10, 40,  // page
    116, 120, 116, 32, 34, 102, 117, 103, 104, 101, 100, 100, 97, 98, 111, 117, 100, 105,
    116, 34, 41, 10, 40, 105, 109, 103, 32, 34, 109, 97, 103, 101, 110, 116, 97, 46, 112,
    110, 103, 34, 41,
    2,                                                                                      // number of items
    0,                                                                                      // item kind
    9,                                                                                      // item name length
    119, 104, 105, 116, 101, 46, 112, 110, 103,                                             // item name
    119, 0, 0, 0,                                                                           // item length
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,   // white.png
    8, 2, 0, 0, 0, 144, 119, 83, 222, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233,
    0, 0, 0, 4, 103, 65, 77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89,
    115, 0, 0, 14, 192, 0, 0, 14, 192, 1, 106, 214, 137, 9, 0, 0, 0, 12, 73, 68, 65, 84,
    24, 87, 99, 248, 255, 255, 63, 0, 5, 254, 2, 254, 167, 53, 129, 132, 0, 0, 0, 0, 73,
    69, 78, 68, 174, 66, 96, 130,
    0,                                                                                      // item kind
    11,                                                                                     // item name length
    109, 97, 103, 101, 110, 116, 97, 46, 112, 110, 103,                                     // item name
    119, 0, 0, 0,                                                                           // item length
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,   // magenta.png
    8, 2, 0, 0, 0, 144, 119, 83, 222, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233, 0,
    0, 0, 4, 103, 65, 77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89, 115,
    0, 0, 14, 192, 0, 0, 14, 192, 1, 106, 214, 137, 9, 0, 0, 0, 12, 73, 68, 65, 84, 24, 87,
    99, 248, 207, 112, 7, 0, 3, 221, 1, 220, 85, 162, 120, 96, 0, 0, 0, 0, 73, 69, 78, 68,
    174, 66, 96, 130,
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        let mut bytes = DATA_REAL.clone();
        let response = Response::from_bytes(&mut bytes).unwrap();
        assert_eq!(response.version, crate::FROGGI_VERSION);
        assert_eq!(
            &response.page,
            r#"(img "white.png")
(txt "fugheddaboudit")
(img "magenta.png")"#
        );
        assert_eq!(&response.items[0].name, "white.png");
        assert_eq!(&response.items[1].name, "magenta.png");
    }

    #[test]
    fn to_bytes() {
        let white = Item::new(
            "white.png".into(),
            ItemKind::Png,
            include_bytes!("../1px_white.png").to_vec(),
        );

        let magenta = Item::new(
            "magenta.png".into(),
            ItemKind::Png,
            include_bytes!("../1px_magenta.png").to_vec(),
        );

        let page = String::from(
            r#"(img "white.png")
(txt "fugheddaboudit")
(img "magenta.png")"#,
        );

        let response =
            Response::new_with_id(ResponseKind::Page, Uuid::nil(), page, vec![white, magenta])
                .unwrap();
        let data_test: Vec<u8> = response.into();

        println!("{:?}", data_test);

        assert_eq!(data_test.len(), DATA_REAL.len());

        crate::test::test_bytes(DATA_REAL, &data_test).unwrap();
    }
}
