#[cfg(test)]
mod tests {
    use crate::engine::lang::ast::*;
    use crate::engine::lang::builders::builder_workflow;
    use crate::engine::lang::parser::{WorkflowParser, Rule};
    use pest::Parser;

    // Helper function to parse and build workflows
    fn parse_workflow(input: &str) -> Vec<Workflow> {
        let pairs = WorkflowParser::parse(Rule::program, input)
            .expect("Failed to parse input");
        builder_workflow::build_workflows(pairs)
    }

    #[test]
    fn test_simple_workflow_building() {
        let input = r#"
            workflow test_workflow {
                score {
                    when true then score = 10
                }
            }
        "#;

        let workflows = parse_workflow(input);
        assert_eq!(workflows.len(), 1);
        
        let workflow = &workflows[0];
        assert_eq!(workflow.name, "test_workflow");
        assert_eq!(workflow.phases.len(), 1);
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                assert_eq!(rules.len(), 1);
                let rule = &rules[0];
                
                // Check condition
                match &rule.condition {
                    Expr::Bool(b) => assert_eq!(*b, true),
                    _ => panic!("Expected Bool expression for condition"),
                }
                
                // Check action
                match &rule.action {
                    Action::AssignScore(expr) => {
                        match expr {
                            Expr::Number(n) => assert_eq!(*n, 10),
                            _ => panic!("Expected Number expression for score"),
                        }
                    },
                    _ => panic!("Expected AssignScore action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }

    #[test]
    fn test_log_action_building() {
        let input = r#"
            workflow log_test {
                score {
                    when false then log "test message"
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                match &rule.action {
                    Action::Log(message) => assert_eq!(message, "test message"),
                    _ => panic!("Expected Log action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }



    #[test]
    fn test_match_phase_building() {
        let input = r#"
            workflow match_test {
                match {
                    when score > 5 then assign to result
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Match(rules) => {
                assert_eq!(rules.len(), 1);
                let rule = &rules[0];
                
                // Check condition (should be a binary operation)
                match &rule.condition {
                    Expr::BinaryOp { left, op, right } => {
                        match (left.as_ref(), op, right.as_ref()) {
                            (Expr::Ident(name), BinaryOperator::Gt, Expr::Number(n)) => {
                                assert_eq!(name, "score");
                                assert_eq!(*n, 5);
                            },
                            _ => panic!("Unexpected binary operation structure"),
                        }
                    },
                    _ => panic!("Expected BinaryOp expression for condition"),
                }
                
                // Check match action
                match &rule.action {
                    MatchAction::AssignTo(var) => assert_eq!(var, "result"),
                }
            },
            _ => panic!("Expected Match phase"),
        }
    }



    #[test]
    fn test_complex_expressions() {
        let input = r#"
            workflow expr_test {
                score {
                    when x + y * 2 == 10 then score = 5
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check the complex condition: x + y * 2 == 10
                match &rule.condition {
                    Expr::BinaryOp { left, op: BinaryOperator::Eq, right } => {
                        // Left side should be: x + y * 2
                        match left.as_ref() {
                            Expr::BinaryOp { left: x, op: BinaryOperator::Add, right: mul_expr } => {
                                match (x.as_ref(), mul_expr.as_ref()) {
                                    (Expr::Ident(x_name), Expr::BinaryOp { left: y, op: BinaryOperator::Mul, right: two }) => {
                                        assert_eq!(x_name, "x");
                                        match (y.as_ref(), two.as_ref()) {
                                            (Expr::Ident(y_name), Expr::Number(n)) => {
                                                assert_eq!(y_name, "y");
                                                assert_eq!(*n, 2);
                                            },
                                            _ => panic!("Unexpected multiplication operands"),
                                        }
                                    },
                                    _ => panic!("Unexpected addition operands"),
                                }
                            },
                            _ => panic!("Expected addition expression on left side"),
                        }
                        
                        // Right side should be: 10
                        match right.as_ref() {
                            Expr::Number(n) => assert_eq!(*n, 10),
                            _ => panic!("Expected number on right side"),
                        }
                    },
                    _ => panic!("Expected equality comparison"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }



    #[test]
    fn test_logical_expressions() {
        let input = r#"
            workflow logic_test {
                score {
                    when x > 0 and y < 10 then score = 1
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check the logical AND condition
                match &rule.condition {
                    Expr::BinaryOp { left, op: BinaryOperator::And, right } => {
                        // Left: x > 0
                        match left.as_ref() {
                            Expr::BinaryOp { left: x, op: BinaryOperator::Gt, right: zero } => {
                                match (x.as_ref(), zero.as_ref()) {
                                    (Expr::Ident(name), Expr::Number(n)) => {
                                        assert_eq!(name, "x");
                                        assert_eq!(*n, 0);
                                    },
                                    _ => panic!("Unexpected left comparison"),
                                }
                            },
                            _ => panic!("Expected greater than comparison on left"),
                        }
                        
                        // Right: y < 10
                        match right.as_ref() {
                            Expr::BinaryOp { left: y, op: BinaryOperator::Lt, right: ten } => {
                                match (y.as_ref(), ten.as_ref()) {
                                    (Expr::Ident(name), Expr::Number(n)) => {
                                        assert_eq!(name, "y");
                                        assert_eq!(*n, 10);
                                    },
                                    _ => panic!("Unexpected right comparison"),
                                }
                            },
                            _ => panic!("Expected less than comparison on right"),
                        }
                    },
                    _ => panic!("Expected logical AND expression"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }



    #[test]
    fn test_unary_expressions() {
        let input = r#"
            workflow unary_test {
                score {
                    when !active then score = -1
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check condition: !active
                match &rule.condition {
                    Expr::UnaryOp { op: UnaryOperator::Not, expr } => {
                        match expr.as_ref() {
                            Expr::Ident(name) => assert_eq!(name, "active"),
                            _ => panic!("Expected identifier in unary expression"),
                        }
                    },
                    _ => panic!("Expected unary NOT expression"),
                }
                
                // Check action: score = -1
                match &rule.action {
                    Action::AssignScore(expr) => {
                        match expr {
                            Expr::UnaryOp { op: UnaryOperator::Neg, expr } => {
                                match expr.as_ref() {
                                    Expr::Number(n) => assert_eq!(*n, 1),
                                    _ => panic!("Expected number in negation"),
                                }
                            },
                            _ => panic!("Expected unary negation"),
                        }
                    },
                    _ => panic!("Expected AssignScore action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }

    #[test]
    fn test_function_call_expressions() {
        let input = r#"
            workflow func_test {
                score {
                    when contains(list, item) then score = calculate(x, y, z)
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check condition: contains(list, item)
                match &rule.condition {
                    Expr::FunctionCall { name, args } => {
                        assert_eq!(name, "contains");
                        assert_eq!(args.len(), 2);
                        
                        match (&args[0], &args[1]) {
                            (Expr::Ident(arg1), Expr::Ident(arg2)) => {
                                assert_eq!(arg1, "list");
                                assert_eq!(arg2, "item");
                            },
                            _ => panic!("Expected identifier arguments"),
                        }
                    },
                    _ => panic!("Expected function call expression"),
                }
                
                // Check action: score = calculate(x, y, z)
                match &rule.action {
                    Action::AssignScore(expr) => {
                        match expr {
                            Expr::FunctionCall { name, args } => {
                                assert_eq!(name, "calculate");
                                assert_eq!(args.len(), 3);
                                
                                for (i, expected) in ["x", "y", "z"].iter().enumerate() {
                                    match &args[i] {
                                        Expr::Ident(name) => assert_eq!(name, expected),
                                        _ => panic!("Expected identifier argument"),
                                    }
                                }
                            },
                            _ => panic!("Expected function call expression"),
                        }
                    },
                    _ => panic!("Expected AssignScore action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }

    #[test]
    fn test_list_expressions() {
        let input = r#"
            workflow list_test {
                score {
                    when item in [1, 2, 3] then score = 1
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check condition: item in [1, 2, 3]
                match &rule.condition {
                    Expr::BinaryOp { left, op: BinaryOperator::In, right } => {
                        // Left: item
                        match left.as_ref() {
                            Expr::Ident(name) => assert_eq!(name, "item"),
                            _ => panic!("Expected identifier on left"),
                        }
                        
                        // Right: [1, 2, 3]
                        match right.as_ref() {
                            Expr::List(items) => {
                                assert_eq!(items.len(), 3);
                                for (i, expected) in [1, 2, 3].iter().enumerate() {
                                    match &items[i] {
                                        Expr::Number(n) => assert_eq!(n, expected),
                                        _ => panic!("Expected number in list"),
                                    }
                                }
                            },
                            _ => panic!("Expected list on right"),
                        }
                    },
                    _ => panic!("Expected 'in' expression"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }

    #[test]
    fn test_multiple_phases() {
        let input = r#"
            workflow multi_phase {
                score {
                    when x > 0 then score = x
                    when y < 0 then log "negative"
                }
                match {
                    when score > 5 then assign to high
                    when score > 0 then assign to low
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        assert_eq!(workflow.phases.len(), 2);
        
        // Check first phase (score)
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                assert_eq!(rules.len(), 2);
                
                // First rule: when x > 0 then score = x
                match &rules[0].action {
                    Action::AssignScore(Expr::Ident(name)) => assert_eq!(name, "x"),
                    _ => panic!("Expected AssignScore with identifier"),
                }
                
                // Second rule: when y < 0 then log "negative"
                match &rules[1].action {
                    Action::Log(message) => assert_eq!(message, "negative"),
                    _ => panic!("Expected Log action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
        
        // Check second phase (match)
        match &workflow.phases[1] {
            Phase::Match(rules) => {
                assert_eq!(rules.len(), 2);
                
                // First rule: when score > 5 then assign to high
                match &rules[0].action {
                    MatchAction::AssignTo(var) => assert_eq!(var, "high"),
                }
                
                // Second rule: when score > 0 then assign to low
                match &rules[1].action {
                    MatchAction::AssignTo(var) => assert_eq!(var, "low"),
                }
            },
            _ => panic!("Expected Match phase"),
        }
    }

    #[test]
    fn test_multiple_workflows() {
        let input = r#"
            workflow first {
                score {
                    when true then score = 1
                }
            }
            
            workflow second {
                match {
                    when false then assign to result
                }
            }
        "#;

        let workflows = parse_workflow(input);
        assert_eq!(workflows.len(), 2);
        
        assert_eq!(workflows[0].name, "first");
        assert_eq!(workflows[1].name, "second");
        
        // Check first workflow has score phase
        match &workflows[0].phases[0] {
            Phase::Score(_) => {},
            _ => panic!("Expected Score phase in first workflow"),
        }
        
        // Check second workflow has match phase
        match &workflows[1].phases[0] {
            Phase::Match(_) => {},
            _ => panic!("Expected Match phase in second workflow"),
        }
    }

    #[test]
    fn test_boolean_literals() {
        let input = r#"
            workflow bool_test {
                score {
                    when true then score = 1
                    when false then score = 0
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                // First rule condition: true
                match &rules[0].condition {
                    Expr::Bool(b) => assert_eq!(*b, true),
                    _ => panic!("Expected boolean true"),
                }
                
                // Second rule condition: false
                match &rules[1].condition {
                    Expr::Bool(b) => assert_eq!(*b, false),
                    _ => panic!("Expected boolean false"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }

    #[test]
    fn test_string_literals() {
        let input = r#"
            workflow string_test {
                score {
                    when name == "test" then log "found test"
                }
            }
        "#;

        let workflows = parse_workflow(input);
        let workflow = &workflows[0];
        
        match &workflow.phases[0] {
            Phase::Score(rules) => {
                let rule = &rules[0];
                
                // Check condition: name == "test"
                match &rule.condition {
                    Expr::BinaryOp { left, op: BinaryOperator::Eq, right } => {
                        match (left.as_ref(), right.as_ref()) {
                            (Expr::Ident(name), Expr::String(value)) => {
                                assert_eq!(name, "name");
                                assert_eq!(value, "test");
                            },
                            _ => panic!("Expected identifier and string comparison"),
                        }
                    },
                    _ => panic!("Expected equality comparison"),
                }
                
                // Check action: log "found test"
                match &rule.action {
                    Action::Log(message) => assert_eq!(message, "found test"),
                    _ => panic!("Expected Log action"),
                }
            },
            _ => panic!("Expected Score phase"),
        }
    }
}