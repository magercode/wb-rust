use crate::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let { name: String, value: Expr },
    Assign { name: String, value: Expr },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    WhileInit {
        init: Box<Stmt>,
        condition: Expr,
        body: Vec<Stmt>,
    },
    While { condition: Expr, body: Vec<Stmt> },
    ForEach { name: String, iterable: Expr, body: Vec<Stmt> },
    Function { name: String, params: Vec<String>, body: Vec<Stmt> },
    Return(Option<Expr>),
    Break,
    Continue,
    Import { module: Expr },
    Export { value: Expr },
}
