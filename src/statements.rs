use crate::expressions::Expr;

#[derive(Clone, Debug)]
pub enum Stmt {
    Print {
        expr: Expr,
    },
    Var {
        name: String,
        expr: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Break {
        line: usize,
        column: usize,
    },
    Return {
        expr: Expr,
    },
    Conditional {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}
