use crate::layout::Document;
use crate::FroggiError;

use std::io::Read;

pub struct Item {
    name: String,
    data: Vec<u8>,
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{ name: {}, data: ... }}", self.name)
    }
}

impl Item {
    pub fn new(name: String, data: Vec<u8>) -> Item {
        Item { name, data }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Represents a response from a froggi server.
#[derive(Debug)]
pub struct Response {
    version: u8,
    page: String,
    items: Vec<Item>,
}

impl Response {
    /// Create a new response.
    pub fn new(page: String, items: Vec<Item>) -> Self {
        Self {
            version: crate::FROGGI_VERSION,
            page,
            items,
        }
    }

    /// Read a response from a source of bytes.
    pub fn from_bytes(bytes: &mut impl Read) -> Result<Self, FroggiError> {
        // response header, 5 bytes long
        let mut header = [0u8; 5];
        bytes.read_exact(&mut header)?;

        // consists of version (1 byte) and page length (4 bytes)
        let version = header[0];
        let page_len = crate::deserialize_four_bytes(&header[1..]);

        // read page
        // Vec::with_capacity doesn't work here for some reason
        let mut page_buf = vec![0; page_len];
        bytes.read_exact(&mut page_buf)?;
        let page = String::from_utf8(page_buf)?;

        // number of items, two bytes
        let mut num_items = [0u8; 2];
        bytes.read_exact(&mut num_items)?;
        let num_items = crate::deserialize_bytes(&num_items);

        // read items
        let mut items = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            // length of the item's name
            let mut item_name_len = [0u8; 2];
            bytes.read_exact(&mut item_name_len)?;
            let item_name_len = crate::deserialize_bytes(&item_name_len);

            // item name
            let mut name_buf = vec![0; item_name_len];
            bytes.read_exact(&mut name_buf)?;
            let name = String::from_utf8(name_buf)?;

            // item length
            let mut item_len = [0u8; 4];
            bytes.read_exact(&mut item_len)?;
            let item_len = crate::deserialize_four_bytes(&item_len);

            // item
            let mut data = vec![0; item_len];
            bytes.read_exact(&mut data)?;

            items.push(Item { name, data });
        }

        Ok(Self {
            version,
            page,
            items,
        })
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn page(&self) -> &str {
        &self.page
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn parse_page(&self) -> Result<Document, Vec<FroggiError>> {
        Document::new(&self.page)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        // first byte: version number
        data.push(self.version);

        // next four bytes: page length
        let page_len = crate::serialize_to_four_bytes(self.page.len());
        data.push(page_len[0]);
        data.push(page_len[1]);
        data.push(page_len[2]);
        data.push(page_len[3]);

        // next string: page
        data.extend_from_slice(self.page.as_bytes());

        // next two bytes: number of items
        let (num_items_low, num_items_high) = crate::serialize_to_bytes(self.items.len());
        data.push(num_items_low);
        data.push(num_items_high);

        for item in self.items.iter() {
            // next two bytes: item name length
            let (item_name_low, item_name_high) = crate::serialize_to_bytes(item.name.len());
            data.push(item_name_low);
            data.push(item_name_high);

            // next string: item name
            data.extend_from_slice(item.name.as_bytes());

            // next four bytes: item length
            let item_len = crate::serialize_to_four_bytes(item.data.len());
            data.push(item_len[0]);
            data.push(item_len[1]);
            data.push(item_len[2]);
            data.push(item_len[3]);

            // next string: item data
            data.extend_from_slice(&item.data);
        }

        assert!(data.len() <= u32::MAX as usize);

        data
    }
}

#[rustfmt::skip]
pub const DATA_REAL: &[u8] = &[
    0x00,                                                                                   // version
    0x3c, 0x00, 0x00, 0x00,                                                                 // page len
    0x28, 0x69, 0x6d, 0x67, 0x20, 0x22, 0x77, 0x68, 0x69, 0x74, 0x65, 0x2e, 0x70, 0x6e,
    0x67, 0x22, 0x29, 0x0a, 0x28, 0x74, 0x78, 0x74, 0x20, 0x22, 0x66, 0x75, 0x67, 0x68,
    0x65, 0x64, 0x64, 0x61, 0x62, 0x6f, 0x75, 0x64, 0x69, 0x74, 0x22, 0x29, 0x0a, 0x28,
    0x69, 0x6d, 0x67, 0x20, 0x22, 0x6d, 0x61, 0x67, 0x65, 0x6e, 0x74, 0x61, 0x2e, 0x70,
    0x6e, 0x67, 0x22, 0x29,
    0x02, 0x00,                                                                             // number of items
    0x09, 0x00,                                                                             // item name len
    0x77, 0x68, 0x69, 0x74, 0x65, 0x2e, 0x70, 0x6e, 0x67,                                   // item name
    0x77, 0x00, 0x00, 0x00,                                                                 // item len
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,     // item
    0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
    0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00,
    0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00, 0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00,
    0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00, 0x00, 0x09, 0x70, 0x48, 0x59, 0x73,
    0x00, 0x00, 0x0e, 0xc1, 0x00, 0x00, 0x0e, 0xc1, 0x01, 0xb8, 0x91, 0x6b, 0xed, 0x00,
    0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0xf8, 0xff, 0xff, 0x3f,
    0x00, 0x05, 0xfe, 0x02, 0xfe, 0xa7, 0x35, 0x81, 0x84, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    0x0b, 0x00,                                                                             // item name len
    0x6d, 0x61, 0x67, 0x65, 0x6e, 0x74, 0x61, 0x2e, 0x70, 0x6e, 0x67,                       // item name
    0x77, 0x00, 0x00, 0x00,                                                                 // item len
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,     // item
    0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
    0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00,
    0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00, 0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00,
    0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00, 0x00, 0x09, 0x70, 0x48, 0x59, 0x73,
    0x00, 0x00, 0x0e, 0xc2, 0x00, 0x00, 0x0e, 0xc2, 0x01, 0x15, 0x28, 0x4a, 0x80, 0x00,
    0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0xf8, 0x6f, 0xfe, 0x0a,
    0x00, 0x04, 0x59, 0x02, 0x21, 0x08, 0x92, 0x4b, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82u8,
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
            "(img \"white.png\")\n(txt \"fugheddaboudit\")\n(img \"magenta.png\")"
        );
        assert_eq!(&response.items[0].name, "white.png");
        assert_eq!(&response.items[1].name, "magenta.png");
    }

    #[test]
    fn to_bytes() {
        let white = Item::new(
            "white.png".into(),
            include_bytes!("../1px_white.png").to_vec(),
        );
        let magenta = Item::new(
            "magenta.png".into(),
            include_bytes!("../1px_magenta.png").to_vec(),
        );

        let page =
            String::from("(img \"white.png\")\n(txt \"fugheddaboudit\")\n(img \"magenta.png\")");
        let response = Response::new(page, vec![white, magenta]);
        let data_test: Vec<u8> = response.into();

        assert_eq!(data_test.len(), DATA_REAL.len());

        crate::test::test_bytes(DATA_REAL, &data_test).unwrap();
    }
}
