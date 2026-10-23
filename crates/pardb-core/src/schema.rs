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

use crate::error::SchemaError;
use crate::types::DataType;
use std::collections::{HashMap, HashSet};

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

impl Schema {
    pub fn new(
        columns: Vec<Column>,
        constraints: Vec<TableConstraint>,
    ) -> Result<Self, SchemaError> {
        let schema = Self {
            columns,
            constraints,
        };

        schema.validate()?;

        Ok(schema)
    }

    /// Returns the columns defined by this schema.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the constraints defined by this schema.
    pub fn constraints(&self) -> &[TableConstraint] {
        &self.constraints
    }

    /// Finds a column by its identifier.
    pub fn column_by_id(&self, id: ColumnId) -> Option<&Column> {
        self.columns.iter().find(|column| column.id == id)
    }

    /// Finds a column by its name.
    pub fn column_by_name(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// Returns the primary key constraint, if one exists.
    pub fn primary_key(&self) -> Option<&TableConstraint> {
        self.constraints
            .iter()
            .find(|constraint| matches!(constraint.kind, ConstraintKind::PrimaryKey { .. }))
    }

    /// Validates the schema's column definitions and table constraints.
    ///
    /// Performs all structural validation required to ensure that the
    /// schema is internally consistent.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if any column definition or table
    /// constraint violates the schema's validation rules.
    fn validate(&self) -> Result<(), SchemaError> {
        self.validate_columns()?;
        self.validate_constraints()?;

        Ok(())
    }

    /// Validates the column definitions within the schema.
    ///
    /// Ensures that:
    /// - Column names are not empty or whitespace-only.
    /// - Column names are unique within the schema.
    /// - Column identifiers are unique within the schema.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`SchemaError::EmptyColumnName`] if a column has an empty name.
    /// - [`SchemaError::DuplicateColumnName`] if column names are repeated.
    /// - [`SchemaError::DuplicateColumnId`] if column identifiers are repeated.
    fn validate_columns(&self) -> Result<(), SchemaError> {
        let mut names = HashSet::new();
        let mut ids = HashSet::new();

        for column in &self.columns {
            if column.name.trim().is_empty() {
                return Err(SchemaError::EmptyColumnName);
            }

            if !names.insert(&column.name) {
                return Err(SchemaError::DuplicateColumnName(column.name.clone()));
            }

            if !ids.insert(column.id) {
                return Err(SchemaError::DuplicateColumnId(column.id));
            }
        }

        Ok(())
    }

    /// Validates all table-level constraints defined by the schema.
    ///
    /// Ensures that:
    /// - Constraint names are not empty or whitespace-only.
    /// - Constraint names are unique within the schema.
    /// - At most one primary key constraint is defined.
    /// - Every constraint references valid columns.
    /// - Primary key columns do not permit `NULL` values.
    ///
    /// Column references are resolved using a temporary lookup map
    /// to avoid repeatedly searching through the column definitions.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if a constraint name is invalid,
    /// multiple primary keys are defined, or any constraint fails
    /// column-level validation.
    fn validate_constraints(&self) -> Result<(), SchemaError> {
        let columns: HashMap<ColumnId, &Column> = self
            .columns
            .iter()
            .map(|column| (column.id, column))
            .collect();

        let mut names = HashSet::new();
        let mut primary_key_found = false;

        for constraint in &self.constraints {
            if constraint.name.trim().is_empty() {
                return Err(SchemaError::EmptyConstraintName);
            }

            if !names.insert(&constraint.name) {
                return Err(SchemaError::DuplicateConstraintName(
                    constraint.name.clone(),
                ));
            }

            let (ids, is_primary) = match &constraint.kind {
                ConstraintKind::PrimaryKey { columns } => {
                    if primary_key_found {
                        return Err(SchemaError::MultiplePrimaryKeys);
                    }

                    primary_key_found = true;
                    (columns, true)
                }

                ConstraintKind::Unique { columns } => (columns, false),
            };

            Self::validate_constraint_columns(ids, is_primary, &columns)?;
        }

        Ok(())
    }

    /// Validates the column references of a single table constraint.
    ///
    /// This helper performs validation shared by primary key and
    /// unique constraints.
    ///
    /// # Arguments
    ///
    /// * `ids` - Column identifiers referenced by the constraint.
    /// * `is_primary` - Whether the constraint is a primary key.
    /// * `columns` - Lookup map containing the schema's columns.
    ///
    /// # Validation Rules
    ///
    /// - A constraint must reference at least one column.
    /// - Every referenced column must exist in the schema.
    /// - A column cannot appear more than once in the same constraint.
    /// - Primary key columns must be non-nullable.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`SchemaError::EmptyConstraintColumns`] if no columns are specified.
    /// - [`SchemaError::DuplicateConstraintColumn`] if a column is repeated.
    /// - [`SchemaError::UnknownColumn`] if a referenced column does not exist.
    /// - [`SchemaError::NullablePrimaryKeyColumn`] if a primary key column
    ///   permits `NULL` values.
    fn validate_constraint_columns(
        ids: &[ColumnId],
        is_primary: bool,
        columns: &HashMap<ColumnId, &Column>,
    ) -> Result<(), SchemaError> {
        if ids.is_empty() {
            return Err(SchemaError::EmptyConstraintColumns);
        }

        let mut seen = HashSet::new();

        for id in ids {
            if !seen.insert(*id) {
                return Err(SchemaError::DuplicateConstraintColumn(*id));
            }

            let column = columns.get(id).ok_or(SchemaError::UnknownColumn(*id))?;

            if is_primary && column.nullable {
                return Err(SchemaError::NullablePrimaryKeyColumn(*id));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------
    // Test Helpers
    // ---------------------------------------------------------

    /// Creates an integer column for testing.
    fn column(id: u32, name: &str, nullable: bool) -> Column {
        Column {
            id: ColumnId(id),
            name: name.to_string(),
            data_type: DataType::Int,
            nullable,
        }
    }

    /// Creates a primary key constraint.
    fn primary_key(name: &str, ids: &[u32]) -> TableConstraint {
        TableConstraint {
            name: name.to_string(),
            kind: ConstraintKind::PrimaryKey {
                columns: ids.iter().copied().map(ColumnId).collect(),
            },
        }
    }

    /// Creates a unique constraint.
    fn unique(name: &str, ids: &[u32]) -> TableConstraint {
        TableConstraint {
            name: name.to_string(),
            kind: ConstraintKind::Unique {
                columns: ids.iter().copied().map(ColumnId).collect(),
            },
        }
    }

    /// Creates a valid schema with three columns.
    fn sample_schema() -> Schema {
        Schema::new(
            vec![
                column(0, "id", false),
                column(1, "age", true),
                column(2, "email", false),
            ],
            vec![primary_key("pk_students", &[0]), unique("uq_email", &[2])],
        )
        .expect("Sample schema should be valid")
    }

    // ---------------------------------------------------------
    // Valid Schema Tests
    // ---------------------------------------------------------

    #[test]
    fn creates_schema_without_constraints() {
        let schema = Schema::new(vec![column(0, "id", false)], vec![]).unwrap();

        assert_eq!(schema.columns().len(), 1);
        assert!(schema.constraints().is_empty());
        assert!(schema.primary_key().is_none());
    }

    #[test]
    fn creates_schema_with_primary_key() {
        let schema = Schema::new(
            vec![column(0, "id", false)],
            vec![primary_key("pk_id", &[0])],
        )
        .unwrap();

        assert!(schema.primary_key().is_some());
    }

    #[test]
    fn creates_composite_primary_key() {
        let schema = Schema::new(
            vec![
                column(0, "student_id", false),
                column(1, "course_id", false),
            ],
            vec![primary_key("pk_enrollment", &[0, 1])],
        )
        .unwrap();

        assert_eq!(
            schema.primary_key(),
            Some(&primary_key("pk_enrollment", &[0, 1]))
        );
    }

    #[test]
    fn creates_single_column_unique_constraint() {
        let schema = Schema::new(
            vec![column(0, "email", true)],
            vec![unique("uq_email", &[0])],
        )
        .unwrap();

        assert_eq!(schema.constraints().len(), 1);
        assert!(schema.primary_key().is_none());
    }

    #[test]
    fn creates_composite_unique_constraint() {
        let schema = Schema::new(
            vec![
                column(0, "first_name", false),
                column(1, "last_name", false),
            ],
            vec![unique("uq_full_name", &[0, 1])],
        )
        .unwrap();

        assert_eq!(schema.constraints()[0], unique("uq_full_name", &[0, 1]));
    }

    #[test]
    fn allows_multiple_unique_constraints() {
        let schema = Schema::new(
            vec![
                column(0, "email", false),
                column(1, "username", false),
                column(2, "age", true),
            ],
            vec![
                unique("uq_email", &[0]),
                unique("uq_username", &[1]),
                unique("uq_username_age", &[1, 2]),
            ],
        )
        .unwrap();

        assert_eq!(schema.constraints().len(), 3);
    }

    #[test]
    fn allows_nullable_columns_in_unique_constraints() {
        let schema = Schema::new(
            vec![column(0, "email", true)],
            vec![unique("uq_email", &[0])],
        );

        assert!(schema.is_ok());
    }

    #[test]
    fn allows_primary_key_and_multiple_unique_constraints() {
        let schema = sample_schema();

        assert_eq!(schema.constraints().len(), 2);
        assert!(schema.primary_key().is_some());
    }

    #[test]
    fn allows_non_sequential_column_ids() {
        let schema = Schema::new(
            vec![column(10, "id", false), column(50, "email", false)],
            vec![primary_key("pk_id", &[10])],
        );

        assert!(schema.is_ok());
    }

    #[test]
    fn allows_empty_schema() {
        let schema = Schema::new(vec![], vec![]).unwrap();

        assert!(schema.columns().is_empty());
        assert!(schema.constraints().is_empty());
    }

    // ---------------------------------------------------------
    // Column Validation Tests
    // ---------------------------------------------------------

    #[test]
    fn rejects_empty_column_name() {
        let result = Schema::new(vec![column(0, "", false)], vec![]);

        assert_eq!(result.unwrap_err(), SchemaError::EmptyColumnName);
    }

    #[test]
    fn rejects_whitespace_column_name() {
        let result = Schema::new(vec![column(0, "   ", false)], vec![]);

        assert_eq!(result.unwrap_err(), SchemaError::EmptyColumnName);
    }

    #[test]
    fn rejects_duplicate_column_names() {
        let result = Schema::new(
            vec![column(0, "email", false), column(1, "email", true)],
            vec![],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::DuplicateColumnName("email".into())
        );
    }

    #[test]
    fn rejects_duplicate_column_ids() {
        let result = Schema::new(
            vec![column(0, "id", false), column(0, "email", false)],
            vec![],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::DuplicateColumnId(ColumnId(0))
        );
    }

    // ---------------------------------------------------------
    // Constraint Name Validation Tests
    // ---------------------------------------------------------

    #[test]
    fn rejects_empty_constraint_name() {
        let result = Schema::new(vec![column(0, "id", false)], vec![primary_key("", &[0])]);

        assert_eq!(result.unwrap_err(), SchemaError::EmptyConstraintName);
    }

    #[test]
    fn rejects_whitespace_constraint_name() {
        let result = Schema::new(vec![column(0, "id", false)], vec![unique("   ", &[0])]);

        assert_eq!(result.unwrap_err(), SchemaError::EmptyConstraintName);
    }

    #[test]
    fn rejects_duplicate_constraint_names() {
        let result = Schema::new(
            vec![column(0, "id", false), column(1, "email", false)],
            vec![
                primary_key("constraint_1", &[0]),
                unique("constraint_1", &[1]),
            ],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::DuplicateConstraintName("constraint_1".into())
        );
    }

    // ---------------------------------------------------------
    // Primary Key Validation Tests
    // ---------------------------------------------------------

    #[test]
    fn rejects_multiple_primary_keys() {
        let result = Schema::new(
            vec![column(0, "id", false), column(1, "other_id", false)],
            vec![
                primary_key("pk_first", &[0]),
                primary_key("pk_second", &[1]),
            ],
        );

        assert_eq!(result.unwrap_err(), SchemaError::MultiplePrimaryKeys);
    }

    #[test]
    fn rejects_nullable_primary_key_column() {
        let result = Schema::new(
            vec![column(0, "id", true)],
            vec![primary_key("pk_id", &[0])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::NullablePrimaryKeyColumn(ColumnId(0))
        );
    }

    #[test]
    fn rejects_nullable_column_in_composite_primary_key() {
        let result = Schema::new(
            vec![column(0, "student_id", false), column(1, "course_id", true)],
            vec![primary_key("pk_enrollment", &[0, 1])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::NullablePrimaryKeyColumn(ColumnId(1))
        );
    }

    #[test]
    fn rejects_empty_primary_key() {
        let result = Schema::new(
            vec![column(0, "id", false)],
            vec![primary_key("pk_id", &[])],
        );

        assert_eq!(result.unwrap_err(), SchemaError::EmptyConstraintColumns);
    }

    // ---------------------------------------------------------
    // Unique Constraint Validation Tests
    // ---------------------------------------------------------

    #[test]
    fn rejects_empty_unique_constraint() {
        let result = Schema::new(
            vec![column(0, "email", false)],
            vec![unique("uq_email", &[])],
        );

        assert_eq!(result.unwrap_err(), SchemaError::EmptyConstraintColumns);
    }

    #[test]
    fn rejects_duplicate_column_in_unique_constraint() {
        let result = Schema::new(
            vec![column(0, "email", false)],
            vec![unique("uq_email", &[0, 0])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::DuplicateConstraintColumn(ColumnId(0))
        );
    }

    #[test]
    fn rejects_duplicate_column_in_primary_key() {
        let result = Schema::new(
            vec![column(0, "id", false)],
            vec![primary_key("pk_id", &[0, 0])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::DuplicateConstraintColumn(ColumnId(0))
        );
    }

    #[test]
    fn rejects_unknown_column_in_primary_key() {
        let result = Schema::new(
            vec![column(0, "id", false)],
            vec![primary_key("pk_id", &[99])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::UnknownColumn(ColumnId(99))
        );
    }

    #[test]
    fn rejects_unknown_column_in_unique_constraint() {
        let result = Schema::new(
            vec![column(0, "email", false)],
            vec![unique("uq_email", &[99])],
        );

        assert_eq!(
            result.unwrap_err(),
            SchemaError::UnknownColumn(ColumnId(99))
        );
    }

    #[test]
    fn allows_same_column_in_different_constraints() {
        let result = Schema::new(
            vec![column(0, "id", false), column(1, "email", false)],
            vec![primary_key("pk_id", &[0]), unique("uq_id_email", &[0, 1])],
        );

        assert!(result.is_ok());
    }

    // ---------------------------------------------------------
    // Schema Accessor Tests
    // ---------------------------------------------------------

    #[test]
    fn returns_columns_in_original_order() {
        let schema = sample_schema();

        let names: Vec<&str> = schema
            .columns()
            .iter()
            .map(|column| column.name.as_str())
            .collect();

        assert_eq!(names, vec!["id", "age", "email"]);
    }

    #[test]
    fn finds_column_by_id() {
        let schema = sample_schema();

        let result = schema.column_by_id(ColumnId(1));

        assert_eq!(result.unwrap().name, "age");
    }

    #[test]
    fn returns_none_for_unknown_column_id() {
        let schema = sample_schema();

        assert!(schema.column_by_id(ColumnId(99)).is_none());
    }

    #[test]
    fn finds_column_by_name() {
        let schema = sample_schema();

        let result = schema.column_by_name("email");

        assert_eq!(result.unwrap().id, ColumnId(2));
    }

    #[test]
    fn returns_none_for_unknown_column_name() {
        let schema = sample_schema();

        assert!(schema.column_by_name("unknown").is_none());
    }

    #[test]
    fn returns_primary_key_constraint() {
        let schema = sample_schema();

        assert_eq!(
            schema.primary_key(),
            Some(&primary_key("pk_students", &[0]))
        );
    }

    #[test]
    fn returns_none_when_primary_key_is_absent() {
        let schema = Schema::new(
            vec![column(0, "email", false)],
            vec![unique("uq_email", &[0])],
        )
        .unwrap();

        assert!(schema.primary_key().is_none());
    }

    #[test]
    fn preserves_constraint_order() {
        let schema = sample_schema();

        assert_eq!(schema.constraints()[0].name, "pk_students");
        assert_eq!(schema.constraints()[1].name, "uq_email");
    }

    #[test]
    fn cloned_schema_equals_original() {
        let schema = sample_schema();
        let cloned = schema.clone();

        assert_eq!(schema, cloned);
    }
}
