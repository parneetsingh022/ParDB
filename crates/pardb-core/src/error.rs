use crate::schema::ColumnId;

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
