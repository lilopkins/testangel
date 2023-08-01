use serde::{Deserialize, Serialize};

/// A type of a parameter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

/// A value of a parameter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
            Self::SpecialType { id, value: _ } => ParameterKind::SpecialType { id: id.clone(), friendly_name: "unknown".to_owned() },
        }
    }
}
