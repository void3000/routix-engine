#[cfg(test)]
mod tests {
    use crate::{
        engine::{
            vm::corevm::CoreVM,
            lang::ast::{
                Workflow, Phase, Rule, MatchRule, Action, MatchAction,
                Expr, BinaryOperator, UnaryOperator, Value
            }
        },
        models::case::CaseConfig
    };

    fn create_test_case() -> CaseConfig {
        CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 3,
            customer: Some("test_customer".to_string()),
            score: 0,
        }
    }

    #[test]
    fn test_basic_expression_evaluation() {
        let mut vm = CoreVM::new();
        
        // Test number literal
        let expr = Expr::Number(42);
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(42));
        
        // Test string literal
        let expr = Expr::String("hello".to_string());
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
        
        // Test boolean literal
        let expr = Expr::Bool(true);
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = CoreVM::new();
        
        // Test addition
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(10)),
            op: BinaryOperator::Add,
            right: Box::new(Expr::Number(5)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(15));
        
        // Test subtraction
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(10)),
            op: BinaryOperator::Sub,
            right: Box::new(Expr::Number(3)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(7));
        
        // Test multiplication
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(4)),
            op: BinaryOperator::Mul,
            right: Box::new(Expr::Number(6)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(24));
        
        // Test division
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(20)),
            op: BinaryOperator::Div,
            right: Box::new(Expr::Number(4)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(5));
    }

    #[test]
    fn test_comparison_operations() {
        let mut vm = CoreVM::new();
        
        // Test equality
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(5)),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Number(5)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // Test inequality
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(5)),
            op: BinaryOperator::Neq,
            right: Box::new(Expr::Number(3)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // Test greater than
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(10)),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Number(5)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // Test less than or equal
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(5)),
            op: BinaryOperator::Le,
            right: Box::new(Expr::Number(5)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_operations() {
        let mut vm = CoreVM::new();
        
        // Test AND operation
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Bool(true)),
            op: BinaryOperator::And,
            right: Box::new(Expr::Bool(false)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
        
        // Test OR operation
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Bool(true)),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Bool(false)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_unary_operations() {
        let mut vm = CoreVM::new();
        
        // Test negation
        let expr = Expr::UnaryOp {
            op: UnaryOperator::Neg,
            expr: Box::new(Expr::Number(5)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(-5));
        
        // Test logical NOT
        let expr = Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(Expr::Bool(true)),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_list_operations() {
        let mut vm = CoreVM::new();
        
        // Test list creation
        let expr = Expr::List(vec![
            Expr::Number(1),
            Expr::Number(2),
            Expr::Number(3),
        ]);
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Number(1),
            Value::Number(2),
            Value::Number(3),
        ]));
        
        // Test 'in' operation with list
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(2)),
            op: BinaryOperator::In,
            right: Box::new(Expr::List(vec![
                Expr::Number(1),
                Expr::Number(2),
                Expr::Number(3),
            ])),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_builtin_functions() {
        let mut vm = CoreVM::new();
        
        // Test len function
        let expr = Expr::FunctionCall {
            name: "len".to_string(),
            args: vec![Expr::List(vec![
                Expr::Number(1),
                Expr::Number(2),
                Expr::Number(3),
            ])],
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(3));
        
        // Test max function
        let expr = Expr::FunctionCall {
            name: "max".to_string(),
            args: vec![
                Expr::Number(5),
                Expr::Number(10),
                Expr::Number(3),
            ],
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(10));
        
        // Test min function
        let expr = Expr::FunctionCall {
            name: "min".to_string(),
            args: vec![
                Expr::Number(5),
                Expr::Number(10),
                Expr::Number(3),
            ],
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(3));
        
        // Test contains function
        let expr = Expr::FunctionCall {
            name: "contains".to_string(),
            args: vec![
                Expr::List(vec![
                    Expr::String("apple".to_string()),
                    Expr::String("banana".to_string()),
                ]),
                Expr::String("apple".to_string()),
            ],
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_variable_lookup() {
        let mut vm = CoreVM::new();
        let case = create_test_case();
        
        // Add case to stack and set up context
        vm.add_case(case);
        vm.setup_case_context(&vm.context.stack.cases[0].clone()).unwrap();
        
        // Test variable lookup
        let expr = Expr::Ident("priority".to_string());
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Number(3));
        
        let expr = Expr::Ident("category".to_string());
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::String("bug".to_string()));
    }

    #[test]
    fn test_score_phase_execution() {
        let mut vm = CoreVM::new();
        let mut case = create_test_case();
        
        // Create a simple score rule: when priority > 2 then score = 10
        let rule = Rule {
            condition: Expr::BinaryOp {
                left: Box::new(Expr::Ident("priority".to_string())),
                op: BinaryOperator::Gt,
                right: Box::new(Expr::Number(2)),
            },
            action: Action::AssignScore(Expr::Number(10)),
        };
        
        vm.add_case(case.clone());
        vm.setup_case_context(&case).unwrap();
        vm.execute_score_phase(&[rule], &mut case).unwrap();
        
        assert_eq!(case.score, 10);
    }

    #[test]
    fn test_match_phase_execution() {
        let mut vm = CoreVM::new();
        let mut case = create_test_case();
        
        // Create a match rule: when category == "bug" then assign to bug_cases
        let rule = MatchRule {
            condition: Expr::BinaryOp {
                left: Box::new(Expr::Ident("category".to_string())),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::String("bug".to_string())),
            },
            action: MatchAction::AssignTo("bug_cases".to_string()),
        };
        
        vm.add_case(case.clone());
        vm.setup_case_context(&case).unwrap();
        vm.execute_match_phase(&[rule], &mut case).unwrap();
        
        // Check that the variable was assigned
        let result = vm.context.env.lookup("bug_cases");
        assert!(result.is_some());
        match result.unwrap() {
            Value::Map(map) => {
                assert_eq!(map.get("category").unwrap(), &crate::engine::lang::ast::Value::String("bug".to_string()));
                assert_eq!(map.get("id").unwrap(), &crate::engine::lang::ast::Value::String("1".to_string()));
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_complete_workflow_execution() {
        let mut vm = CoreVM::new();
        let case = create_test_case();
        
        // Create a workflow with both score and match phases
        let workflow = Workflow {
            name: "test_workflow".to_string(),
            phases: vec![
                Phase::Score(vec![
                    Rule {
                        condition: Expr::BinaryOp {
                            left: Box::new(Expr::Ident("priority".to_string())),
                            op: BinaryOperator::Gt,
                            right: Box::new(Expr::Number(2)),
                        },
                        action: Action::AssignScore(Expr::Number(15)),
                    },
                    Rule {
                        condition: Expr::BinaryOp {
                            left: Box::new(Expr::Ident("category".to_string())),
                            op: BinaryOperator::Eq,
                            right: Box::new(Expr::String("bug".to_string())),
                        },
                        action: Action::AssignScore(Expr::BinaryOp {
                            left: Box::new(Expr::Ident("score".to_string())),
                            op: BinaryOperator::Add,
                            right: Box::new(Expr::Number(5)),
                        }),
                    },
                ]),
                Phase::Match(vec![
                    MatchRule {
                        condition: Expr::BinaryOp {
                            left: Box::new(Expr::Ident("score".to_string())),
                            op: BinaryOperator::Gt,
                            right: Box::new(Expr::Number(10)),
                        },
                        action: MatchAction::AssignTo("high_priority".to_string()),
                    },
                ]),
            ],
        };
        
        vm.add_case(case);
        vm.execute_workflow(&workflow).unwrap();
        
        // Check that the case was processed correctly
        let processed_cases = vm.get_cases();
        assert_eq!(processed_cases.len(), 1);
        assert_eq!(processed_cases[0].score, 20); // 15 + 5
    }

    #[test]
    fn test_complex_expressions() {
        let mut vm = CoreVM::new();
        let case = create_test_case();
        
        vm.add_case(case);
        vm.setup_case_context(&vm.context.stack.cases[0].clone()).unwrap();
        
        // Test complex expression: (priority * 2) + 1 > 5
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Ident("priority".to_string())),
                    op: BinaryOperator::Mul,
                    right: Box::new(Expr::Number(2)),
                }),
                op: BinaryOperator::Add,
                right: Box::new(Expr::Number(1)),
            }),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Number(5)),
        };
        
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true)); // (3 * 2) + 1 = 7 > 5
    }

    #[test]
    fn test_log_action() {
        let mut vm = CoreVM::new();
        let mut case = create_test_case();
        
        let action = Action::Log("Test log message".to_string());
        
        // This should not panic and should execute successfully
        vm.execute_action(&action, &mut case).unwrap();
    }

    #[test]
    fn test_error_handling() {
        let mut vm = CoreVM::new();
        
        // Test division by zero
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(10)),
            op: BinaryOperator::Div,
            right: Box::new(Expr::Number(0)),
        };
        let result = vm.evaluate_expr(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Division by zero"));
        
        // Test undefined variable
        let expr = Expr::Ident("undefined_var".to_string());
        let result = vm.evaluate_expr(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
        
        // Test unknown function
        let expr = Expr::FunctionCall {
            name: "unknown_func".to_string(),
            args: vec![],
        };
        let result = vm.evaluate_expr(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown function"));
    }

    #[test]
    fn test_multiple_cases() {
        let mut vm = CoreVM::new();
        
        // Add multiple cases
        let case1 = CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 1,
            customer: Some("customer1".to_string()),
            score: 0,
        };
        
        let case2 = CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "closed".to_string(),
            priority: 5,
            customer: Some("customer2".to_string()),
            score: 0,
        };
        
        vm.add_case(case1);
        vm.add_case(case2);
        
        // Create a workflow that scores based on priority
        let workflow = Workflow {
            name: "priority_scoring".to_string(),
            phases: vec![
                Phase::Score(vec![
                    Rule {
                        condition: Expr::Bool(true), // Always true
                        action: Action::AssignScore(Expr::BinaryOp {
                            left: Box::new(Expr::Ident("priority".to_string())),
                            op: BinaryOperator::Mul,
                            right: Box::new(Expr::Number(10)),
                        }),
                    },
                ]),
            ],
        };
        
        vm.execute_workflow(&workflow).unwrap();
        
        let processed_cases = vm.get_cases();
        assert_eq!(processed_cases.len(), 2);
        assert_eq!(processed_cases[0].score, 10); // priority 1 * 10
        assert_eq!(processed_cases[1].score, 50); // priority 5 * 10
    }

    #[test]
    fn test_string_operations() {
        let mut vm = CoreVM::new();
        
        // Test string concatenation
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::String("Hello ".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expr::String("World".to_string())),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
        
        // Test string 'in' operation
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::String("ell".to_string())),
            op: BinaryOperator::In,
            right: Box::new(Expr::String("Hello".to_string())),
        };
        let result = vm.evaluate_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}