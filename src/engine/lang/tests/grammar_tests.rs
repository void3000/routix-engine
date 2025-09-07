#[cfg(test)]
mod tests {
    use crate::engine::lang::parser::{WorkflowParser, Rule};
    use pest::Parser;

    // Helper function to test if a rule parses successfully
    fn assert_parses(grammar_rule: Rule, input: &str) {
        let result = WorkflowParser::parse(grammar_rule, input);
        assert!(result.is_ok(), "Failed to parse '{}' with rule {:?}: {:?}", input, grammar_rule, result.err());
    }

    // Helper function to test if a rule fails to parse
    fn assert_fails(grammar_rule: Rule, input: &str) {
        let result = WorkflowParser::parse(grammar_rule, input);
        assert!(result.is_err(), "Expected '{}' to fail parsing with rule {:?}", input, grammar_rule);
    }

    #[test]
    fn test_basic_literals() {
        // Test identifiers
        assert_parses(Rule::ident, "hello");
        assert_parses(Rule::ident, "test_var");
        assert_parses(Rule::ident, "var123");
        assert_parses(Rule::ident, "CamelCase");
        assert_parses(Rule::ident, "123invalid"); // Valid according to grammar
        assert_parses(Rule::ident, "with"); // "with-dash" would be parsed as "with" 
        assert_fails(Rule::ident, ""); // empty string

        // Test numbers
        assert_parses(Rule::number, "0");
        assert_parses(Rule::number, "123");
        assert_parses(Rule::number, "999");
        assert_parses(Rule::number, "12"); // Grammar only supports integers
        assert_fails(Rule::number, "-123");

        // Test booleans
        assert_parses(Rule::bool, "true");
        assert_parses(Rule::bool, "false");
        assert_fails(Rule::bool, "True");
        assert_fails(Rule::bool, "FALSE");

        // Test strings
        assert_parses(Rule::string, r#""hello""#);
        assert_parses(Rule::string, r#""hello world""#);
        assert_parses(Rule::string, r#""with \"escaped\" quotes""#);
        assert_parses(Rule::string, r#""""#); // empty string
        assert_fails(Rule::string, r#""unclosed"#);
    }

    #[test]
    fn test_lists() {
        assert_parses(Rule::list, "[]");
        assert_parses(Rule::list, "[1]");
        assert_parses(Rule::list, "[1, 2, 3]");
        assert_parses(Rule::list, r#"["hello", "world"]"#);
        assert_parses(Rule::list, "[true, false]");
        assert_parses(Rule::list, "[var1, var2]");
        assert_parses(Rule::list, "[[1, 2], [3, 4]]"); // nested lists
        assert_fails(Rule::list, "[1, 2,]"); // trailing comma
        assert_fails(Rule::list, "[,1]"); // leading comma
    }

    #[test]
    fn test_function_calls() {
        assert_parses(Rule::function_call, "func()");
        assert_parses(Rule::function_call, "func(1)");
        assert_parses(Rule::function_call, "func(1, 2)");
        assert_parses(Rule::function_call, r#"func("hello", 42, true)"#);
        assert_parses(Rule::function_call, "nested(func(1), 2)");
        assert_fails(Rule::function_call, "func(1,)"); // trailing comma
        assert_fails(Rule::function_call, "func(,1)"); // leading comma
    }

    #[test]
    fn test_expressions() {
        // Primary expressions
        assert_parses(Rule::primary_expr, "42");
        assert_parses(Rule::primary_expr, "true");
        assert_parses(Rule::primary_expr, "variable");
        assert_parses(Rule::primary_expr, r#""string""#);
        assert_parses(Rule::primary_expr, "[1, 2, 3]");
        assert_parses(Rule::primary_expr, "func()");
        assert_parses(Rule::primary_expr, "(1 + 2)");

        // Unary expressions
        assert_parses(Rule::unary_expr, "42");
        assert_parses(Rule::unary_expr, "-42");
        assert_parses(Rule::unary_expr, "!true");
        assert_parses(Rule::unary_expr, "--42");
        assert_parses(Rule::unary_expr, "!!true");

        // Arithmetic expressions
        assert_parses(Rule::mul_expr, "2 * 3");
        assert_parses(Rule::mul_expr, "10 / 2");
        assert_parses(Rule::mul_expr, "2 * 3 / 4");
        
        assert_parses(Rule::add_expr, "1 + 2");
        assert_parses(Rule::add_expr, "10 - 5");
        assert_parses(Rule::add_expr, "1 + 2 - 3");
        assert_parses(Rule::add_expr, "2 * 3 + 4");

        // Comparison expressions
        assert_parses(Rule::comp_expr, "1 == 2");
        assert_parses(Rule::comp_expr, "1 != 2");
        assert_parses(Rule::comp_expr, "1 < 2");
        assert_parses(Rule::comp_expr, "1 > 2");
        assert_parses(Rule::comp_expr, "1 <= 2");
        assert_parses(Rule::comp_expr, "1 >= 2");
        assert_parses(Rule::comp_expr, r#"item in ["list"]"#);

        // Logical expressions
        assert_parses(Rule::and_expr, "true and false");
        assert_parses(Rule::and_expr, "1 == 2 and 3 < 4");
        
        assert_parses(Rule::or_expr, "true or false");
        assert_parses(Rule::or_expr, "1 == 2 or 3 < 4");

        // Complex expressions
        assert_parses(Rule::expr, "1 + 2 * 3 == 7 and true or false");
        assert_parses(Rule::expr, "func(1, 2) > 0 and item in [1, 2, 3]");
        assert_parses(Rule::expr, "!(x > 0) or y <= 10");
    }

    #[test]
    fn test_actions() {
        // Score actions
        assert_parses(Rule::action, "score = 10");
        assert_parses(Rule::action, "score = variable");
        assert_parses(Rule::action, "score = 1 + 2");
        assert_parses(Rule::action, "score = func(x, y)");

        // Log actions
        assert_parses(Rule::action, r#"log "hello""#);
        assert_parses(Rule::action, r#"log "debug message""#);
        assert_parses(Rule::action, r#"log "value: 42""#);

        // Invalid actions
        assert_fails(Rule::action, "invalid action");
        assert_fails(Rule::action, "score 10"); // missing =
        assert_fails(Rule::action, "log hello"); // missing quotes
    }

    #[test]
    fn test_match_actions() {
        assert_parses(Rule::match_action, "assign to result");
        assert_parses(Rule::match_action, "assign to output_var");
        assert_parses(Rule::match_action, "assign to var123");
        
        assert_fails(Rule::match_action, "assign result"); // missing "to"
        assert_fails(Rule::match_action, "assign to"); // missing identifier
    }

    #[test]
    fn test_rules() {
        // Basic rules
        assert_parses(Rule::rule, "when true then score = 10");
        assert_parses(Rule::rule, r#"when false then log "message""#);
        assert_parses(Rule::rule, "when x > 0 then score = x * 2");
        assert_parses(Rule::rule, r#"when item in ["a", "b"] then log "found""#);

        // Complex conditions
        assert_parses(Rule::rule, "when x > 0 and y < 10 then score = 5");
        assert_parses(Rule::rule, "when func(x) == true or z != 0 then score = 1");

        // Match rules
        assert_parses(Rule::match_rule, "when true then assign to result");
        assert_parses(Rule::match_rule, "when score > 5 then assign to high_score");
        assert_parses(Rule::match_rule, "when category == \"urgent\" then assign to urgent_items");
    }

    #[test]
    fn test_phases() {
        // Score phases
        assert_parses(Rule::score_phase, "score {}");
        assert_parses(Rule::score_phase, r#"score {
            when true then score = 10
        }"#);
        assert_parses(Rule::score_phase, r#"score {
            when x > 0 then score = x
            when y < 0 then log "negative"
        }"#);

        // Match phases
        assert_parses(Rule::match_phase, "match {}");
        assert_parses(Rule::match_phase, r#"match {
            when score > 5 then assign to high
        }"#);
        assert_parses(Rule::match_phase, r#"match {
            when category == "urgent" then assign to urgent
            when priority > 3 then assign to important
        }"#);
    }

    #[test]
    fn test_workflows() {
        // Empty workflow
        assert_parses(Rule::workflow, "workflow empty {}");

        // Workflow with score phase
        assert_parses(Rule::workflow, r#"workflow scoring {
            score {
                when true then score = 10
            }
        }"#);

        // Workflow with match phase
        assert_parses(Rule::workflow, r#"workflow matching {
            match {
                when score > 5 then assign to result
            }
        }"#);

        // Workflow with both phases
        assert_parses(Rule::workflow, r#"workflow complete {
            score {
                when x > 0 then score = x * 2
                when y < 0 then log "negative value"
            }
            match {
                when score > 10 then assign to high
                when score > 5 then assign to medium
                when score > 0 then assign to low
            }
        }"#);

        // Multiple phases of same type
        assert_parses(Rule::workflow, r#"workflow multi_phase {
            score {
                when condition1 then score = 1
            }
            score {
                when condition2 then score = 2
            }
            match {
                when score > 0 then assign to positive
            }
        }"#);
    }

    #[test]
    fn test_function_definitions() {
        // Function without parameters
        assert_parses(Rule::function_def, "function simple() = 42");
        assert_parses(Rule::function_def, r#"function greeting() = "hello""#);

        // Function with single parameter
        assert_parses(Rule::function_def, "function double(x) = x * 2");
        assert_parses(Rule::function_def, "function negate(flag) = !flag");

        // Function with multiple parameters
        assert_parses(Rule::function_def, "function add(x, y) = x + y");
        assert_parses(Rule::function_def, "function max(a, b, c) = a > b and a > c");

        // Complex function body
        assert_parses(Rule::function_def, "function complex(x, y) = func(x + 1, y * 2) > 0 and x in [1, 2, 3]");
    }

    #[test]
    fn test_complete_programs() {
        // Program with just a workflow
        assert_parses(Rule::program, r#"
            workflow simple {
                score {
                    when true then score = 10
                }
            }
        "#);

        // Program with function and workflow
        assert_parses(Rule::program, r#"
            function double(x) = x * 2
            
            workflow test {
                score {
                    when value > 0 then score = double(value)
                }
                match {
                    when score > 5 then assign to result
                }
            }
        "#);

        // Program with multiple functions and workflows
        assert_parses(Rule::program, r#"
            function add(x, y) = x + y
            function multiply(x, y) = x * y
            
            workflow math_workflow {
                score {
                    when a > 0 then score = add(a, b)
                    when c > 0 then score = multiply(c, d)
                }
            }
            
            workflow simple_workflow {
                match {
                    when result == "success" then assign to completed
                }
            }
        "#);
    }

    #[test]
    fn test_whitespace_and_comments() {
        // Test that comments are ignored
        assert_parses(Rule::program, r#"
            # This is a comment
            workflow test { # Another comment
                score {
                    # Comment in score block
                    when true then score = 10 # End of line comment
                }
            }
        "#);

        // Test various whitespace combinations
        assert_parses(Rule::workflow, "workflow   test   {   }");
        assert_parses(Rule::rule, "when   true   then   score   =   10");
        assert_parses(Rule::expr, "1   +   2   *   3");
    }

    #[test]
    fn test_edge_cases() {
        // Nested parentheses
        assert_parses(Rule::expr, "((1 + 2) * (3 + 4))");
        
        // Complex nested function calls
        assert_parses(Rule::expr, "func1(func2(x, y), func3(z))");
        
        // Mixed operators with precedence
        assert_parses(Rule::expr, "1 + 2 * 3 == 7 and !false or true");
        
        // Long identifier names
        assert_parses(Rule::ident, "very_long_identifier_name_with_numbers_123");
        
        // Complex string with escapes
        assert_parses(Rule::string, r#""String with \"quotes\" and content""#);
        
        // Empty program
        assert_parses(Rule::program, "");
        
        // Program with only comments
        assert_parses(Rule::program, "# Just a comment");
    }

    #[test]
    fn test_invalid_syntax() {
        // Invalid workflow syntax
        assert_fails(Rule::workflow, "workflow {}"); // missing name
        assert_fails(Rule::workflow, "workflow test"); // missing braces
        
        // Invalid rule syntax
        assert_fails(Rule::rule, "when then score = 10"); // missing condition
        assert_fails(Rule::rule, "when true score = 10"); // missing "then"
        
        // Invalid expression syntax
        assert_fails(Rule::expr, ""); // empty expression
        assert_fails(Rule::expr, "* 1"); // invalid operator position
        
        // Invalid function call syntax
        assert_fails(Rule::function_call, "func("); // unclosed parenthesis
        assert_fails(Rule::function_call, "func 1, 2)"); // missing opening parenthesis
        
        // Invalid list syntax
        assert_fails(Rule::list, "[1, 2"); // unclosed bracket
        assert_fails(Rule::list, "1, 2]"); // missing opening bracket
    }
}