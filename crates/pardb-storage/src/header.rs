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

use crate::page::{PAGE_SIZE, PageId};

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
}
