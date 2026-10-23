use crate::error::RowValidationError;
use crate::row::Row;
use crate::schema::{Column, Schema};
use crate::types::Value;

impl Schema {
    /// Validates a row against the schema.
    ///
    /// Checks column count, data types, and nullability.
    ///
    /// # Errors
    ///
    /// Returns [`RowValidationError`] if the row does not
    /// conform to the schema.
    pub fn validate_row(&self, row: &Row) -> Result<(), RowValidationError> {
        self.validate_column_count(row)?;

        for (column, value) in self.columns().iter().zip(row.values()) {
            Self::validate_value(column, value)?;
        }

        Ok(())
    }

    /// Validates that the row contains the expected number of values.
    ///
    /// # Errors
    ///
    /// Returns [`RowValidationError::ColumnCountMismatch`] if
    /// the number of values differs from the schema's column count.
    fn validate_column_count(&self, row: &Row) -> Result<(), RowValidationError> {
        let expected = self.columns().len();
        let actual = row.values().len();

        if expected != actual {
            return Err(RowValidationError::ColumnCountMismatch { expected, actual });
        }

        Ok(())
    }

    /// Validates a value against its corresponding column definition.
    ///
    /// Checks whether NULL is permitted and whether non-null
    /// values match the column's declared data type.
    ///
    /// # Errors
    ///
    /// Returns [`RowValidationError::NullViolation`] if a NULL
    /// value is assigned to a non-nullable column.
    ///
    /// Returns [`RowValidationError::TypeMismatch`] if a non-null
    /// value has an incompatible data type.
    fn validate_value(column: &Column, value: &Value) -> Result<(), RowValidationError> {
        match value {
            Value::Null => {
                if !column.nullable {
                    return Err(RowValidationError::NullViolation { column: column.id });
                }
            }

            _ => {
                let actual = value
                    .data_type()
                    .expect("Non-null values must have a data type");

                if actual != column.data_type {
                    return Err(RowValidationError::TypeMismatch {
                        column: column.id,
                        expected: column.data_type,
                        actual,
                    });
                }
            }
        }

        Ok(())
    }
}
