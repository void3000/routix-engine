use crate::{
    engine::{
        lang::ast::{Action, MatchAction, Value},
        vm::{context::VmContext, evaluators::expr_evaluator::ExprEvaluator},
    },
    models::case::CaseConfig,
};
use std::collections::HashMap;

pub struct ActionEvaluator;

impl ActionEvaluator {
    pub fn execute_action(
        context: &mut VmContext,
        action: &Action,
        case: &mut CaseConfig,
    ) -> Result<(), String> {
        match action {
            Action::AssignScore(expr) => {
                let score_value = ExprEvaluator::evaluate_expr(context, expr)?;
                match score_value {
                    Value::Number(n) => {
                        case.score = n;
                        context.env.set("score", Value::Number(n));
                        tracing::debug!("Assigned score: {}", n);
                    }
                    _ => {
                        return Err("Score must be a number".to_string());
                    }
                }
            }
            Action::Log(message) => {
                tracing::debug!("LOG: {}", message);
            }
            Action::Assign(var_name) => {
                context.env.insert(var_name, Value::Bool(true));
            }
        }
        Ok(())
    }

    pub fn execute_match_action(
        context: &mut VmContext,
        action: &MatchAction,
        case: &mut CaseConfig,
    ) -> Result<(), String> {
        match action {
            MatchAction::AssignTo(var_name) => {
                let case_map = Self::case_to_map(case);
                context.env.insert(var_name, Value::Map(case_map));
                tracing::debug!("Assigned case to variable: {}", var_name);
            }
        }
        Ok(())
    }

    fn case_to_map(case: &CaseConfig) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::String(case.id.to_string()));
        map.insert("category".to_string(), Value::String(case.category.clone()));
        map.insert("status".to_string(), Value::String(case.status.clone()));
        map.insert(
            "priority".to_string(),
            Value::String(case.priority.to_string()),
        );
        map.insert("score".to_string(), Value::String(case.score.to_string()));
        if let Some(customer) = &case.customer {
            map.insert("customer".to_string(), Value::String(customer.clone()));
        }
        map
    }
}
