//! Table definitions and metadata for ParDB.
//!
//! This module provides the [`Table`] structure and [`TableId`]
//! identifier used to represent tables within a database.
//!
//! A table contains its name and schema, but does not store
//! row data. Row persistence and retrieval are handled by
//! ParDB's storage engine.

use crate::schema::Schema;

/// Uniquely identifies a table within a database.
///
/// Table identifiers are independent of table names, allowing
/// tables to be renamed without changing their identity.
///
/// Identifiers must be unique within a database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TableId(pub u64);

/// Represents the logical definition of a ParDB table.
///
/// A table contains a unique identifier, a name, and a schema
/// describing its columns and constraints.
///
/// This structure stores only table metadata. Actual row data
/// is managed separately by the storage engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    id: TableId,
    name: String,
    schema: Schema,
}

impl Table {
    /// Creates a new table definition.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier of the table within its database.
    /// * `name` - Name of the table.
    /// * `schema` - Schema defining the table's structure.
    ///
    /// # Note
    ///
    /// This constructor does not validate the schema or check
    /// whether the table ID or name already exists in a database.
    pub fn new(id: TableId, name: impl Into<String>, schema: Schema) -> Self {
        Self {
            id,
            name: name.into(),
            schema,
        }
    }

    /// Returns the unique identifier of the table.
    pub fn id(&self) -> TableId {
        self.id
    }

    /// Returns the name of the table.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the table's schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Column, ColumnId, ConstraintKind, TableConstraint};
    use crate::types::DataType;

    /// Creates a sample schema for testing.
    ///
    /// This helper constructs a schema with two columns
    /// and a single-column primary key.
    fn sample_schema() -> Schema {
        Schema::new(
            vec![
                Column {
                    id: ColumnId(0),
                    name: "id".to_string(),
                    data_type: DataType::Int,
                    nullable: false,
                },
                Column {
                    id: ColumnId(1),
                    name: "name".to_string(),
                    data_type: DataType::String,
                    nullable: false,
                },
            ],
            vec![TableConstraint {
                name: "pk_students".to_string(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec![ColumnId(0)],
                },
            }],
        )
        .expect("Sample schema should be valid")
    }

    #[test]
    fn test_create_table() {
        let schema = sample_schema();
        let table = Table::new(TableId(1), "students", schema.clone());

        assert_eq!(table.id(), TableId(1));
        assert_eq!(table.name(), "students");
        assert_eq!(table.schema(), &schema);
    }

    #[test]
    fn test_table_id_equality() {
        let id1 = TableId(1);
        let id2 = TableId(1);
        let id3 = TableId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_table_equality() {
        let schema = sample_schema();

        let table1 = Table::new(TableId(1), "students", schema.clone());

        let table2 = Table::new(TableId(1), "students", schema.clone());

        let table3 = Table::new(TableId(2), "courses", schema);

        assert_eq!(table1, table2);
        assert_ne!(table1, table3);
    }

    #[test]
    fn test_table_clone() {
        let original = Table::new(TableId(1), "students", sample_schema());

        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_table_name_from_string() {
        let table = Table::new(TableId(1), String::from("students"), sample_schema());

        assert_eq!(table.name(), "students");
    }

    #[test]
    fn test_table_schema_access() {
        let schema = sample_schema();

        let table = Table::new(TableId(1), "students", schema.clone());

        assert_eq!(table.schema(), &schema);
    }
}
