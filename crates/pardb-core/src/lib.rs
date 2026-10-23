//! Core data structures, schemas, and validation for ParDB.
//!
//! This crate defines the fundamental types shared across
//! ParDB's database engine and storage components.

mod error;
mod row;
mod schema;
mod table;
mod types;
mod validation;

// Error types
pub use error::{RowValidationError, SchemaError};

// Row representation
pub use row::Row;

// Schema definitions
pub use schema::{Column, ColumnId, ConstraintKind, Schema, TableConstraint};

// Table metadata
pub use table::{Table, TableId};

// Database value types
pub use types::{DataType, Value};
