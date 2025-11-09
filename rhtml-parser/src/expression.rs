// File: src/parser/expression.rs
// Purpose: Evaluate simple Rust-like expressions in templates

use std::collections::HashMap;

/// Simple expression evaluator for conditions and interpolations
pub struct ExpressionEvaluator {
    pub variables: HashMap<String, Value>,
}

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

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.variables.insert(name.into(), value);
    }

    /// Evaluate a boolean expression (for r-if conditions)
    pub fn eval_bool(&self, expr: &str) -> bool {
        let expr = expr.trim();

        // Handle simple boolean literals
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Handle variable lookup
        if let Some(value) = self.variables.get(expr) {
            return self.value_to_bool(value);
        }

        // Handle negation: !variable
        if let Some(stripped) = expr.strip_prefix('!') {
            let var_name = stripped.trim();
            if let Some(value) = self.variables.get(var_name) {
                return !self.value_to_bool(value);
            }
            return false;
        }

        // Handle comparisons: variable == value, variable > value, etc.
        if let Some(result) = self.eval_comparison(expr) {
            return result;
        }

        // Default: false for unknown expressions
        false
    }

    /// Evaluate comparison expressions
    fn eval_comparison(&self, expr: &str) -> Option<bool> {
        // >= <= == != > <
        let operators = [">=", "<=", "==", "!=", ">", "<"];

        for op in operators {
            if let Some(pos) = expr.find(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();

                let left_val = self.eval_value(left)?;
                let right_val = self.eval_value(right)?;

                return Some(self.compare_values(&left_val, &right_val, op));
            }
        }

        None
    }

    /// Compare two values with an operator
    fn compare_values(&self, left: &Value, right: &Value, op: &str) -> bool {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => match op {
                "==" => l == r,
                "!=" => l != r,
                ">" => l > r,
                "<" => l < r,
                ">=" => l >= r,
                "<=" => l <= r,
                _ => false,
            },
            (Value::String(l), Value::String(r)) => match op {
                "==" => l == r,
                "!=" => l != r,
                _ => false,
            },
            (Value::Bool(l), Value::Bool(r)) => match op {
                "==" => l == r,
                "!=" => l != r,
                _ => false,
            },
            _ => false,
        }
    }

    /// Evaluate an expression to a value
    fn eval_value(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();

        // String literals
        if expr.starts_with('"') && expr.ends_with('"') {
            return Some(Value::String(expr[1..expr.len() - 1].to_string()));
        }

        // Number literals
        if let Ok(num) = expr.parse::<f64>() {
            return Some(Value::Number(num));
        }

        // Boolean literals
        if expr == "true" {
            return Some(Value::Bool(true));
        }
        if expr == "false" {
            return Some(Value::Bool(false));
        }

        // Variable lookup
        self.variables.get(expr).cloned()
    }

    /// Convert a value to boolean
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
        }
    }

    /// Evaluate an expression and return string representation
    pub fn eval_string(&self, expr: &str) -> String {
        let expr = expr.trim();

        // Remove curly braces if present
        let expr = if expr.starts_with('{') && expr.ends_with('}') {
            &expr[1..expr.len() - 1]
        } else {
            expr
        };

        // Variable lookup
        if let Some(value) = self.variables.get(expr) {
            return self.value_to_string(value);
        }

        // String literal
        if expr.starts_with('"') && expr.ends_with('"') {
            return expr[1..expr.len() - 1].to_string();
        }

        // Return as-is if can't evaluate
        expr.to_string()
    }

    /// Convert value to string
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => {
                // Format nicely (no .0 for whole numbers)
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                // Format array as [item1, item2, item3]
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                // Format object as {key1: value1, key2: value2}
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Null => String::new(),
        }
    }

    /// Get an array value from a variable
    pub fn get_array(&self, name: &str) -> Option<Vec<Value>> {
        match self.variables.get(name)? {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_literals() {
        let eval = ExpressionEvaluator::new();
        assert!(eval.eval_bool("true"));
        assert!(!eval.eval_bool("false"));
    }

    #[test]
    fn test_variable_lookup() {
        let mut eval = ExpressionEvaluator::new();
        eval.set("is_active", Value::Bool(true));
        eval.set("count", Value::Number(5.0));

        assert!(eval.eval_bool("is_active"));
        assert!(!eval.eval_bool("!is_active"));
    }

    #[test]
    fn test_comparisons() {
        let mut eval = ExpressionEvaluator::new();
        eval.set("age", Value::Number(25.0));

        assert!(eval.eval_bool("age >= 18"));
        assert!(!eval.eval_bool("age < 18"));
        assert!(eval.eval_bool("age == 25"));
    }
}
