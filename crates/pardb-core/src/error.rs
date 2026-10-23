use crate::schema::ColumnId;
use crate::types::DataType;

/// Errors that can occur when constructing a table schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    EmptyColumnName,
    DuplicateColumnName(String),
    DuplicateColumnId(ColumnId),
    EmptyConstraintName,
    DuplicateConstraintName(String),
    EmptyConstraintColumns,
    UnknownColumn(ColumnId),
    DuplicateConstraintColumn(ColumnId),
    MultiplePrimaryKeys,
    NullablePrimaryKeyColumn(ColumnId),
}

/// Errors encountered when validating a row against a table schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowValidationError {
    /// The number of values does not match the number of columns.
    ColumnCountMismatch {
        /// Number of columns expected by the schema.
        expected: usize,

        /// Number of values provided in the row.
        actual: usize,
    },

    /// A value has an incompatible data type.
    TypeMismatch {
        /// Identifier of the column containing the invalid value.
        column: ColumnId,

        /// Data type required by the column.
        expected: DataType,

        /// Actual data type of the provided value.
        actual: DataType,
    },

    /// A NULL value was provided for a non-nullable column.
    NullViolation {
        /// Identifier of the column that rejected NULL.
        column: ColumnId,
    },
}
