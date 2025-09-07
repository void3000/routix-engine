use pest::iterators::{ Pair, Pairs };
use crate::engine::lang::ast;
use crate::engine::lang::parser::Rule;
use crate::engine::lang::builders::builder_rule::{ build_rule, build_match_rule };
use crate::engine::lang::builders::builder_expr::build_expr;

pub fn build_program(pairs: Pairs<Rule>) -> ast::Program {
    let mut functions = Vec::new();
    let mut workflows = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::function_def => functions.push(build_function_def(inner)),
                    Rule::workflow => workflows.push(build_workflow(inner)),
                    _ => {}
                }
            }
        }
    }

    ast::Program { functions, workflows }
}

pub fn build_workflows(pairs: Pairs<Rule>) -> Vec<ast::Workflow> {
    pairs
        .flat_map(|pair| {
            if pair.as_rule() == Rule::program {
                pair.into_inner()
                    .filter(|p| p.as_rule() == Rule::workflow)
                    .map(build_workflow)
                    .collect::<Vec<_>>()
            } else if pair.as_rule() == Rule::workflow {
                vec![build_workflow(pair)]
            } else {
                vec![]
            }
        })
        .collect()
}

pub fn build_function_def(pair: Pair<Rule>) -> ast::FunctionDef {
    let mut name = String::new();
    let mut params = Vec::new();
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::param_list => {
                params = inner
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::ident)
                    .map(|p| p.as_str().to_string())
                    .collect();
            }
            Rule::function_body => {
                body = Some(build_function_body(inner));
            }
            _ => {}
        }
    }

    ast::FunctionDef {
        name,
        params,
        body: body.unwrap(),
    }
}

pub fn build_function_body(pair: Pair<Rule>) -> ast::FunctionBody {
    let mut inner_pairs = pair.into_inner();
    let first = inner_pairs.next().unwrap();

    match first.as_rule() {
        Rule::expr => ast::FunctionBody::Expression(build_expr(first)),
        Rule::statement => {
            let mut statements = vec![build_statement(first)];
            statements.extend(inner_pairs.map(build_statement));
            ast::FunctionBody::Block(statements)
        }
        _ => {
            let mut statements = Vec::new();
            statements.push(build_statement(first));
            statements.extend(inner_pairs.map(build_statement));
            ast::FunctionBody::Block(statements)
        }
    }
}

pub fn build_statement(pair: Pair<Rule>) -> ast::Statement {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::let_statement => build_let_statement(inner),
        Rule::assign_statement => build_assign_statement(inner),
        Rule::if_statement => build_if_statement(inner),
        Rule::return_statement => build_return_statement(inner),
        Rule::expr_statement => build_expr_statement(inner),
        _ => unreachable!("Unexpected statement type: {:?}", inner.as_rule()),
    }
}

pub fn build_let_statement(pair: Pair<Rule>) -> ast::Statement {
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::expr => {
                value = Some(build_expr(inner));
            }
            _ => {}
        }
    }

    ast::Statement::Let {
        name,
        value: value.unwrap(),
    }
}

pub fn build_assign_statement(pair: Pair<Rule>) -> ast::Statement {
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::expr => {
                value = Some(build_expr(inner));
            }
            _ => {}
        }
    }

    ast::Statement::Assign {
        name,
        value: value.unwrap(),
    }
}

pub fn build_if_statement(pair: Pair<Rule>) -> ast::Statement {
    let mut condition = None;
    let mut then_body = Vec::new();
    let mut else_body = None;
    let mut in_else = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr if condition.is_none() => {
                condition = Some(build_expr(inner));
            }
            Rule::statement => {
                if in_else {
                    if else_body.is_none() {
                        else_body = Some(Vec::new());
                    }
                    else_body.as_mut().unwrap().push(build_statement(inner));
                } else {
                    then_body.push(build_statement(inner));
                }
            }
            _ => {
                if inner.as_str() == "else" {
                    in_else = true;
                }
            }
        }
    }

    ast::Statement::If {
        condition: condition.unwrap(),
        then_body,
        else_body,
    }
}

pub fn build_return_statement(pair: Pair<Rule>) -> ast::Statement {
    let expr = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::expr)
        .map(build_expr)
        .unwrap();

    ast::Statement::Return(expr)
}

pub fn build_expr_statement(pair: Pair<Rule>) -> ast::Statement {
    let expr = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::expr)
        .map(build_expr)
        .unwrap();

    ast::Statement::Expression(expr)
}

pub fn build_workflow(pair: Pair<Rule>) -> ast::Workflow {
    let mut name = String::new();
    let mut phases = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::phase => phases.push(build_phase(inner)),
            _ => {}
        }
    }

    ast::Workflow { name, phases }
}

pub fn build_phase(pair: Pair<Rule>) -> ast::Phase {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::score_phase => {
            let rules = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::rule)
                .map(build_rule)
                .collect();
            ast::Phase::Score(rules)
        }
        Rule::match_phase => {
            let rules = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::match_rule)
                .map(build_match_rule)
                .collect();
            ast::Phase::Match(rules)
        }
        Rule::filter_phase => {
            let condition = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::expr)
                .map(build_expr)
                .unwrap();
            ast::Phase::Filter(ast::FilterRule { condition })
        }
        Rule::sort_phase => {
            let mut key = None;
            let mut order = ast::SortOrder::Asc; // Default to ascending
            
            for inner_pair in inner.into_inner() {
                match inner_pair.as_rule() {
                    Rule::expr => {
                        key = Some(build_expr(inner_pair));
                    }
                    Rule::sort_order => {
                        order = match inner_pair.as_str() {
                            "desc" => ast::SortOrder::Desc,
                            _ => ast::SortOrder::Asc,
                        };
                    }
                    _ => {}
                }
            }
            
            ast::Phase::Sort(ast::SortRule {
                key: key.unwrap(),
                order,
            })
        }
        _ => unreachable!("Unexpected phase type: {:?}", inner.as_rule()),
    }
}
