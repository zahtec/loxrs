use crate::{
    error::{Error, ErrorType},
    expressions::Expr,
    interpreter::Interpreter,
    statements::Stmt,
    tokens::Token,
};
use std::collections::HashMap;

struct Resolver<'src> {
    interpreter: &'src mut Interpreter<'src>,
    error: &'src Error,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver<'_> {
    fn new<'src>(interpreter: &'src mut Interpreter<'src>, error: &'src Error) -> Resolver<'src> {
        Resolver {
            interpreter,
            error,
            scopes: Vec::new(),
        }
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned(), true);
        }
    }

    fn resolve(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_loc(&mut self, expr: Expr, name: &str) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                // self.interpreter.locals.insert(expr, i);
                return;
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Print { expr } => self.resolve_expr(expr),
            Stmt::Function { name, params, body } => {
                if let Some(name) = name {
                    self.declare(&name);
                    self.define(&name);
                }

                self.scopes.push(HashMap::new());

                for param in params {
                    self.declare(&param);
                    self.define(&param);
                }

                self.resolve(body);

                self.scopes.pop();
            }
            Stmt::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(*else_branch);
                }
            }
            Stmt::Return { expr } => self.resolve_expr(expr),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*body);
            }
            Stmt::Expression { expr } => self.resolve_expr(expr),
            Stmt::Block { statements } => {
                self.resolve(statements);
            }
            Stmt::Var { name, expr } => {
                self.declare(&name);

                self.resolve_expr(expr);

                self.define(&name);
            }
            _ => (),
        }
    }

    fn resolve_expr(&mut self, expr: Expr) {
        match expr.clone() {
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Variable { name } => {
                let token = match name {
                    Token::Identifier {
                        value,
                        line,
                        column,
                    } => (value, line, column),
                    _ => unreachable!(),
                };

                if self.scopes.len() > 0 {
                    if let Some(scope) = self.scopes.last_mut() {
                        if let Some(value) = scope.get(&token.0) {
                            if *value {
                                self.error.report(
                                    (&token.1, &token.2),
                                    ErrorType::ResolverError,
                                    "Can't read local variable in its own initializer.",
                                );
                            }
                        }
                    }
                }

                self.resolve_loc(expr, &token.0);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(*value);

                let name = match name {
                    Token::Identifier { value, .. } => value,
                    _ => unreachable!(),
                };

                self.resolve_loc(expr, &name);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(*callee);

                for argument in arguments {
                    self.resolve_stmt(argument);
                }
            }
            Expr::Grouping { expression } => self.resolve_expr(*expression),
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Unary { right, .. } => self.resolve_expr(*right),
            _ => (),
        }
    }
}
