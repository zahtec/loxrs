use crate::{callable::Callable, statements::Stmt, tokens::Token};
use std::fmt;

#[derive(Clone, Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Callable(Callable),
    Nil,
}

impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Boolean(true) => true,
            Literal::Boolean(false) => false,
            Literal::Nil => false,
            Literal::Number(..) => true,
            Literal::String(..) => true,
            Literal::Callable(..) => true,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Callable(..) => write!(f, "<fn>"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Stmt>,
    },
}
