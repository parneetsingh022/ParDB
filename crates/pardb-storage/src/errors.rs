/// Errors encountered while accessing a database page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageError {
    /// A read or write extends beyond the page boundary.
    OutOfBounds,
}

/// Errors encountered while encoding, decoding, or validating
/// a ParDB database file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// An error occurred while accessing the header page.
    Page(PageError),

    /// The database file contains invalid magic bytes.
    InvalidMagic,

    /// The database uses an unsupported file format version.
    UnsupportedVersion {
        /// Version found in the database header.
        found: u16,
    },

    /// The database specifies an unsupported page size.
    InvalidPageSize {
        /// Page size found in the database header.
        found: u16,
    },

    /// The database header specifies an invalid page count.
    InvalidPageCount,

    /// The catalog root references an invalid page.
    InvalidCatalogRoot,
}

impl From<PageError> for HeaderError {
    fn from(error: PageError) -> Self {
        Self::Page(error)
    }
}
