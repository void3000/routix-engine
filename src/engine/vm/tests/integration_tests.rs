#[cfg(test)]
mod tests {
    use crate::{
        engine::{
            vm::corevm::CoreVM,
            lang::{
                parser::{WorkflowParser, Rule},
                builders::builder_workflow,
            }
        },
        models::case::CaseConfig
    };
    use pest::Parser;

    #[test]
    fn test_end_to_end_workflow_execution() {
        // Define a workflow in the DSL
        let workflow_source = r#"
            workflow case_prioritization {
                score {
                    when priority > 3 then score = priority * 10
                    when category == "bug" then score = score + 20
                    when status == "open" then score = score + 5
                    when customer != "" then score = score + 15
                }
                match {
                    when score > 50 then assign to high_priority
                    when score > 25 then assign to medium_priority
                    when score > 0 then assign to low_priority
                }
            }
        "#;

        // Parse the workflow
        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        
        assert_eq!(workflows.len(), 1);
        let workflow = &workflows[0];
        assert_eq!(workflow.name, "case_prioritization");

        // Create test cases
        let cases = vec![
            CaseConfig {
                id: 1,
                category: "bug".to_string(),
                status: "open".to_string(),
                priority: 5,
                customer: Some("important_customer".to_string()),
                score: 0,
            },
            CaseConfig {
                id: 2,
                category: "feature".to_string(),
                status: "closed".to_string(),
                priority: 2,
                customer: None,
                score: 0,
            },
            CaseConfig {
                id: 3,
                category: "bug".to_string(),
                status: "open".to_string(),
                priority: 1,
                customer: Some("regular_customer".to_string()),
                score: 0,
            },
        ];

        // Set up the VM and add cases
        let mut vm = CoreVM::new();
        for case in cases {
            vm.add_case(case);
        }

        // Execute the workflow
        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        // Verify the results
        let processed_cases = vm.get_cases();
        assert_eq!(processed_cases.len(), 3);

        // Case 1: priority=5, bug, open, has customer
        // Score: 5*10 + 20 + 5 + 15 = 90 (high priority)
        assert_eq!(processed_cases[0].score, 90);

        // Case 2: priority=2, feature, closed, no customer
        // Score: 0 (no conditions match)
        assert_eq!(processed_cases[1].score, 0);

        // Case 3: priority=1, bug, open, has customer
        // Score: 0 + 20 + 5 + 15 = 40 (medium priority)
        assert_eq!(processed_cases[2].score, 40);
    }

    #[test]
    fn test_filter_phase() {
        let source = r#"
            workflow filter_test {
                score {
                    when priority > 5 then score = priority * 10
                    when priority < 6 then score = priority * 5
                }
                
                filter {
                    when status == "open" and priority > 4
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Add test cases
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 8,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "closed".to_string(),
            priority: 7,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 3,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Should only have 1 case (open status and priority > 4)
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].id, 1);
        assert_eq!(cases[0].status, "open");
        assert_eq!(cases[0].priority, 8);
        assert_eq!(cases[0].score, 80);
    }

    #[test]
    fn test_sort_phase() {
        let source = r#"
            workflow sort_test {
                score {
                    when true then score = priority
                }
                
                sort {
                    by score desc
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Add test cases with different priorities
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "open".to_string(),
            priority: 8,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 3,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 5,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Should be sorted by score descending (8, 5, 3)
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].score, 8);
        assert_eq!(cases[1].score, 5);
        assert_eq!(cases[2].score, 3);
    }

    #[test]
    fn test_sort_phase_ascending() {
        let source = r#"
            workflow sort_asc_test {
                score {
                    when true then score = priority
                }
                
                sort {
                    by priority asc
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Add test cases with different priorities
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 7,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "open".to_string(),
            priority: 2,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 3,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 5,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Should be sorted by priority ascending (2, 5, 7)
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].priority, 2);
        assert_eq!(cases[1].priority, 5);
        assert_eq!(cases[2].priority, 7);
    }

    #[test]
    fn test_combined_filter_and_sort() {
        let source = r#"
            workflow combined_test {
                score {
                    when priority > 5 then score = priority * 10
                    when priority < 6 then score = priority * 5
                }
                
                filter {
                    when status == "open"
                }
                
                sort {
                    by score desc
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Add test cases
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 8,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "closed".to_string(),
            priority: 9,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 3,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 4,
            category: "enhancement".to_string(),
            status: "open".to_string(),
            priority: 6,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Should only have open cases, sorted by score descending
        assert_eq!(cases.len(), 3);
        assert!(cases.iter().all(|c| c.status == "open"));
        
        // Check sorting (scores should be 80, 60, 15)
        assert_eq!(cases[0].score, 80); // priority 8 * 10
        assert_eq!(cases[1].score, 60); // priority 6 * 10
        assert_eq!(cases[2].score, 15); // priority 3 * 5
    }

    #[test]
    fn test_dot_notation_case_properties() {
        let source = r#"
            workflow dot_notation_test {
                score {
                    when case.priority > 5 then score = case.priority * 10
                    when case.status == "open" then score = score + 5
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 8,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "closed".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Case 1: priority > 5 and status == "open" -> score = 8 * 10 + 5 = 85
        assert_eq!(cases[0].score, 85);
        
        // Case 2: priority <= 5 but status != "open" -> score = 0 + 0 = 0
        assert_eq!(cases[1].score, 0);
    }

    #[test]
    fn test_dot_notation_agent_properties() {
        use std::collections::HashMap;
        
        let source = r#"
            workflow agent_test {
                score {
                    when agent.department == "support" then score = 100
                    when agent.level == "3" then score = score + 50
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Set up agent object
        let mut agent_map = HashMap::new();
        agent_map.insert("id".to_string(), crate::engine::lang::ast::Value::String("agent_001".to_string()));
        agent_map.insert("department".to_string(), crate::engine::lang::ast::Value::String("support".to_string()));
        agent_map.insert("level".to_string(), crate::engine::lang::ast::Value::String("3".to_string()));
        
        vm.context.env.insert("agent", crate::engine::lang::ast::Value::Map(agent_map));
        
        vm.add_case(CaseConfig {
            id: 1,
            category: "test".to_string(),
            status: "open".to_string(),
            priority: 1,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Both conditions match now: score = 100 + 50 = 150
        // With proper Value types, the second condition now works correctly
        assert_eq!(cases[0].score, 150);
    }

    #[test]
    fn test_dot_notation_in_filter_and_sort() {
        let source = r#"
            workflow dot_filter_sort_test {
                score {
                    when case.priority > 3 then score = case.priority * 10
                }
                
                filter {
                    when case.status == "open" and case.priority > 4
                }
                
                sort {
                    by case.priority desc
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 8,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "feature".to_string(),
            status: "closed".to_string(),
            priority: 7,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 3,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 5,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 4,
            category: "enhancement".to_string(),
            status: "open".to_string(),
            priority: 2,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Should only have cases with status="open" and priority > 4, sorted by priority desc
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].id, 1); // priority 8
        assert_eq!(cases[1].id, 3); // priority 5
        assert_eq!(cases[0].score, 80); // 8 * 10
        assert_eq!(cases[1].score, 50); // 5 * 10
    }

    #[test]
    fn test_agent_skills_dot_notation() {
        use std::collections::HashMap;
        
        let source = r#"
            workflow agent_skills_test {
                score {
                    when case.category in agent.skills then score = 100
                    when true then score = score + case.priority
                }
            }
        "#;

        let workflows = builder_workflow::build_workflows(
            WorkflowParser::parse(Rule::program, source).unwrap()
        );
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        
        // Set up agent object with skills as list
        let mut agent_map = HashMap::new();
        agent_map.insert("id".to_string(), crate::engine::lang::ast::Value::String("agent_001".to_string()));
        let skills_list = vec![
            crate::engine::lang::ast::Value::String("bug".to_string()),
            crate::engine::lang::ast::Value::String("feature".to_string()),
            crate::engine::lang::ast::Value::String("support".to_string()),
        ];
        agent_map.insert("skills".to_string(), crate::engine::lang::ast::Value::List(skills_list));
        
        vm.context.env.insert("agent", crate::engine::lang::ast::Value::Map(agent_map));
        
        // Add test cases
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),      // in skills
            status: "open".to_string(),
            priority: 5,
            customer: None,
            score: 0,
        });
        
        vm.add_case(CaseConfig {
            id: 2,
            category: "enhancement".to_string(), // not in skills
            status: "open".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let cases = vm.get_cases();
        
        // Case 1: category in skills -> score = 100 + 5 = 105
        assert_eq!(cases[0].score, 105);
        
        // Case 2: category not in skills -> score = 0 + 3 = 3
        assert_eq!(cases[1].score, 3);
    }

    #[test]
    fn test_complex_expression_workflow() {
        let workflow_source = r#"
            workflow complex_scoring {
                score {
                    when (priority * 2) + 1 > 5 then score = max(priority * 10, 50)
                    when len(category) > 3 and status in ["open", "pending"] then score = score + 25
                    when !contains(["low", "normal"], category) then score = score * 2
                }
            }
        "#;

        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        vm.add_case(CaseConfig {
            id: 1,
            category: "critical".to_string(),
            status: "open".to_string(),
            priority: 4,
            customer: Some("test".to_string()),
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let processed_cases = vm.get_cases();
        // Expected: max(4*10, 50) = 50, then +25 for length and status, then *2 for not in list = 150
        assert_eq!(processed_cases[0].score, 150);
    }

    #[test]
    fn test_match_phase_assignment() {
        let workflow_source = r#"
            workflow categorization {
                score {
                    when priority > 3 then score = 100
                    when priority < 4 then score = 10
                }
                match {
                    when score > 99 then assign to urgent_cases
                    when score > 49 then assign to normal_cases
                    when score > 0 then assign to low_cases
                }
            }
        "#;

        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        vm.add_case(CaseConfig {
            id: 1,
            category: "test".to_string(),
            status: "open".to_string(),
            priority: 5,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        // Check that the case was assigned to the urgent_cases variable
        let urgent_cases = vm.context.env.lookup("urgent_cases");
        assert!(urgent_cases.is_some());
        
        // Verify it's a map with the case data
        match urgent_cases.unwrap() {
            crate::engine::lang::ast::Value::Map(map) => {
                assert_eq!(map.get("id").unwrap(), &crate::engine::lang::ast::Value::String("1".to_string()));
                assert_eq!(map.get("priority").unwrap(), &crate::engine::lang::ast::Value::String("5".to_string()));
                assert_eq!(map.get("score").unwrap(), &crate::engine::lang::ast::Value::String("100".to_string()));
            }
            _ => panic!("Expected map value for urgent_cases"),
        }
    }

    #[test]
    fn test_function_calls_in_workflow() {
        let workflow_source = r#"
            workflow function_demo {
                score {
                    when contains(["bug", "critical"], category) then score = 100
                    when len(status) > 4 then score = score + 10
                    when min(priority, 5) == priority then score = score + priority
                }
            }
        "#;

        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "pending".to_string(),
            priority: 3,
            customer: None,
            score: 0,
        });

        vm.execute_workflow(workflow).expect("Failed to execute workflow");

        let processed_cases = vm.get_cases();
        // Expected: 100 (contains) + 10 (len > 4) + 3 (min condition) = 113
        assert_eq!(processed_cases[0].score, 113);
    }

    #[test]
    fn test_multiple_workflows() {
        let workflow_source = r#"
            workflow initial_scoring {
                score {
                    when priority > 2 then score = priority * 5
                }
            }
            
            workflow final_adjustment {
                score {
                    when score > 10 then score = score + 20
                    when category == "bug" then score = score * 2
                }
            }
        "#;

        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        
        assert_eq!(workflows.len(), 2);

        let mut vm = CoreVM::new();
        vm.add_case(CaseConfig {
            id: 1,
            category: "bug".to_string(),
            status: "open".to_string(),
            priority: 4,
            customer: None,
            score: 0,
        });

        // Execute first workflow
        vm.execute_workflow(&workflows[0]).expect("Failed to execute first workflow");
        let after_first = vm.get_cases()[0].score;
        assert_eq!(after_first, 20); // 4 * 5

        // Execute second workflow
        vm.execute_workflow(&workflows[1]).expect("Failed to execute second workflow");
        let after_second = vm.get_cases()[0].score;
        assert_eq!(after_second, 80); // (20 + 20) * 2
    }

    #[test]
    fn test_error_handling_in_workflow() {
        let workflow_source = r#"
            workflow error_demo {
                score {
                    when unknown_variable > 0 then score = 100
                }
            }
        "#;

        let pairs = WorkflowParser::parse(Rule::program, workflow_source)
            .expect("Failed to parse workflow");
        let workflows = builder_workflow::build_workflows(pairs);
        let workflow = &workflows[0];

        let mut vm = CoreVM::new();
        vm.add_case(CaseConfig {
            id: 1,
            category: "test".to_string(),
            status: "open".to_string(),
            priority: 1,
            customer: None,
            score: 0,
        });

        // This should fail due to undefined variable
        let result = vm.execute_workflow(workflow);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable: unknown_variable"));
    }
}