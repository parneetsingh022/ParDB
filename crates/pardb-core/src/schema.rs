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

/// Defines a constraint applied to a database table.
///
/// A table constraint may optionally have a name. Constraint names are
/// scoped to the table that contains them and can be used to identify
/// constraints in error messages or schema-management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableConstraint {
    /// Name assigned to the constraint.
    pub name: String,

    /// The kind of rule enforced by the constraint.
    pub kind: ConstraintKind,
}

/// Describes the type and columns involved in a table constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintKind {
    /// Identifies each row uniquely using one or more columns.
    ///
    /// A table may have only one primary key. The referenced columns
    /// must exist in the table and should not allow `NULL` values.
    PrimaryKey {
        /// Columns that make up the primary key.
        ///
        /// Multiple columns represent a composite primary key.
        columns: Vec<ColumnId>,
    },

    /// Prevents duplicate values across one or more columns.
    ///
    /// Multiple unique constraints may exist on the same table.
    Unique {
        /// Columns whose combined values must be unique.
        ///
        /// Multiple columns represent a composite unique constraint.
        columns: Vec<ColumnId>,
    },
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

    /// Constraints associated with the table.
    constraints: Vec<TableConstraint>,
}
