use std::fmt;

use serde::{Deserialize, Serialize};
#[cfg(feature="schemas")] use schemars::JsonSchema;

/// A type of a parameter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "t", content = "v")]
pub enum ParameterKind {
    /// A string type.
    String,
    /// An integer, stored as a 32-bit signed integer.
    Integer,
    /// A decimal number, stored as a 32-bit float.
    Decimal,
    /// A special type. The value is always held as a string, but the user will see it identified differently.
    SpecialType {
        /// The internal ID of the special type. Must be unique to this type.
        id: String,
        /// A friendly name for this special type.
        friendly_name: String,
    },
}
impl ParameterKind {
    pub fn default_value(&self) -> ParameterValue {
        match self {
            Self::String => ParameterValue::String(String::new()),
            Self::Integer => ParameterValue::Integer(0),
            Self::Decimal => ParameterValue::Decimal(0.),
            Self::SpecialType {
                id,
                friendly_name: _,
            } => ParameterValue::SpecialType {
                id: id.clone(),
                value: String::new(),
            },
        }
    }
}

impl fmt::Display for ParameterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "Text"),
            Self::Integer => write!(f, "Integer"),
            Self::Decimal => write!(f, "Decimal"),
            Self::SpecialType {
                id: _,
                friendly_name,
            } => write!(f, "{friendly_name}"),
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
    /// An integer, stored as a 32-bit signed integer.
    Integer(i32),
    /// A decimal number, stored as a 32-bit float.
    Decimal(f32),
    /// A special type. The value is always held as a string, but the user will see it identified differently.
    SpecialType {
        /// The internal ID of the special type. Must be unique to this type.
        id: String,
        /// The value of the parameter.
        value: String,
    },
}

impl ParameterValue {
    /// Returns the id of this special type, or panics if it isn't a special type.
    pub fn special_type_id(&self) -> String {
        match self {
            Self::SpecialType { id, value: _ } => id.clone(),
            _ => panic!("value isn't a special type"),
        }
    }

    /// Returns the value as an f32, or panics if it isn't.
    pub fn value_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            Self::SpecialType { id: _, value } => value.clone(),
            _ => panic!("value isn't an string"),
        }
    }

    /// Returns the value as an i32, or panics if it isn't.
    pub fn value_i32(&self) -> i32 {
        match self {
            Self::Integer(v) => *v,
            _ => panic!("value isn't an i32"),
        }
    }

    /// Returns the value as an f32, or panics if it isn't.
    pub fn value_f32(&self) -> f32 {
        match self {
            Self::Decimal(v) => *v,
            _ => panic!("value isn't an f32"),
        }
    }

    /// Get the kind of this parameter
    pub fn kind(&self) -> ParameterKind {
        match self {
            Self::Decimal(_) => ParameterKind::Decimal,
            Self::Integer(_) => ParameterKind::Integer,
            Self::String(_) => ParameterKind::String,
            Self::SpecialType { id, value: _ } => ParameterKind::SpecialType {
                id: id.clone(),
                friendly_name: "unknown".to_owned(),
            },
        }
    }

    /// Get a mutable pointer to the value, or panics if it isn't an i32.
    pub fn int_mut(&mut self) -> &mut i32 {
        match self {
            Self::Integer(a) => a,
            _ => panic!("value isn't an i32"),
        }
    }

    /// Get a mutable pointer to the value, or panics if it isn't an f32.
    pub fn f32_mut(&mut self) -> &mut f32 {
        match self {
            Self::Decimal(a) => a,
            _ => panic!("value isn't an f32"),
        }
    }

    /// Get a mutable pointer to the value, or panics if it isn't a String or SpecialValue.
    pub fn string_mut(&mut self) -> &mut String {
        match self {
            Self::String(a) => a,
            Self::SpecialType { id: _, value } => value,
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
            Self::SpecialType { id: _, value } => write!(f, "{value}"),
        }
    }
}
