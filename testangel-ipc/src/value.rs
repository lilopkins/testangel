use std::fmt;

#[cfg(feature = "schemas")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A type of a parameter
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "t", content = "v")]
pub enum ParameterKind {
    /// A string type.
    String,
    /// An integer, stored as a 32-bit signed integer.
    Integer,
    /// A decimal number, stored as a 32-bit float.
    Decimal,
    /// A boolean value.
    Boolean,
}
impl ParameterKind {
    #[must_use]
    pub fn default_value(&self) -> ParameterValue {
        match self {
            Self::String => ParameterValue::String(String::new()),
            Self::Integer => ParameterValue::Integer(0),
            Self::Decimal => ParameterValue::Decimal(0.),
            Self::Boolean => ParameterValue::Boolean(false),
        }
    }
}

impl fmt::Display for ParameterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "Text"),
            Self::Integer => write!(f, "Integer"),
            Self::Decimal => write!(f, "Decimal"),
            Self::Boolean => write!(f, "Boolean"),
        }
    }
}

/// A value of a parameter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "t", content = "v")]
pub enum ParameterValue {
    /// A string type.
    String(String),
    /// An integer, stored as a 64-bit signed integer.
    Integer(i64),
    /// A decimal number, stored as a 64-bit float.
    Decimal(f64),
    /// A boolean value
    Boolean(bool),
}

impl ParameterValue {
    /// Returns the value as an string.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a string.
    #[must_use]
    pub fn value_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            _ => panic!("value isn't an string"),
        }
    }

    /// Returns the value as an i64.
    ///
    /// # Panics
    ///
    /// Panics if the value is not an i64.
    #[must_use]
    pub fn value_i64(&self) -> i64 {
        match self {
            Self::Integer(v) => *v,
            _ => panic!("value isn't an i64"),
        }
    }

    /// Returns the value as an f64.
    ///
    /// # Panics
    ///
    /// Panics if the value is not an f64.
    #[must_use]
    pub fn value_f64(&self) -> f64 {
        match self {
            Self::Decimal(v) => *v,
            _ => panic!("value isn't an f64"),
        }
    }

    /// Returns the value as an bool.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a bool.
    #[must_use]
    pub fn value_bool(&self) -> bool {
        match self {
            Self::Boolean(v) => *v,
            _ => panic!("value isn't a boolean"),
        }
    }

    /// Get the kind of this parameter
    #[must_use]
    pub fn kind(&self) -> ParameterKind {
        match self {
            Self::Decimal(_) => ParameterKind::Decimal,
            Self::Integer(_) => ParameterKind::Integer,
            Self::String(_) => ParameterKind::String,
            Self::Boolean(_) => ParameterKind::Boolean,
        }
    }

    /// Get a mutable pointer to the value.
    ///
    /// # Panics
    ///
    /// Panics if the value is not an i64.
    #[must_use]
    pub fn i64_mut(&mut self) -> &mut i64 {
        match self {
            Self::Integer(a) => a,
            _ => panic!("value isn't an i32"),
        }
    }

    /// Get a mutable pointer to the value.
    ///
    /// # Panics
    ///
    /// Panics if the value is not an f64.
    #[must_use]
    pub fn f64_mut(&mut self) -> &mut f64 {
        match self {
            Self::Decimal(a) => a,
            _ => panic!("value isn't an f32"),
        }
    }

    /// Get a mutable pointer to the value.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a string.
    #[must_use]
    pub fn string_mut(&mut self) -> &mut String {
        match self {
            Self::String(a) => a,
            _ => panic!("value isn't a string"),
        }
    }
}

impl fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(a) => write!(f, "{a}"),
            Self::Decimal(a) => write!(f, "{a}"),
            Self::String(a) => write!(f, "{a}"),
            Self::Boolean(b) => write!(f, "{b}"),
        }
    }
}
