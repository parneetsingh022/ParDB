//! Physical page definitions for ParDB.
//!
//! Pages are fixed-size units of storage used by the
//! database's storage engine.
use crate::errors::PageError;

/// Size of a database page in bytes.
pub const PAGE_SIZE: usize = 4096;

/// Uniquely identifies a page within a database file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(pub u64);

impl PageId {
    /// Returns the byte offset of this page within the database file.
    ///
    /// Pages are stored consecutively, each occupying [`PAGE_SIZE`] bytes.
    ///
    /// Returns [`None`] if the calculated offset exceeds `u64::MAX`.
    pub fn offset(&self) -> Option<u64> {
        self.0.checked_mul(PAGE_SIZE as u64)
    }
}

/// Identifies the purpose of a page within a ParDB database file.
///
/// Each page type determines how the storage engine interprets
/// and manages the data contained within that page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    /// Stores database metadata, including table definitions,
    /// schemas, and index information.
    Catalog,

    /// Stores database records belonging to a table.
    ///
    /// A data page may contain multiple records from the
    /// same table.
    Data,

    /// Represents an unallocated or released page.
    ///
    /// Free pages do not contain active database records
    /// and may be reused for future allocations.
    Free,
}

/// Represents a fixed-size page of database storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new() -> Self {
        Self {
            data: [0; PAGE_SIZE],
        }
    }

    /// Creates a page from an existing fixed-size byte buffer.
    ///
    /// The provided bytes are stored without modification or validation.
    /// Higher-level storage components are responsible for interpreting
    /// and validating the page's contents.
    pub fn from_bytes(data: [u8; PAGE_SIZE]) -> Self {
        Self { data }
    }

    /// Returns the page's raw bytes.
    pub fn as_bytes(&self) -> &[u8; PAGE_SIZE] {
        &self.data
    }

    /// Returns a slice of bytes from the specified page region.
    ///
    /// # Errors
    ///
    /// Returns [`PageError::OutOfBounds`] if the requested
    /// region extends beyond the page.
    pub fn read(&self, offset: usize, length: usize) -> Result<&[u8], PageError> {
        let end = offset.checked_add(length).ok_or(PageError::OutOfBounds)?;

        self.data.get(offset..end).ok_or(PageError::OutOfBounds)
    }

    /// Writes bytes into the page at the specified offset.
    ///
    /// # Errors
    ///
    /// Returns [`PageError::OutOfBounds`] if the data
    /// does not fit within the page.
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), PageError> {
        let end = offset
            .checked_add(data.len())
            .ok_or(PageError::OutOfBounds)?;

        let destination = self
            .data
            .get_mut(offset..end)
            .ok_or(PageError::OutOfBounds)?;

        destination.copy_from_slice(data);

        Ok(())
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ---------------------------------------------------------
    // PageId Offset Tests
    // ---------------------------------------------------------

    #[test]
    fn page_zero_has_zero_offset() {
        assert_eq!(PageId(0).offset(), Some(0));
    }

    #[test]
    fn page_one_has_correct_offset() {
        assert_eq!(PageId(1).offset(), Some(4096));
    }

    #[test]
    fn page_multiple_has_correct_offset() {
        assert_eq!(PageId(10).offset(), Some(40960));
        assert_eq!(PageId(100).offset(), Some(409600));
        assert_eq!(PageId(1000).offset(), Some(4096000));
    }

    #[test]
    fn consecutive_pages_have_correct_spacing() {
        for id in 0..100 {
            let current = PageId(id).offset().unwrap();
            let next = PageId(id + 1).offset().unwrap();

            assert_eq!(next - current, PAGE_SIZE as u64);
        }
    }

    #[test]
    fn maximum_valid_page_id_does_not_overflow() {
        let max_id = u64::MAX / PAGE_SIZE as u64;

        assert_eq!(PageId(max_id).offset(), Some(max_id * PAGE_SIZE as u64));
    }

    #[test]
    fn page_id_above_maximum_returns_none() {
        let max_id = u64::MAX / PAGE_SIZE as u64;

        assert_eq!(PageId(max_id + 1).offset(), None);
    }

    #[test]
    fn maximum_u64_page_id_returns_none() {
        assert_eq!(PageId(u64::MAX).offset(), None);
    }

    #[test]
    fn page_offset_never_panics_on_overflow() {
        let ids = [
            PageId(0),
            PageId(1),
            PageId(u64::MAX / PAGE_SIZE as u64),
            PageId(u64::MAX),
        ];

        for id in ids {
            let result = std::panic::catch_unwind(|| id.offset());

            assert!(result.is_ok());
        }
    }

    // ---------------------------------------------------------
    // PageId Trait Tests
    // ---------------------------------------------------------

    #[test]
    fn page_ids_with_same_value_are_equal() {
        assert_eq!(PageId(5), PageId(5));
    }

    #[test]
    fn page_ids_with_different_values_are_not_equal() {
        assert_ne!(PageId(5), PageId(10));
    }

    #[test]
    fn page_id_supports_copy() {
        let original = PageId(42);
        let copied = original;

        assert_eq!(original, copied);
    }

    #[test]
    fn page_id_can_be_used_in_hashset() {
        let mut ids = HashSet::new();

        assert!(ids.insert(PageId(1)));
        assert!(ids.insert(PageId(2)));

        // Duplicate identifiers should not be inserted.
        assert!(!ids.insert(PageId(1)));

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&PageId(1)));
        assert!(ids.contains(&PageId(2)));
    }

    // ---------------------------------------------------------
    // PageType Tests
    // ---------------------------------------------------------

    #[test]
    fn page_type_debug_representation() {
        assert_eq!(format!("{:?}", PageType::Catalog), "Catalog");
        assert_eq!(format!("{:?}", PageType::Data), "Data");
        assert_eq!(format!("{:?}", PageType::Free), "Free");
    }

    // ---------------------------------------------------------
    // Page Initialization Tests
    // ---------------------------------------------------------

    #[test]
    fn new_page_has_correct_size() {
        let page = Page::new();

        assert_eq!(page.as_bytes().len(), PAGE_SIZE);
    }

    #[test]
    fn new_page_is_zero_initialized() {
        let page = Page::new();

        assert!(page.as_bytes().iter().all(|&byte| byte == 0));
    }

    #[test]
    fn default_page_is_zero_initialized() {
        let page = Page::default();

        assert_eq!(page, Page::new());
        assert!(page.as_bytes().iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_bytes_preserves_data() {
        let data = [0xAB; PAGE_SIZE];

        let page = Page::from_bytes(data);

        assert_eq!(page.as_bytes(), &data);
    }

    #[test]
    fn from_bytes_accepts_zero_initialized_buffer() {
        let page = Page::from_bytes([0; PAGE_SIZE]);

        assert_eq!(page, Page::new());
    }

    // ---------------------------------------------------------
    // Page Write Tests
    // ---------------------------------------------------------

    #[test]
    fn write_single_byte() {
        let mut page = Page::new();

        assert_eq!(page.write(0, &[42]), Ok(()));
        assert_eq!(page.as_bytes()[0], 42);
    }

    #[test]
    fn write_first_and_last_byte() {
        let mut page = Page::new();

        page.write(0, &[10]).unwrap();
        page.write(PAGE_SIZE - 1, &[255]).unwrap();

        assert_eq!(page.as_bytes()[0], 10);
        assert_eq!(page.as_bytes()[PAGE_SIZE - 1], 255);
    }

    #[test]
    fn write_multiple_bytes() {
        let mut page = Page::new();
        let data = [10, 20, 30, 40, 50];

        page.write(0, &data).unwrap();

        assert_eq!(page.read(0, data.len()).unwrap(), &data);
    }

    #[test]
    fn write_entire_page() {
        let mut page = Page::new();
        let data = [0xAB; PAGE_SIZE];

        page.write(0, &data).unwrap();

        assert_eq!(page.as_bytes(), &data);
    }

    #[test]
    fn write_at_middle_of_page() {
        let mut page = Page::new();
        let data = [1, 2, 3, 4];

        page.write(100, &data).unwrap();

        assert_eq!(page.read(100, data.len()).unwrap(), &data);
    }

    #[test]
    fn overwriting_bytes_updates_data() {
        let mut page = Page::new();

        page.write(100, &[10]).unwrap();
        page.write(100, &[20]).unwrap();

        assert_eq!(page.read(100, 1).unwrap(), &[20]);
    }

    #[test]
    fn modifying_bytes_preserves_other_bytes() {
        let mut page = Page::new();

        page.write(100, &[42]).unwrap();

        assert_eq!(page.read(100, 1).unwrap(), &[42]);

        assert!(
            page.as_bytes()
                .iter()
                .enumerate()
                .all(|(index, &byte)| index == 100 || byte == 0)
        );
    }

    #[test]
    fn write_binary_data() {
        let mut page = Page::new();

        let binary_data = [0x00, 0xFF, 0x7F, 0x80, 0x01, 0xFE, 0xAA, 0x55];

        page.write(0, &binary_data).unwrap();

        assert_eq!(page.read(0, binary_data.len()).unwrap(), &binary_data);
    }

    // ---------------------------------------------------------
    // Page Read Tests
    // ---------------------------------------------------------

    #[test]
    fn read_single_byte() {
        let mut page = Page::new();

        page.write(50, &[99]).unwrap();

        assert_eq!(page.read(50, 1).unwrap(), &[99]);
    }

    #[test]
    fn read_multiple_bytes() {
        let mut page = Page::new();
        let data = [10, 20, 30, 40];

        page.write(200, &data).unwrap();

        assert_eq!(page.read(200, 4).unwrap(), &data);
    }

    #[test]
    fn read_entire_page() {
        let data = [0xCC; PAGE_SIZE];
        let page = Page::from_bytes(data);

        assert_eq!(page.read(0, PAGE_SIZE).unwrap(), &data);
    }

    #[test]
    fn read_last_byte() {
        let mut page = Page::new();

        page.write(PAGE_SIZE - 1, &[255]).unwrap();

        assert_eq!(page.read(PAGE_SIZE - 1, 1).unwrap(), &[255]);
    }

    #[test]
    fn read_zero_initialized_region() {
        let page = Page::new();

        assert_eq!(page.read(100, 4).unwrap(), &[0, 0, 0, 0]);
    }

    #[test]
    fn read_does_not_modify_page() {
        let mut page = Page::new();
        page.write(100, &[1, 2, 3]).unwrap();

        let original = page.clone();

        let _ = page.read(100, 3).unwrap();

        assert_eq!(page, original);
    }

    // ---------------------------------------------------------
    // Page Boundary Validation Tests
    // ---------------------------------------------------------

    #[test]
    fn read_beyond_page_boundary_returns_error() {
        let page = Page::new();

        assert_eq!(page.read(PAGE_SIZE - 2, 4), Err(PageError::OutOfBounds));
    }

    #[test]
    fn write_beyond_page_boundary_returns_error() {
        let mut page = Page::new();

        assert_eq!(
            page.write(PAGE_SIZE - 2, &[1, 2, 3, 4]),
            Err(PageError::OutOfBounds)
        );
    }

    #[test]
    fn read_offset_beyond_page_returns_error() {
        let page = Page::new();

        assert_eq!(page.read(PAGE_SIZE + 1, 1), Err(PageError::OutOfBounds));
    }

    #[test]
    fn write_offset_beyond_page_returns_error() {
        let mut page = Page::new();

        assert_eq!(page.write(PAGE_SIZE + 1, &[1]), Err(PageError::OutOfBounds));
    }

    #[test]
    fn read_integer_overflow_returns_error() {
        let page = Page::new();

        assert_eq!(page.read(usize::MAX, 10), Err(PageError::OutOfBounds));
    }

    #[test]
    fn write_integer_overflow_returns_error() {
        let mut page = Page::new();

        assert_eq!(
            page.write(usize::MAX, &[1, 2, 3]),
            Err(PageError::OutOfBounds)
        );
    }

    #[test]
    fn failed_write_does_not_modify_page() {
        let mut page = Page::new();

        page.write(100, &[10, 20, 30]).unwrap();

        let original = page.clone();

        assert_eq!(
            page.write(PAGE_SIZE - 1, &[1, 2, 3]),
            Err(PageError::OutOfBounds)
        );

        assert_eq!(page, original);
    }

    #[test]
    fn read_empty_slice_at_valid_offset() {
        let page = Page::new();

        assert_eq!(page.read(100, 0).unwrap(), &[]);
    }

    #[test]
    fn write_empty_slice_at_valid_offset() {
        let mut page = Page::new();
        let original = page.clone();

        assert_eq!(page.write(100, &[]), Ok(()));
        assert_eq!(page, original);
    }

    #[test]
    fn read_empty_slice_at_page_end() {
        let page = Page::new();

        assert_eq!(page.read(PAGE_SIZE, 0).unwrap(), &[]);
    }

    #[test]
    fn write_empty_slice_at_page_end() {
        let mut page = Page::new();

        assert_eq!(page.write(PAGE_SIZE, &[]), Ok(()));
    }

    // ---------------------------------------------------------
    // Page Clone and Equality Tests
    // ---------------------------------------------------------

    #[test]
    fn cloned_page_equals_original() {
        let mut original = Page::new();
        original.write(100, &[42]).unwrap();

        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn modifying_cloned_page_does_not_modify_original() {
        let mut original = Page::new();
        original.write(100, &[42]).unwrap();

        let mut cloned = original.clone();
        cloned.write(100, &[99]).unwrap();

        assert_eq!(original.read(100, 1).unwrap(), &[42]);
        assert_eq!(cloned.read(100, 1).unwrap(), &[99]);
        assert_ne!(original, cloned);
    }

    #[test]
    fn pages_with_identical_data_are_equal() {
        let mut page1 = Page::new();
        let mut page2 = Page::new();

        page1.write(50, &[100]).unwrap();
        page2.write(50, &[100]).unwrap();

        assert_eq!(page1, page2);
    }

    #[test]
    fn pages_with_different_data_are_not_equal() {
        let mut page1 = Page::new();
        let mut page2 = Page::new();

        page1.write(50, &[100]).unwrap();
        page2.write(50, &[200]).unwrap();

        assert_ne!(page1, page2);
    }

    #[test]
    fn from_bytes_page_equals_written_page() {
        let data = [0xAB; PAGE_SIZE];

        let from_bytes = Page::from_bytes(data);

        let mut written = Page::new();
        written.write(0, &data).unwrap();

        assert_eq!(from_bytes, written);
    }
}
