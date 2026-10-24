/// Errors encountered while accessing a database page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageError {
    /// A read or write extends beyond the page boundary.
    OutOfBounds,
}
