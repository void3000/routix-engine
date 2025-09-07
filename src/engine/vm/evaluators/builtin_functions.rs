use crate::engine::lang::ast::Value;
use std::collections::HashMap;

pub struct BuiltinFunctions;

impl BuiltinFunctions {
    /// Register all built-in functions
    pub fn register_all() -> HashMap<String, fn(&[Value]) -> Result<Value, String>> {
        let mut functions = HashMap::new();

        functions.insert("len".to_string(), Self::len_function as fn(&[Value]) -> Result<Value, String>);
        functions.insert("max".to_string(), Self::max_function as fn(&[Value]) -> Result<Value, String>);
        functions.insert("min".to_string(), Self::min_function as fn(&[Value]) -> Result<Value, String>);
        functions.insert("contains".to_string(), Self::contains_function as fn(&[Value]) -> Result<Value, String>);

        functions
    }

    /// len() function - get length of lists or strings
    fn len_function(args: &[Value]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("len() takes exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::List(list) => Ok(Value::Number(list.len() as i64)),
            Value::String(s) => Ok(Value::Number(s.len() as i64)),
            _ => Err("len() can only be applied to lists or strings".to_string()),
        }
    }

    /// max() function - find maximum value among numbers
    fn max_function(args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("max() requires at least 1 argument".to_string());
        }
        let mut max_val = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err("max() can only be applied to numbers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                Value::Number(n) => {
                    if *n > max_val {
                        max_val = *n;
                    }
                }
                _ => return Err("max() can only be applied to numbers".to_string()),
            }
        }
        Ok(Value::Number(max_val))
    }

    /// min() function - find minimum value among numbers
    fn min_function(args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("min() requires at least 1 argument".to_string());
        }
        let mut min_val = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err("min() can only be applied to numbers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                Value::Number(n) => {
                    if *n < min_val {
                        min_val = *n;
                    }
                }
                _ => return Err("min() can only be applied to numbers".to_string()),
            }
        }
        Ok(Value::Number(min_val))
    }

    /// contains() function - check if list/string contains a value
    fn contains_function(args: &[Value]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("contains() takes exactly 2 arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Value::List(list), value) => {
                for item in list {
                    if Self::values_equal(item, value) {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            (Value::String(s), Value::String(substr)) => Ok(Value::Bool(s.contains(substr))),
            _ => Err("contains() first argument must be a list or string".to_string()),
        }
    }

    /// Helper function to compare values for equality
    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}