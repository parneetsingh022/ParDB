//! Core data structures and types for ParDB.

/// Represents the supported data types in ParDB.
///
/// Each variant defines a type that can be assigned to a
/// column in a table schema.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DataType {
    /// A variable-length string.
    String,

    /// A signed integer.
    Int,

    /// A floating-point number.
    Float,

    /// A single character.
    Char,

    /// A boolean value (`true` or `false`).
    Bool,
}

/// Represents a value stored in a ParDB database.
///
/// Each variant contains a value corresponding to one of
/// the supported database data types.
///
/// `Null` represents the absence of a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A variable-length UTF-8 string.
    String(String),

    /// A 64-bit signed integer.
    Int(i64),

    /// A 64-bit floating-point number.
    Float(f64),

    /// A Unicode character.
    Char(char),

    /// A boolean value (`true` or `false`).
    Bool(bool),

    /// Represents a missing or unknown value.
    Null,
}

impl Value {
    /// Returns the data type associated with this value.
    ///
    /// Each non-null value corresponds to a specific [`DataType`].
    ///
    /// Returns [`None`] for [`Value::Null`], since NULL does not
    /// have a concrete data type.
    ///
    /// # Examples
    ///
    /// ```
    /// use pardb_core::{DataType, Value};
    ///
    /// let value = Value::Int(42);
    /// assert_eq!(value.data_type(), Some(DataType::Int));
    ///
    /// let null = Value::Null;
    /// assert_eq!(null.data_type(), None);
    /// ```
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Self::String(_) => Some(DataType::String),
            Self::Int(_) => Some(DataType::Int),
            Self::Float(_) => Some(DataType::Float),
            Self::Char(_) => Some(DataType::Char),
            Self::Bool(_) => Some(DataType::Bool),
            Self::Null => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_value_returns_string_type() {
        let value = Value::String("ParDB".to_string());

        assert_eq!(value.data_type(), Some(DataType::String));
    }

    #[test]
    fn int_value_returns_int_type() {
        let value = Value::Int(42);

        assert_eq!(value.data_type(), Some(DataType::Int));
    }

    #[test]
    fn float_value_returns_float_type() {
        let value = Value::Float(6.99);

        assert_eq!(value.data_type(), Some(DataType::Float));
    }

    #[test]
    fn char_value_returns_char_type() {
        let value = Value::Char('P');

        assert_eq!(value.data_type(), Some(DataType::Char));
    }

    #[test]
    fn bool_value_returns_bool_type() {
        let value = Value::Bool(true);

        assert_eq!(value.data_type(), Some(DataType::Bool));
    }

    #[test]
    fn null_value_returns_none() {
        let value = Value::Null;

        assert_eq!(value.data_type(), None);
    }

    #[test]
    fn data_type_does_not_depend_on_value() {
        let values = [
            Value::Int(0),
            Value::Int(-100),
            Value::Int(i64::MAX),
            Value::Int(i64::MIN),
        ];

        for value in values {
            assert_eq!(value.data_type(), Some(DataType::Int));
        }
    }

    #[test]
    fn empty_string_is_still_string_type() {
        let value = Value::String(String::new());

        assert_eq!(value.data_type(), Some(DataType::String));
    }

    #[test]
    fn unicode_character_is_char_type() {
        let value = Value::Char('界');

        assert_eq!(value.data_type(), Some(DataType::Char));
    }

    #[test]
    fn special_float_values_are_float_type() {
        let values = [
            Value::Float(0.0),
            Value::Float(-0.0),
            Value::Float(f64::NAN),
            Value::Float(f64::INFINITY),
            Value::Float(f64::NEG_INFINITY),
        ];

        for value in values {
            assert_eq!(value.data_type(), Some(DataType::Float));
        }
    }

    #[test]
    fn all_variants_return_expected_types() {
        let cases = [
            (Value::String("hello".into()), Some(DataType::String)),
            (Value::Int(10), Some(DataType::Int)),
            (Value::Float(2.5), Some(DataType::Float)),
            (Value::Char('A'), Some(DataType::Char)),
            (Value::Bool(false), Some(DataType::Bool)),
            (Value::Null, None),
        ];

        for (value, expected) in cases {
            assert_eq!(value.data_type(), expected);
        }
    }
}
