use pest::iterators::Pair;
use crate::engine::lang::ast;
use crate::engine::lang::parser::Rule;
use crate::engine::lang::builders::builder_expr::build_expr;

pub fn build_action(pair: Pair<Rule>) -> ast::Action {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr => { ast::Action::AssignScore(build_expr(inner)) }
        Rule::string => { ast::Action::Log(inner.as_str().trim_matches('"').to_string()) }
        _ => unreachable!("Unexpected action rule: {:?}", inner.as_rule()),
    }
}

pub fn build_match_action(pair: Pair<Rule>) -> ast::MatchAction {
    let ident_pair = pair.into_inner().next().unwrap();
    ast::MatchAction::AssignTo(ident_pair.as_str().to_string())
}
