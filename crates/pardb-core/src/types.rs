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
