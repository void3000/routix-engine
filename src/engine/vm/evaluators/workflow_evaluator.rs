use crate::{
    engine::{
        lang::ast::{ Workflow, Phase, Rule, MatchRule, FilterRule, SortRule, SortOrder, Value },
        vm::{
            context::VmContext,
            evaluators::{ expr_evaluator::ExprEvaluator, action_evaluator::ActionEvaluator },
        },
    },
    models::case::CaseConfig,
};

pub struct WorkflowEvaluator;

impl WorkflowEvaluator {
    pub fn execute_workflow(
        context: &mut VmContext,
        workflow: &Workflow,
        cases: Vec<CaseConfig>
    ) -> Result<Vec<CaseConfig>, String> {
        tracing::debug!("Executing workflow: {}", workflow.name);

        let mut processed_cases = cases;

        for phase in &workflow.phases {
            match phase {
                Phase::Score(rules) => {
                    processed_cases = Self::execute_score_phase_on_cases(
                        context,
                        rules,
                        processed_cases
                    )?;
                }
                Phase::Match(rules) => {
                    processed_cases = Self::execute_match_phase_on_cases(
                        context,
                        rules,
                        processed_cases
                    )?;
                }
                Phase::Filter(filter_rule) => {
                    processed_cases = Self::execute_filter_phase(
                        context,
                        filter_rule,
                        processed_cases
                    )?;
                }
                Phase::Sort(sort_rule) => {
                    processed_cases = Self::execute_sort_phase(
                        context,
                        sort_rule,
                        processed_cases
                    )?;
                }
            }
        }

        Ok(processed_cases)
    }

    pub fn setup_case_context(context: &mut VmContext, case: &CaseConfig) -> Result<(), String> {
        context.env.enter_scope();

        context.env.insert("id", Value::Number(case.id as i64));
        context.env.insert("category", Value::String(case.category.clone()));
        context.env.insert("status", Value::String(case.status.clone()));
        context.env.insert("priority", Value::Number(case.priority as i64));
        context.env.insert("score", Value::Number(case.score));

        if let Some(customer) = &case.customer {
            context.env.insert("customer", Value::String(customer.clone()));
        } else {
            context.env.insert("customer", Value::String("".to_string()));
        }

        Ok(())
    }

    pub fn execute_score_phase(
        context: &mut VmContext,
        rules: &[Rule],
        case: &mut CaseConfig
    ) -> Result<(), String> {
        for rule in rules {
            let condition_result = ExprEvaluator::evaluate_expr(context, &rule.condition)?;

            if ExprEvaluator::is_truthy(&condition_result) {
                ActionEvaluator::execute_action(context, &rule.action, case)?;
            }
        }
        Ok(())
    }

    pub fn execute_match_phase(
        context: &mut VmContext,
        rules: &[MatchRule],
        case: &mut CaseConfig
    ) -> Result<(), String> {
        for rule in rules {
            let condition_result = ExprEvaluator::evaluate_expr(context, &rule.condition)?;

            if ExprEvaluator::is_truthy(&condition_result) {
                ActionEvaluator::execute_match_action(context, &rule.action, case)?;
                break;
            }
        }
        Ok(())
    }

    pub fn execute_score_phase_on_cases(
        context: &mut VmContext,
        rules: &[Rule],
        cases: Vec<CaseConfig>
    ) -> Result<Vec<CaseConfig>, String> {
        let mut processed_cases = Vec::new();

        for case in cases {
            let mut case_copy = case;
            Self::setup_case_context(context, &case_copy)?;

            Self::execute_score_phase(context, rules, &mut case_copy)?;

            context.env.exit_scope();
            processed_cases.push(case_copy);
        }

        Ok(processed_cases)
    }

    pub fn execute_match_phase_on_cases(
        context: &mut VmContext,
        rules: &[MatchRule],
        cases: Vec<CaseConfig>
    ) -> Result<Vec<CaseConfig>, String> {
        let mut processed_cases = Vec::new();

        for case in cases {
            let mut case_copy = case;
            Self::setup_case_context(context, &case_copy)?;

            let pre_match_vars = Self::get_persistent_variables(context);

            Self::execute_match_phase(context, rules, &mut case_copy)?;

            let post_match_vars = Self::get_persistent_variables(context);

            context.env.exit_scope();

            for (name, value) in post_match_vars {
                if !pre_match_vars.contains_key(&name) {
                    context.env.insert(name, value);
                }
            }

            processed_cases.push(case_copy);
        }

        Ok(processed_cases)
    }

    pub fn execute_filter_phase(
        context: &mut VmContext,
        filter_rule: &FilterRule,
        cases: Vec<CaseConfig>
    ) -> Result<Vec<CaseConfig>, String> {
        let mut filtered_cases = Vec::new();
        let original_count = cases.len();

        for case in cases {
            Self::setup_case_context(context, &case)?;

            let condition_result = ExprEvaluator::evaluate_expr(context, &filter_rule.condition)?;

            if ExprEvaluator::is_truthy(&condition_result) {
                filtered_cases.push(case);
            }

            context.env.exit_scope();
        }

        tracing::debug!("Filtered {} cases to {} cases", original_count, filtered_cases.len());

        Ok(filtered_cases)
    }

    pub fn execute_sort_phase(
        context: &mut VmContext,
        sort_rule: &SortRule,
        cases: Vec<CaseConfig>
    ) -> Result<Vec<CaseConfig>, String> {
        let mut case_key_pairs = Vec::new();

        for case in cases {
            Self::setup_case_context(context, &case)?;

            let sort_key = ExprEvaluator::evaluate_expr(context, &sort_rule.key)?;

            case_key_pairs.push((case, sort_key));
            context.env.exit_scope();
        }

        case_key_pairs.sort_by(|(_, a), (_, b)| {
            let cmp = Self::compare_values(a, b);
            match sort_rule.order {
                SortOrder::Asc => cmp,
                SortOrder::Desc => cmp.reverse(),
            }
        });

        let sorted_cases: Vec<CaseConfig> = case_key_pairs
            .into_iter()
            .map(|(case, _)| case)
            .collect();

        tracing::debug!("Sorted {} cases by key expression", sorted_cases.len());

        Ok(sorted_cases)
    }

    fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => a.cmp(b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            _ => {
                let a_str = Self::value_to_string(a);
                let b_str = Self::value_to_string(b);
                a_str.cmp(&b_str)
            }
        }
    }

    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Map(_) => "map".to_string(),
            Value::BuiltinFunction(_) => "builtin_function".to_string(),
            Value::UserFunction(f) => format!("user_function_{}", f.name),
        }
    }

    fn get_persistent_variables(context: &VmContext) -> std::collections::HashMap<String, Value> {
        use std::collections::HashMap;
        let mut persistent_vars = HashMap::new();

        if let Some(current_scope) = context.env.env.last() {
            for (name, value) in current_scope {
                if
                    !matches!(
                        name.as_str(),
                        "id" | "category" | "status" | "priority" | "score" | "customer"
                    ) &&
                    !matches!(value, Value::BuiltinFunction(_) | Value::UserFunction(_))
                {
                    persistent_vars.insert(name.clone(), value.clone());
                }
            }
        }

        persistent_vars
    }
}
