//! Database file header definitions for ParDB.
//!
//! This module defines the metadata stored in the first page
//! of a ParDB database file.
//!
//! The database header identifies the file format, records its
//! version, and provides information required to locate and
//! manage database pages.
//!
//! All ParDB database files begin with a fixed-size header
//! stored in [`PageId(0)`].
//!
//! Multi-byte integer fields are encoded in little-endian
//! byte order when written to disk.

use crate::{
    errors::PageError,
    page::{PAGE_SIZE, Page, PageId},
};

pub const MAGIC: [u8; 4] = *b"PDB1";
pub const FORMAT_VERSION: u16 = 1;

/// Represents the metadata stored in a ParDB database header.
///
/// The header occupies the first page of the database file
/// and contains information required to interpret its contents.
///
/// The header is serialized explicitly to ensure a stable
/// binary representation independent of Rust's memory layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseHeader {
    /// Magic bytes identifying the database file as ParDB.
    ///
    /// Must match [`MAGIC`] for the file to be considered valid.
    pub magic: [u8; 4],

    /// Version of the database file format.
    pub version: u16,

    /// Size of each database page in bytes.
    pub page_size: u16,

    /// Total number of allocated pages in the database file.
    pub page_count: u64,

    /// Root page of the database catalog.
    ///
    /// The catalog stores metadata about tables, schemas,
    /// constraints, and indexes.
    ///
    /// [`None`] indicates that no catalog page has been allocated.
    pub catalog_root: Option<PageId>,
}

impl DatabaseHeader {
    pub fn encode(&self) -> Result<Page, PageError> {
        let mut page = Page::new();
        page.write(0, &self.magic)?; // bytes 0-3
        page.write(4, &self.version.to_le_bytes())?; // bytes 4-5
        page.write(6, &self.page_size.to_le_bytes())?; // bytes 6-7
        page.write(8, &self.page_count.to_le_bytes())?; // bytes 8-15

        // Zero means that no catalog root is currently assigned.
        let catalog_root = self.catalog_root.map_or(0, |page_id| page_id.0);

        page.write(16, &catalog_root.to_le_bytes())?; // bytes 16-23

        Ok(page)
    }

    pub fn decode(page: &Page) -> Result<Self, PageError> {
        // Bytes 0-3: Magic number
        let magic = page.read(0, 4)?.try_into().unwrap();

        // Bytes 4-5: Format version
        let version = u16::from_le_bytes(page.read(4, 2)?.try_into().unwrap());

        // Bytes 6-7: Page size
        let page_size = u16::from_le_bytes(page.read(6, 2)?.try_into().unwrap());

        // Bytes 8-15: Total page count
        let page_count = u64::from_le_bytes(page.read(8, 8)?.try_into().unwrap());

        // Bytes 16-23: Catalog root page ID
        let root = u64::from_le_bytes(page.read(16, 8)?.try_into().unwrap());

        let catalog_root = match root {
            0 => None,
            id => Some(PageId(id)),
        };

        let header = Self {
            magic,
            version,
            page_size,
            page_count,
            catalog_root,
        };

        Ok(header)
    }
}

impl Default for DatabaseHeader {
    /// Creates a header containing the default ParDB file metadata.
    ///
    /// A new header uses the current format version, the configured
    /// page size, and a single allocated header page.
    ///
    /// The catalog root is initially unset.
    fn default() -> Self {
        Self {
            magic: MAGIC,
            version: FORMAT_VERSION,
            page_size: PAGE_SIZE as u16,
            page_count: 1,
            catalog_root: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_header_has_correct_magic() {
        let header = DatabaseHeader::default();

        assert_eq!(header.magic, MAGIC);
    }

    #[test]
    fn default_header_has_current_format_version() {
        let header = DatabaseHeader::default();

        assert_eq!(header.version, FORMAT_VERSION);
    }

    #[test]
    fn default_header_has_correct_page_size() {
        let header = DatabaseHeader::default();

        assert_eq!(header.page_size, PAGE_SIZE as u16);
    }

    #[test]
    fn default_header_has_one_allocated_page() {
        let header = DatabaseHeader::default();

        assert_eq!(header.page_count, 1);
    }

    #[test]
    fn default_header_has_no_catalog_root() {
        let header = DatabaseHeader::default();

        assert_eq!(header.catalog_root, None);
    }

    #[test]
    fn cloned_header_equals_original() {
        let original = DatabaseHeader::default();
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn headers_with_different_metadata_are_not_equal() {
        let original = DatabaseHeader::default();

        let modified = DatabaseHeader {
            page_count: 5,
            catalog_root: Some(PageId(1)),
            ..DatabaseHeader::default()
        };

        assert_ne!(original, modified);
    }

    #[test]
    fn header_supports_catalog_root() {
        let header = DatabaseHeader {
            page_count: 2,
            catalog_root: Some(PageId(1)),
            ..DatabaseHeader::default()
        };

        assert_eq!(header.catalog_root, Some(PageId(1)));
    }

    // -------------------------------------------
    //  HEADER ENCODE TESTS
    // -------------------------------------------
    #[test]
    fn encode_writes_header_fields_at_expected_offsets() {
        let header = DatabaseHeader {
            magic: *b"PDB1",
            version: 1,
            page_size: 4096,
            page_count: 25,
            catalog_root: Some(PageId(3)),
        };

        let page = header.encode().expect("header encoding should succeed");

        assert_eq!(page.read(0, 4).unwrap(), b"PDB1");
        assert_eq!(page.read(4, 2).unwrap(), 1u16.to_le_bytes());
        assert_eq!(page.read(6, 2).unwrap(), 4096u16.to_le_bytes());
        assert_eq!(page.read(8, 8).unwrap(), 25u64.to_le_bytes());
        assert_eq!(page.read(16, 8).unwrap(), 3u64.to_le_bytes());
    }

    #[test]
    fn encode_writes_zero_when_catalog_root_is_none() {
        let header = DatabaseHeader {
            magic: *b"PDB1",
            version: 1,
            page_size: 4096,
            page_count: 1,
            catalog_root: None,
        };

        let page = header.encode().expect("header encoding should succeed");

        assert_eq!(page.read(16, 8).unwrap(), 0u64.to_le_bytes());
    }

    #[test]
    fn encode_writes_little_endian_values() {
        let header = DatabaseHeader {
            magic: *b"PDB1",
            version: 0x1234,
            page_size: 0x5678,
            page_count: 0x0102_0304_0506_0708,
            catalog_root: Some(PageId(9)),
        };

        let page = header.encode().expect("header encoding should succeed");

        assert_eq!(page.read(4, 2).unwrap(), [0x34, 0x12]);
        assert_eq!(page.read(6, 2).unwrap(), [0x78, 0x56]);
        assert_eq!(
            page.read(8, 8).unwrap(),
            [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    // ---------------------------------------------------------
    // Header Decoding Tests
    // ---------------------------------------------------------

    #[test]
    fn decode_default_header() {
        let original = DatabaseHeader::default();
        let page = original.encode().unwrap();

        let decoded = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn decode_header_with_catalog_root() {
        let original = DatabaseHeader {
            page_count: 10,
            catalog_root: Some(PageId(3)),
            ..DatabaseHeader::default()
        };

        let page = original.encode().unwrap();
        let decoded = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(decoded.catalog_root, Some(PageId(3)));
        assert_eq!(decoded.page_count, 10);
    }

    #[test]
    fn decode_header_without_catalog_root() {
        let original = DatabaseHeader {
            page_count: 10,
            catalog_root: None,
            ..DatabaseHeader::default()
        };

        let page = original.encode().unwrap();
        let decoded = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(decoded.catalog_root, None);
    }

    #[test]
    fn decode_reads_fields_at_expected_offsets() {
        let mut page = Page::new();

        page.write(0, b"PDB1").unwrap();
        page.write(4, &1u16.to_le_bytes()).unwrap();
        page.write(6, &4096u16.to_le_bytes()).unwrap();
        page.write(8, &10u64.to_le_bytes()).unwrap();
        page.write(16, &3u64.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.magic, *b"PDB1");
        assert_eq!(header.version, 1);
        assert_eq!(header.page_size, 4096);
        assert_eq!(header.page_count, 10);
        assert_eq!(header.catalog_root, Some(PageId(3)));
    }

    #[test]
    fn decode_uses_little_endian_encoding() {
        let mut page = Page::new();

        page.write(0, b"PDB1").unwrap();
        page.write(4, &[0x02, 0x01]).unwrap();
        page.write(6, &[0x00, 0x10]).unwrap();
        page.write(8, &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01])
            .unwrap();
        page.write(16, &5u64.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.version, 0x0102);
        assert_eq!(header.page_size, 4096);
        assert_eq!(header.page_count, 0x0102030405060708);
        assert_eq!(header.catalog_root, Some(PageId(5)));
    }

    #[test]
    fn decode_zero_catalog_root_as_none() {
        let mut page = Page::new();

        page.write(16, &0u64.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.catalog_root, None);
    }

    #[test]
    fn decode_nonzero_catalog_root_as_page_id() {
        let mut page = Page::new();

        page.write(16, &42u64.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.catalog_root, Some(PageId(42)));
    }

    #[test]
    fn decode_maximum_page_count() {
        let mut page = Page::new();

        page.write(8, &u64::MAX.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.page_count, u64::MAX);
    }

    #[test]
    fn decode_preserves_invalid_magic() {
        let mut page = Page::new();

        page.write(0, b"XXXX").unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.magic, *b"XXXX");
    }

    #[test]
    fn decode_preserves_unsupported_version() {
        let mut page = Page::new();

        page.write(4, &99u16.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.version, 99);
    }

    #[test]
    fn decode_preserves_invalid_page_size() {
        let mut page = Page::new();

        page.write(6, &8192u16.to_le_bytes()).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header.page_size, 8192);
    }

    #[test]
    fn decode_does_not_modify_page() {
        let original = DatabaseHeader::default();
        let page = original.encode().unwrap();
        let snapshot = page.clone();

        let _ = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(page, snapshot);
    }

    #[test]
    fn decode_ignores_reserved_bytes() {
        let mut page = DatabaseHeader::default().encode().unwrap();

        page.write(24, &[0xAA, 0xBB, 0xCC]).unwrap();

        let header = DatabaseHeader::decode(&page).unwrap();

        assert_eq!(header, DatabaseHeader::default());
    }

    #[test]
    fn encode_decode_round_trip_with_custom_metadata() {
        let original = DatabaseHeader {
            page_count: 100,
            catalog_root: Some(PageId(25)),
            ..DatabaseHeader::default()
        };

        let encoded = original.encode().unwrap();
        let decoded = DatabaseHeader::decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }
}
