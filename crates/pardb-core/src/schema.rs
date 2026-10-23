//! Table schema definitions and constraints for ParDB.
//!
//! This module defines the logical structure of database tables,
//! including column metadata, data types, and table-level constraints.
//!
//! A [`Schema`] contains an ordered collection of [`Column`] definitions
//! and a set of [`TableConstraint`] values that describe the rules
//! governing the table.
//!
//! Column references use [`ColumnId`] rather than column names, allowing
//! constraints to refer to columns independently of their names.

use crate::types::DataType;

/// Uniquely identifies a column within a table.
///
/// Column identifiers are scoped to an individual table. Different
/// tables may therefore contain columns with the same identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnId(pub u32);

/// Defines the metadata associated with a table column.
///
/// Each column has an identifier, a name, a data type, and a nullability
/// flag. The column definition describes the expected structure of
/// values stored in that column, but does not contain actual row data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// Unique identifier of the column within its table.
    pub id: ColumnId,

    /// Name of the column.
    pub name: String,

    /// Data type accepted by the column.
    pub data_type: DataType,

    /// Whether the column permits `NULL` values.
    pub nullable: bool,
}

/// Defines additional constraints on a table.
///
/// Table constraints describe rules that must be satisfied
/// by the data stored in a table.
///
/// A table may contain multiple constraints, including
/// single-column and composite uniqueness constraints.
///
/// Primary key definitions are managed separately by [`Schema`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableConstraint {
    /// Requires the specified column values to be unique
    /// across rows in the table.
    ///
    /// A single column identifier defines a single-column
    /// constraint, while multiple identifiers define a
    /// composite uniqueness constraint.
    ///
    /// The handling of `NULL` values depends on ParDB's
    /// uniqueness semantics.
    Unique(Vec<ColumnId>),
}

/// Describes the logical structure of a database table.
///
/// A schema contains an ordered collection of column definitions
/// and the constraints associated with those columns.
///
/// It defines how rows should be structured and validated, but does
/// not store the rows themselves.
///
/// Schema validation and construction methods will be implemented
/// separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    /// Column definitions in their logical order.
    columns: Vec<Column>,

    /// Optional primary key, supporting composite keys.
    primary_key: Option<Vec<ColumnId>>,

    /// Constraints associated with the table.
    constraints: Vec<TableConstraint>,
}
