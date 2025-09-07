use pest::iterators::Pair;
use crate::engine::lang::{ ast, parser::Rule };

pub fn build_expr(pair: Pair<Rule>) -> ast::Expr {
    match pair.as_rule() {
        Rule::number => ast::Expr::Number(pair.as_str().parse().unwrap()),
        Rule::string => ast::Expr::String(pair.as_str().trim_matches('"').to_string()),
        Rule::ident | Rule::bool =>
            match pair.as_str() {
                "true" => ast::Expr::Bool(true),
                "false" => ast::Expr::Bool(false),
                other => ast::Expr::Ident(other.to_string()),
            }
        Rule::list => ast::Expr::List(pair.into_inner().map(build_expr).collect()),
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let args = inner
                .flat_map(|p| {
                    if p.as_rule() == Rule::arg_list {
                        p.into_inner().map(build_expr).collect::<Vec<_>>()
                    } else {
                        vec![build_expr(p)]
                    }
                })
                .collect();
            ast::Expr::FunctionCall { name, args }
        }
        Rule::member_access => {
            let parts: Vec<&str> = pair.as_str().split('.').collect();
            if parts.len() == 2 {
                ast::Expr::MemberAccess {
                    object: parts[0].trim().to_string(),
                    property: parts[1].trim().to_string(),
                }
            } else {
                // For now, only support single-level member access (object.property)
                // Could be extended to support deeper nesting later
                ast::Expr::MemberAccess {
                    object: parts[0].trim().to_string(),
                    property: parts[1..].join(".").trim().to_string(),
                }
            }
        }
        Rule::expr | Rule::primary_expr => build_expr(pair.into_inner().next().unwrap()),
        Rule::or_expr => build_binary_chain(pair, ast::BinaryOperator::Or),
        Rule::and_expr => build_binary_chain(pair, ast::BinaryOperator::And),
        Rule::add_expr =>
            build_binary_from_text(
                pair,
                &[
                    ("+", ast::BinaryOperator::Add),
                    ("-", ast::BinaryOperator::Sub),
                ]
            ),
        Rule::mul_expr =>
            build_binary_from_text(
                pair,
                &[
                    ("*", ast::BinaryOperator::Mul),
                    ("/", ast::BinaryOperator::Div),
                ]
            ),
        Rule::comp_expr => build_comparison(pair),
        Rule::unary_expr => build_unary_expr(pair),
        _ => unreachable!("Unexpected expr: {:?}", pair.as_rule()),
    }
}

fn build_binary_chain(pair: Pair<Rule>, op: ast::BinaryOperator) -> ast::Expr {
    let mut inner = pair.into_inner();
    let first = build_expr(inner.next().unwrap());
    inner.fold(first, |left, p| ast::Expr::BinaryOp {
        left: Box::new(left),
        op: op.clone(),
        right: Box::new(build_expr(p)),
    })
}

fn build_binary_from_text(pair: Pair<Rule>, ops: &[(&str, ast::BinaryOperator)]) -> ast::Expr {
    let full_text = pair.as_str();
    let inner: Vec<_> = pair.into_inner().collect();
    if inner.len() == 1 {
        return build_expr(inner[0].clone());
    }

    let mut result = build_expr(inner[0].clone());
    let mut current_pos = 0;

    for i in 1..inner.len() {
        let prev_text = inner[i - 1].as_str();
        let curr_text = inner[i].as_str();

        let prev_end =
            full_text[current_pos..].find(prev_text).unwrap() + prev_text.len() + current_pos;
        let curr_start = full_text[prev_end..].find(curr_text).unwrap() + prev_end;
        let op_text = full_text[prev_end..curr_start].trim();

        if let Some((_, op)) = ops.iter().find(|(s, _)| *s == op_text) {
            result = ast::Expr::BinaryOp {
                left: Box::new(result),
                op: op.clone(),
                right: Box::new(build_expr(inner[i].clone())),
            };
        }
        current_pos = curr_start;
    }
    result
}

fn build_comparison(pair: Pair<Rule>) -> ast::Expr {
    let full_text = pair.as_str();
    let inner: Vec<_> = pair.into_inner().collect();
    if inner.len() == 1 {
        return build_expr(inner[0].clone());
    }

    let left = build_expr(inner[0].clone());
    let right = build_expr(inner[1].clone());

    let left_text = inner[0].as_str();
    let right_text = inner[1].as_str();

    let left_end = full_text.find(left_text).unwrap() + left_text.len();
    let right_start = full_text.rfind(right_text).unwrap();
    let op_text = full_text[left_end..right_start].trim();

    let op = match op_text {
        "==" => ast::BinaryOperator::Eq,
        "!=" => ast::BinaryOperator::Neq,
        "in" => ast::BinaryOperator::In,
        ">" => ast::BinaryOperator::Gt,
        "<" => ast::BinaryOperator::Lt,
        ">=" => ast::BinaryOperator::Ge,
        "<=" => ast::BinaryOperator::Le,
        _ => unreachable!("Unexpected comparison operator: {}", op_text),
    };

    ast::Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) }
}

fn build_unary_expr(pair: Pair<Rule>) -> ast::Expr {
    let full_text = pair.as_str();
    let mut inner = pair.into_inner();
    let expr = build_expr(inner.next().unwrap());

    let prefix = &full_text[..full_text.len() - expr_to_str_len(&expr)];
    prefix
        .chars()
        .rev()
        .fold(expr, |acc, ch| {
            match ch {
                '-' => ast::Expr::UnaryOp { op: ast::UnaryOperator::Neg, expr: Box::new(acc) },
                '!' => ast::Expr::UnaryOp { op: ast::UnaryOperator::Not, expr: Box::new(acc) },
                _ => acc,
            }
        })
}

fn expr_to_str_len(expr: &ast::Expr) -> usize {
    match expr {
        ast::Expr::Number(n) => n.to_string().len(),
        ast::Expr::String(s) => s.len() + 2,
        ast::Expr::Ident(s) => s.len(),
        _ => 1, // fallback
    }
}
