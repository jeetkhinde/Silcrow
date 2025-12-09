// File: src/value.rs
// Purpose: Template value types

use std::collections::HashMap;

/// Supported value types in templates
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    /// Convert value to string for display
    pub fn to_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => {
                // Format number nicely (remove .0 for integers)
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(_) => "[Object]".to_string(),
            Value::Null => "".to_string(),
        }
    }

    /// Convert value to boolean
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(arr: Vec<Value>) -> Self {
        Value::Array(arr)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(obj: HashMap<String, Value>) -> Self {
        Value::Object(obj)
    }
}
