use crate::engine::{
    lang::ast::{ Expr, BinaryOperator, UnaryOperator, Value },
    vm::context::VmContext,
};

pub struct ExprEvaluator;

impl ExprEvaluator {
    pub fn evaluate_expr(
        context: &mut VmContext,
        expr: &Expr
    ) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Ident(name) => {
                context.env
                    .lookup(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::List(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(Self::evaluate_expr(context, expr)?);
                }
                Ok(Value::List(values))
            }
            Expr::BinaryOp { left, op, right } => {
                Self::evaluate_binary_op(context, left, op, right)
            }
            Expr::UnaryOp { op, expr } =>
                Self::evaluate_unary_op(context, op, expr),
            Expr::FunctionCall { name, args } => {
                Self::evaluate_function_call(context, name, args)
            }
            Expr::MemberAccess { object, property } => {
                Self::evaluate_member_access(context, object, property)
            }
        }
    }

    fn evaluate_binary_op(
        context: &mut VmContext,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr
    ) -> Result<Value, String> {
        let left_val = Self::evaluate_expr(context, left)?;
        let right_val = Self::evaluate_expr(context, right)?;

        match op {
            BinaryOperator::Add => Self::add_values(&left_val, &right_val),
            BinaryOperator::Sub => Self::sub_values(&left_val, &right_val),
            BinaryOperator::Mul => Self::mul_values(&left_val, &right_val),
            BinaryOperator::Div => Self::div_values(&left_val, &right_val),
            BinaryOperator::Eq => Ok(Value::Bool(Self::values_equal(&left_val, &right_val))),
            BinaryOperator::Neq => Ok(Value::Bool(!Self::values_equal(&left_val, &right_val))),
            BinaryOperator::Lt => Self::compare_values(&left_val, &right_val, |a, b| a < b),
            BinaryOperator::Le => Self::compare_values(&left_val, &right_val, |a, b| a <= b),
            BinaryOperator::Gt => Self::compare_values(&left_val, &right_val, |a, b| a > b),
            BinaryOperator::Ge => Self::compare_values(&left_val, &right_val, |a, b| a >= b),
            BinaryOperator::And => {
                if Self::is_truthy(&left_val) { Ok(right_val) } else { Ok(left_val) }
            }
            BinaryOperator::Or => {
                if Self::is_truthy(&left_val) { Ok(left_val) } else { Ok(right_val) }
            }
            BinaryOperator::In => Self::in_operation(&left_val, &right_val),
        }
    }

    /// Evaluate a unary operation
    fn evaluate_unary_op(
        context: &mut VmContext,
        op: &UnaryOperator,
        expr: &Expr
    ) -> Result<Value, String> {
        let val = Self::evaluate_expr(context, expr)?;

        match op {
            UnaryOperator::Neg =>
                match val {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err("Cannot negate non-number".to_string()),
                }
            UnaryOperator::Not => Ok(Value::Bool(!Self::is_truthy(&val))),
        }
    }

    fn evaluate_function_call(
        context: &mut VmContext,
        name: &str,
        args: &[Expr]
    ) -> Result<Value, String> {
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Self::evaluate_expr(context, arg)?);
        }

        // Look up function in environment
        if let Some(function_value) = context.env.lookup(name) {
            match function_value {
                Value::BuiltinFunction(func) => {
                    return func(&arg_values);
                }
                Value::UserFunction(user_func) => {
                    // Clone the function definition to avoid borrowing issues
                    let user_func_clone = user_func.clone();
                    return Self::evaluate_user_function(context, &user_func_clone, &arg_values);
                }
                _ => return Err(format!("'{}' is not a function", name)),
            }
        }

        Err(format!("Unknown function: {}", name))
    }

    fn evaluate_user_function(
        context: &mut VmContext,
        function: &crate::engine::lang::ast::FunctionDef,
        args: &[Value]
    ) -> Result<Value, String> {
        if args.len() != function.params.len() {
            return Err(
                format!(
                    "Function '{}' expects {} arguments, got {}",
                    function.name,
                    function.params.len(),
                    args.len()
                )
            );
        }

        context.env.enter_scope();

        for (param, arg) in function.params.iter().zip(args.iter()) {
            context.env.insert(param, arg.clone());
        }

        let result = match &function.body {
            crate::engine::lang::ast::FunctionBody::Expression(expr) => {
                Self::evaluate_expr(context, expr)
            }
            crate::engine::lang::ast::FunctionBody::Block(statements) => {
                Self::evaluate_function_block(context, statements)
            }
        };

        context.env.exit_scope();

        result
    }

    fn evaluate_function_block(
        context: &mut VmContext,
        statements: &[crate::engine::lang::ast::Statement]
    ) -> Result<Value, String> {
        let mut last_value = Value::Null;

        for statement in statements {
            match statement {
                crate::engine::lang::ast::Statement::Let { name, value } => {
                    let val = Self::evaluate_expr(context, value)?;
                    context.env.insert(name, val);
                }
                crate::engine::lang::ast::Statement::Assign { name, value } => {
                    let val = Self::evaluate_expr(context, value)?;
                    context.env.insert(name, val);
                }
                crate::engine::lang::ast::Statement::If { condition, then_body, else_body } => {
                    let cond_val = Self::evaluate_expr(context, condition)?;
                    if Self::is_truthy(&cond_val) {
                        last_value = Self::evaluate_function_block(context, then_body)?;
                    } else if let Some(else_stmts) = else_body {
                        last_value = Self::evaluate_function_block(context, else_stmts)?;
                    }
                }
                crate::engine::lang::ast::Statement::Return(expr) => {
                    return Self::evaluate_expr(context, expr);
                }
                crate::engine::lang::ast::Statement::Expression(expr) => {
                    last_value = Self::evaluate_expr(context, expr)?;
                }
            }
        }

        Ok(last_value)
    }

    fn add_values(left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err("Cannot add these types".to_string()),
        }
    }

    fn sub_values(left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err("Cannot subtract non-numbers".to_string()),
        }
    }

    fn mul_values(left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err("Cannot multiply non-numbers".to_string()),
        }
    }

    fn div_values(left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0 { Err("Division by zero".to_string()) } else { Ok(Value::Number(a / b)) }
            }
            _ => Err("Cannot divide non-numbers".to_string()),
        }
    }

    fn compare_values<F>(left: &Value, right: &Value, op: F) -> Result<Value, String>
        where F: Fn(i64, i64) -> bool
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(op(*a, *b))),
            _ => Err("Cannot compare non-numbers".to_string()),
        }
    }

    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::BuiltinFunction(a), Value::BuiltinFunction(b)) => {
                // Compare function pointers
                std::ptr::eq(a as *const _, b as *const _)
            }
            (Value::UserFunction(a), Value::UserFunction(b)) => {
                // Compare function definitions by name and parameters
                a.name == b.name && a.params == b.params
            }
            _ => false,
        }
    }

    fn in_operation(left: &Value, right: &Value) -> Result<Value, String> {
        match right {
            Value::List(list) => {
                for item in list {
                    if Self::values_equal(left, item) {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            Value::String(s) =>
                match left {
                    Value::String(substr) => Ok(Value::Bool(s.contains(substr))),
                    _ => Err("'in' operation with string requires string on left side".to_string()),
                }
            _ => Err("'in' operation requires list or string on right side".to_string()),
        }
    }

    /// Evaluate member access expressions like agent.id, case.priority, etc.
    fn evaluate_member_access(
        context: &mut VmContext,
        object: &str,
        property: &str
    ) -> Result<Value, String> {
        // Look up the object in the environment
        if let Some(obj_value) = context.env.lookup(object) {
            match obj_value {
                Value::Map(map) => {
                    // If the object is a map, look up the property and return the Value directly
                    if let Some(prop_value) = map.get(property) {
                        Ok(prop_value.clone())
                    } else {
                        Err(format!("Property '{}' not found on object '{}'", property, object))
                    }
                }
                _ => {
                    // For non-map objects, check if it's a special case like agent or case
                    let obj_value_clone = obj_value.clone();
                    Self::evaluate_special_member_access(context, object, property, &obj_value_clone)
                }
            }
        } else {
            // Check if it's a built-in object like 'case' or 'agent'
            Self::evaluate_builtin_member_access(context, object, property)
        }
    }

    /// Handle special member access for complex objects
    fn evaluate_special_member_access(
        _context: &mut VmContext,
        object: &str,
        property: &str,
        _obj_value: &Value
    ) -> Result<Value, String> {
        // This can be extended for custom object types in the future
        Err(format!("Cannot access property '{}' on object '{}' of this type", property, object))
    }

    /// Handle built-in member access for case and agent objects
    fn evaluate_builtin_member_access(
        context: &mut VmContext,
        object: &str,
        property: &str
    ) -> Result<Value, String> {
        match object {
            "case" => {
                // Access case properties directly from environment variables
                match property {
                    "id" => context.env.lookup("id").cloned().ok_or_else(|| "Case id not available".to_string()),
                    "category" => context.env.lookup("category").cloned().ok_or_else(|| "Case category not available".to_string()),
                    "status" => context.env.lookup("status").cloned().ok_or_else(|| "Case status not available".to_string()),
                    "priority" => context.env.lookup("priority").cloned().ok_or_else(|| "Case priority not available".to_string()),
                    "score" => context.env.lookup("score").cloned().ok_or_else(|| "Case score not available".to_string()),
                    "customer" => context.env.lookup("customer").cloned().ok_or_else(|| "Case customer not available".to_string()),
                    _ => Err(format!("Unknown case property: {}", property))
                }
            }
            "agent" => {
                // Look up agent object and access its properties
                if let Some(agent_value) = context.env.lookup("agent") {
                    match agent_value {
                        Value::Map(agent_map) => {
                            if let Some(prop_value) = agent_map.get(property) {
                                // Now we can return the Value directly since Map contains Value types
                                Ok(prop_value.clone())
                            } else {
                                Err(format!("Agent property '{}' not found", property))
                            }
                        }
                        _ => Err("Agent is not a map object".to_string())
                    }
                } else {
                    Err("Agent object not available in context".to_string())
                }
            }
            _ => {
                // Check if the object exists as a map in the environment
                if let Some(obj_value) = context.env.lookup(object) {
                    match obj_value {
                        Value::Map(map) => {
                            if let Some(prop_value) = map.get(property) {
                                // Return the Value directly since Map now contains Value types
                                Ok(prop_value.clone())
                            } else {
                                Err(format!("Property '{}' not found on object '{}'", property, object))
                            }
                        }
                        _ => Err(format!("Object '{}' is not accessible with dot notation", object))
                    }
                } else {
                    Err(format!("Unknown object: {}", object))
                }
            }
        }
    }

    pub fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Null => false,
            Value::Map(m) => !m.is_empty(),
            Value::BuiltinFunction(_) => true,
            Value::UserFunction(_) => true,
        }
    }
}
