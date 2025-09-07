use pest::iterators::Pair;
use crate::engine::lang::ast;
use crate::engine::lang::parser::Rule;
use crate::engine::lang::builders::builder_action::{ build_action, build_match_action };
use crate::engine::lang::builders::builder_expr::build_expr;

pub fn build_rule(pair: Pair<Rule>) -> ast::Rule {
    let mut condition = None;
    let mut action = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => {
                condition = Some(build_expr(inner));
            }
            Rule::action => {
                action = Some(build_action(inner));
            }
            _ => {}
        }
    }

    ast::Rule {
        condition: condition.unwrap(),
        action: action.unwrap(),
    }
}

pub fn build_match_rule(pair: Pair<Rule>) -> ast::MatchRule {
    let mut condition = None;
    let mut action = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => {
                condition = Some(build_expr(inner));
            }
            Rule::match_action => {
                action = Some(build_match_action(inner));
            }
            _ => {}
        }
    }

    ast::MatchRule {
        condition: condition.unwrap(),
        action: action.unwrap(),
    }
}
