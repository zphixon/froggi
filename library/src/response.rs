use crate::FroggiError;

use std::io::Write;

pub struct Item {
    name: String,
    data: Vec<u8>,
}

impl Item {
    pub fn new(name: String, data: Vec<u8>) -> Item {
        Item { name, data }
    }
}

pub struct Response {
    version: u8,
    page: String,
    items: Vec<Item>,
}

// page:
// (img "lol.png") (img "xd.png")

// response bytes
// version          : 00
// page             : 1e 00
//                    28 69 6d 67 20 22 6c 6f 6c 2e 70 6e 67 22 29 20
//                    28 69 6d 67 20 22 78 64 2e 70 6e 67 22 29
// items            : 02                                                  -- number of items
//                    07 00 6c 6f 6c 2e 70 6e 67                          -- filename len, filename
//                    77 00                                               -- file data len
//                    50 89 47 4e 0a 0d 0a 1a 00 00 0d 00 48 49 52 44     -- file data
//                    00 00 01 00 00 00 01 00 02 08 00 00 90 00 53 77
//                    00 de 00 00 73 01 47 52 00 42 ce ae e9 1c 00 00
//                    04 00 41 67 41 4d 00 00 8f b1 fc 0b 05 61 00 00
//                    09 00 48 70 73 59 00 00 c3 0e 00 00 c3 0e c7 01
//                    a8 6f 00 64 00 00 49 0c 41 44 18 54 63 57 ff f8
//                    3f ff 05 00 02 fe a7 fe 81 35 00 84 00 00 49 00
//                    4e 45 ae 44 60 42 00 82
//                    06 00 78 64 2e 70 6e 67                             -- second filename len, filename
//                    77 00                                               -- second file data len
//                    50 89 47 4e 0a 0d 0a 1a 00 00 0d 00 48 49 52 44     -- second file data
//                    00 00 01 01 00 00 01 00 02 08 00 00 90 00 53 77
//                    00 de 00 00 73 01 47 52 00 42 ce ae e9 1c 00 00
//                    04 00 41 67 41 4d 00 00 8f b1 fc 0b 05 61 00 00
//                    09 00 48 70 73 59 00 00 c2 0e 00 00 c2 0e 15 01
//                    4a 28 00 80 00 00 49 0c 41 44 18 54 63 57 6f f8
//                    0a fe 04 00 02 59 08 21 4b 92 00 1a 00 00 49 00
//                    4e 45 ae 44 60 42 00 82

impl Response {
    pub fn new(page: String, items: Vec<Item>) -> Self {
        Self {
            version: crate::FROGGI_VERSION,
            page,
            items,
        }
    }

    pub fn from_bytes(_bytes: &mut impl Write) -> Result<Self, FroggiError> {
        // TODO
        Ok(Self {
            version: crate::FROGGI_VERSION,
            page: String::from(""),
            items: vec![],
        })
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        // first byte: version number
        data.push(self.version);

        // next two bytes: page length
        let (page_len_low, page_len_high) = crate::serialize_to_bytes(self.page.len());
        data.push(page_len_low);
        data.push(page_len_high);

        // next string: page
        data.extend_from_slice(self.page.as_bytes());

        // next byte: number of items
        data.push(self.items.len() as u8);
        for item in self.items.iter() {
            // next bytes: item name length
            let (item_name_low, item_name_high) = crate::serialize_to_bytes(item.name.len());
            data.push(item_name_low);
            data.push(item_name_high);

            // next string: item name
            data.extend_from_slice(item.name.as_bytes());

            // next bytes: item length
            let (item_len_low, item_len_high) = crate::serialize_to_bytes(item.data.len());
            data.push(item_len_low);
            data.push(item_len_high);

            // next string: item data
            data.extend_from_slice(&item.data);
        }

        assert!(data.len() <= u32::MAX as usize);

        data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
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

        let data_real = vec![
            0x00, // version
            0x3c, 0x00, // page len
            0x28, 0x69, 0x6d, 0x67, 0x20, 0x22, 0x77, 0x68, 0x69, 0x74, 0x65, 0x2e, 0x70, 0x6e,
            0x67, 0x22, 0x29, 0x0a, 0x28, 0x74, 0x78, 0x74, 0x20, 0x22, 0x66, 0x75, 0x67, 0x68,
            0x65, 0x64, 0x64, 0x61, 0x62, 0x6f, 0x75, 0x64, 0x69, 0x74, 0x22, 0x29, 0x0a, 0x28,
            0x69, 0x6d, 0x67, 0x20, 0x22, 0x6d, 0x61, 0x67, 0x65, 0x6e, 0x74, 0x61, 0x2e, 0x70,
            0x6e, 0x67, 0x22, 0x29, 0x02, // number of items
            0x09, 0x00, // item name len
            0x77, 0x68, 0x69, 0x74, 0x65, 0x2e, 0x70, 0x6e, 0x67, // item name
            0x77, 0x00, // item len
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00,
            0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00, 0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00,
            0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00, 0x00, 0x09, 0x70, 0x48, 0x59, 0x73,
            0x00, 0x00, 0x0e, 0xc1, 0x00, 0x00, 0x0e, 0xc1, 0x01, 0xb8, 0x91, 0x6b, 0xed, 0x00,
            0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0xf8, 0xff, 0xff, 0x3f,
            0x00, 0x05, 0xfe, 0x02, 0xfe, 0xa7, 0x35, 0x81, 0x84, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82, 0x0b, 0x00, // item name len
            0x6d, 0x61, 0x67, 0x65, 0x6e, 0x74, 0x61, 0x2e, 0x70, 0x6e, 0x67, // item name
            0x77, 0x00, // item len
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00,
            0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00, 0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00,
            0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00, 0x00, 0x09, 0x70, 0x48, 0x59, 0x73,
            0x00, 0x00, 0x0e, 0xc2, 0x00, 0x00, 0x0e, 0xc2, 0x01, 0x15, 0x28, 0x4a, 0x80, 0x00,
            0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0xf8, 0x6f, 0xfe, 0x0a,
            0x00, 0x04, 0x59, 0x02, 0x21, 0x08, 0x92, 0x4b, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82u8,
        ];

        assert_eq!(data_test.len(), data_real.len());

        let mut had_error = false;
        for (i, (byte_test, byte_real)) in data_test.iter().zip(data_real.iter()).enumerate() {
            if byte_test != byte_real {
                eprintln!(
                    "error in byte {}, test {:x} != real {:x}",
                    i, byte_test, byte_real
                );
                had_error = true;
            }
        }
        assert!(!had_error);
    }
}
