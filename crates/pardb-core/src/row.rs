use crate::types::Value;

/// Represents a single row of values in a ParDB table.
///
/// Values correspond positionally to columns in the table schema.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    /// Creates a row from a collection of values.
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Returns the values contained in the row.
    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_row() {
        let values = vec![
            Value::Int(1),
            Value::String("Alex".to_string()),
            Value::Float(3.8),
        ];

        let row = Row::new(values.clone());

        assert_eq!(row.values(), values.as_slice());
    }

    #[test]
    fn test_empty_row() {
        let row = Row::new(vec![]);

        assert!(row.values().is_empty());
    }

    #[test]
    fn test_row_with_all_value_types() {
        let values = vec![
            Value::Int(42),
            Value::Float(3.5),
            Value::String("ParDB".to_string()),
            Value::Char('P'),
            Value::Bool(true),
            Value::Null,
        ];

        let row = Row::new(values.clone());

        assert_eq!(row.values(), values.as_slice());
        assert_eq!(row.values().len(), 6);
    }

    #[test]
    fn test_row_preserves_value_order() {
        let row = Row::new(vec![
            Value::Int(1),
            Value::String("Alice".to_string()),
            Value::Bool(false),
        ]);

        assert_eq!(row.values()[0], Value::Int(1));
        assert_eq!(row.values()[1], Value::String("Alice".to_string()));
        assert_eq!(row.values()[2], Value::Bool(false));
    }

    #[test]
    fn test_row_clone() {
        let original = Row::new(vec![Value::Int(10), Value::String("Bob".to_string())]);

        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_row_equality() {
        let row1 = Row::new(vec![Value::Int(1), Value::String("Alex".to_string())]);

        let row2 = Row::new(vec![Value::Int(1), Value::String("Alex".to_string())]);

        let row3 = Row::new(vec![Value::Int(2), Value::String("Sam".to_string())]);

        assert_eq!(row1, row2);
        assert_ne!(row1, row3);
    }
}
