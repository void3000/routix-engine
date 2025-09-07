use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDef>,
    pub workflows: Vec<Workflow>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone)]
pub enum FunctionBody {
    Expression(Expr),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    Return(Expr),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub phases: Vec<Phase>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Score(Vec<Rule>),
    Match(Vec<MatchRule>),
    Filter(FilterRule),
    Sort(SortRule),
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub condition: Expr,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub struct MatchRule {
    pub condition: Expr,
    pub action: MatchAction,
}

#[derive(Debug, Clone)]
pub enum Action {
    AssignScore(Expr),
    Log(String),
    Assign(String),
}

#[derive(Debug, Clone)]
pub enum MatchAction {
    AssignTo(String),
}

#[derive(Debug, Clone)]
pub struct FilterRule {
    pub condition: Expr,
}

#[derive(Debug, Clone)]
pub struct SortRule {
    pub key: Expr,
    pub order: SortOrder,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    MemberAccess {
        object: String,
        property: String,
    },
    List(Vec<Expr>),
    Ident(String),
    Number(i64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Eq,
    Neq,
    In,
    Gt,
    Lt,
    Ge,
    Le,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Null,
    Map(HashMap<String, Value>),
    BuiltinFunction(fn(&[Value]) -> Result<Value, String>),
    UserFunction(FunctionDef),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::BuiltinFunction(a), Value::BuiltinFunction(b)) => {
                std::ptr::eq(a as *const _, b as *const _)
            }
            (Value::UserFunction(a), Value::UserFunction(b)) => {
                a.name == b.name && a.params == b.params
            }
            _ => false,
        }
    }
}
