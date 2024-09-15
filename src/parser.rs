use crate::{
    error::{Error, ErrorType},
    expressions::{Expr, Literal},
    statements::Stmt,
    tokens::Token,
};

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ("?" expression ":" expression)? ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

pub struct Parser<'src> {
    tokens: Vec<Token>,
    error: &'src Error,
    current: usize,
    in_function: bool,
}

impl Parser<'_> {
    pub fn new<'src>(error: &'src Error) -> Parser<'src> {
        Parser {
            tokens: Vec::new(),
            error,
            current: 0,
            in_function: false,
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn is_end(&self) -> bool {
        if let Token::Eof { .. } = self.peek() {
            true
        } else {
            false
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<Stmt>, Vec<Stmt>> {
        self.current = 0;

        self.tokens = tokens;

        let mut stmts: Vec<Stmt> = Vec::new();

        while !self.is_end() {
            stmts.push(match self.parse_token() {
                Ok(stmt) => stmt,
                Err(_) => return Err(stmts),
            });
        }

        Ok(stmts)
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn synchronize(&mut self) {
        while !self.is_end() {
            if let Token::Semicolon { .. } = self.peek() {
                return;
            }

            match self.peek() {
                Token::Class { .. }
                | Token::Fun { .. }
                | Token::Var { .. }
                | Token::For { .. }
                | Token::If { .. }
                | Token::While { .. }
                | Token::Print { .. }
                | Token::Return { .. } => return,
                _ => (),
            }

            self.current += 1;
        }
    }

    fn check_semicolon(&mut self, message: &str) -> bool {
        let prev = self.previous();

        if let Token::Semicolon { .. } = self.peek() {
            self.current += 1;
            true
        } else {
            self.error
                .report(prev.location(), ErrorType::ParserError, message);
            self.synchronize();
            false
        }
    }

    fn parse_token(&mut self) -> Result<Stmt, ()> {
        match self.peek() {
            Token::Identifier { .. } => {
                let expr = self.assignment()?;

                if self.in_function {
                    Ok(Stmt::Expression { expr })
                } else {
                    if !self.check_semicolon("Expect ';' after expression.") {
                        return Err(());
                    }

                    Ok(Stmt::Expression { expr })
                }
            }
            Token::Print { .. } => {
                self.current += 1;

                let expr = self.assignment()?;

                if self.check_semicolon("Expected ';' after statement.") {
                    return Ok(Stmt::Print { expr });
                }

                Err(())
            }
            Token::Break { line, column } => {
                self.current += 1;

                if self.check_semicolon("Expected ';' after statement.") {
                    return Ok(Stmt::Break { line, column });
                }

                Err(())
            }
            Token::Var { .. } => {
                self.current += 1;

                let token = self.peek();
                let name: String;

                self.current += 1;

                if let Token::Identifier { value, .. } = token {
                    name = value;
                } else {
                    self.error.report(
                        token.location(),
                        ErrorType::ParserError,
                        "Expected identifier.",
                    );
                    return Err(());
                }

                if let Token::Equal { .. } = self.peek() {
                    self.current += 1;
                } else {
                    if self.check_semicolon("Expected ';' after expression.") {
                        return Ok(Stmt::Var {
                            name,
                            expr: Expr::Literal {
                                value: Literal::Nil,
                            },
                        });
                    }
                }

                let expr = self.assignment()?;

                if self.check_semicolon("Expected ';' after expression.") {
                    return Ok(Stmt::Var { name, expr });
                }

                Err(())
            }
            Token::LeftBrace { .. } => {
                self.current += 1;

                let mut stmts: Vec<Stmt> = Vec::new();

                while !self.is_end() {
                    if let Token::RightBrace { .. } = self.peek() {
                        break;
                    } else {
                        stmts.push(self.parse_token()?);
                    }
                }

                if let Token::RightBrace { .. } = self.peek() {
                    self.current += 1;
                    Ok(Stmt::Block { statements: stmts })
                } else {
                    self.error.report(
                        self.peek().location(),
                        ErrorType::ParserError,
                        "Expected '}' after block.",
                    );
                    self.synchronize();
                    Err(())
                }
            }
            Token::If { .. } => {
                self.current += 1;

                if let Token::LeftParen { .. } = self.peek() {
                    self.current += 1;

                    let condition = self.assignment()?;

                    if let Token::RightParen { .. } = self.peek() {
                        self.current += 1;

                        let then_branch = Box::new(self.parse_token()?);

                        if let Token::Else { .. } = self.peek() {
                            self.current += 1;

                            let else_branch = Some(Box::new(self.parse_token()?));
                            return Ok(Stmt::Conditional {
                                condition,
                                then_branch,
                                else_branch,
                            });
                        } else {
                            return Ok(Stmt::Conditional {
                                condition,
                                then_branch,
                                else_branch: None,
                            });
                        }
                    } else {
                        self.error.report(
                            self.peek().location(),
                            ErrorType::ParserError,
                            "Expected ')' after condition.",
                        );
                        self.synchronize();
                    }
                } else {
                    self.error.report(
                        self.previous().location(),
                        ErrorType::ParserError,
                        "Expected '(' after 'if'.",
                    );
                    self.synchronize();
                }

                Err(())
            }
            Token::While { .. } => {
                self.current += 1;

                if let Token::LeftParen { .. } = self.peek() {
                    self.current += 1;

                    let condition = self.assignment()?;

                    if let Token::RightParen { .. } = self.peek() {
                        self.current += 1;

                        let body = Box::new(self.parse_token()?);

                        return Ok(Stmt::While { condition, body });
                    } else {
                        self.error.report(
                            self.peek().location(),
                            ErrorType::ParserError,
                            "Expected ')' after condition.",
                        );
                        self.synchronize();
                    }
                } else {
                    self.error.report(
                        self.previous().location(),
                        ErrorType::ParserError,
                        "Expected '(' after 'while'.",
                    );
                }

                Err(())
            }
            Token::For { .. } => {
                self.current += 1;

                if let Token::LeftParen { .. } = self.peek() {
                    self.current += 1;

                    let init = match self.peek() {
                        Token::Semicolon { .. } => {
                            self.current += 1;
                            None
                        }
                        _ => Some(self.parse_token()?),
                    };

                    let condition = match self.peek() {
                        Token::Semicolon { .. } => {
                            self.current += 1;
                            Expr::Literal {
                                value: Literal::Boolean(true),
                            }
                        }
                        _ => {
                            let condition = self.assignment()?;

                            if self.check_semicolon("Expected ';' after expression.") {
                                condition
                            } else {
                                return Err(());
                            }
                        }
                    };

                    let incr = match self.peek() {
                        Token::Semicolon { .. } => {
                            self.current += 1;
                            None
                        }
                        _ => Some(self.assignment()?),
                    };

                    if let Token::RightParen { .. } = self.peek() {
                        self.current += 1;
                    } else {
                        self.error.report(
                            self.peek().location(),
                            ErrorType::ParserError,
                            "Expected ')' after loop increment.",
                        );
                        self.synchronize();
                        return Err(());
                    }

                    let mut body = self.parse_token()?;

                    if let Some(incr) = incr {
                        body = Stmt::Block {
                            statements: vec![body, Stmt::Expression { expr: incr }],
                        };
                    }

                    body = Stmt::While {
                        condition,
                        body: Box::new(body),
                    };

                    if let Some(init) = init {
                        body = Stmt::Block {
                            statements: vec![init, body],
                        };
                    }

                    Ok(body)
                } else {
                    self.error.report(
                        self.previous().location(),
                        ErrorType::ParserError,
                        "Expected '(' after 'for'.",
                    );
                    self.synchronize();
                    Err(())
                }
            }
            Token::Fun { .. } => {
                self.current += 1;

                let mut name = None;

                if let Token::Identifier { value, .. } = self.peek() {
                    self.current += 1;
                    name = Some(value);
                }

                if let Token::LeftParen { .. } = self.peek() {
                    self.current += 1;

                    let mut params = Vec::new();

                    while !self.is_end() {
                        let token = self.peek();

                        if let Token::RightParen { .. } = token {
                            self.current += 1;

                            let stmt = self.parse_token()?;

                            match stmt {
                                Stmt::Block { statements } => {
                                    return Ok(Stmt::Function {
                                        name,
                                        params,
                                        body: statements,
                                    })
                                }
                                _ => {
                                    self.error.report(
                                        self.peek().location(),
                                        ErrorType::ParserError,
                                        "Expected block after function declaration.",
                                    );
                                    self.synchronize();
                                    return Err(());
                                }
                            }
                        } else {
                            if params.len() >= 255 {
                                self.error.report(
                                    token.location(),
                                    ErrorType::ParserError,
                                    "Can not have more than 255 parameters.",
                                );
                            }

                            if let Token::Identifier { value, .. } = self.peek() {
                                self.current += 1;
                                params.push(value);
                            } else {
                                self.error.report(
                                    token.location(),
                                    ErrorType::ParserError,
                                    "Expected identifier.",
                                );
                            }

                            let token = self.peek();

                            if let Token::RightParen { .. } = token {
                                continue;
                            }

                            if let Token::Comma { .. } = token {
                                self.current += 1;
                            } else {
                                self.error.report(
                                    token.location(),
                                    ErrorType::ParserError,
                                    "Expected ')' or ',' after parameter.",
                                );
                                self.synchronize();
                                return Err(());
                            }
                        }
                    }

                    self.error.report(
                        self.peek().location(),
                        ErrorType::ParserError,
                        "Expected ')' after parameters.",
                    );
                    self.synchronize();
                } else {
                    self.error.report(
                        self.previous().location(),
                        ErrorType::ParserError,
                        "Expected '(' after function name.",
                    );
                    self.synchronize();
                }

                Err(())
            }
            Token::Return { .. } => {
                self.current += 1;

                let expr = self.assignment()?;

                if self.check_semicolon("Expected ';' after return value.") {
                    Ok(Stmt::Return { expr })
                } else {
                    Err(())
                }
            }
            _ => Ok(Stmt::Expression {
                expr: self.assignment()?,
            }),
        }
    }

    fn assignment(&mut self) -> Result<Expr, ()> {
        let expr = self.or()?;

        if let Expr::Variable { name } = &expr {
            if let Token::Identifier { .. } = name {
                match self.peek() {
                    Token::Equal { .. } => {
                        self.current += 1;

                        let value = Box::new(self.assignment()?);

                        return Ok(Expr::Assign {
                            name: name.clone(),
                            value,
                        });
                    }
                    Token::PlusEqual { line, column } => {
                        self.current += 1;

                        let value = Box::new(self.assignment()?);

                        return Ok(Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable { name: name.clone() }),
                                operator: Token::Plus { line, column },
                                right: value,
                            }),
                        });
                    }
                    Token::MinusEqual { line, column } => {
                        self.current += 1;

                        let value = Box::new(self.assignment()?);

                        return Ok(Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable { name: name.clone() }),
                                operator: Token::Minus { line, column },
                                right: value,
                            }),
                        });
                    }
                    Token::StarEqual { line, column } => {
                        self.current += 1;

                        let value = Box::new(self.assignment()?);

                        return Ok(Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable { name: name.clone() }),
                                operator: Token::Star { line, column },
                                right: value,
                            }),
                        });
                    }
                    _ => (),
                }
            } else {
                panic!("Invalid identifier.");
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ()> {
        let expr = self.and()?;

        if let Token::Or { .. } = self.peek() {
            self.current += 1;
            Ok(Expr::Logical {
                left: Box::new(expr),
                operator: self.previous(),
                right: Box::new(self.and()?),
            })
        } else {
            Ok(expr)
        }
    }

    fn and(&mut self) -> Result<Expr, ()> {
        let expr = self.equality()?;

        if let Token::And { .. } = self.peek() {
            self.current += 1;
            Ok(Expr::Logical {
                left: Box::new(expr),
                operator: self.previous(),
                right: Box::new(self.equality()?),
            })
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        let mut expr = self.comparison()?;

        while let Token::BangEqual { .. } | Token::EqualEqual { .. } = self.peek() && !self.is_end() {
            self.current += 1;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous(),
                right: Box::new(self.comparison()?),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut expr = self.term()?;

        while let Token::Greater { .. } | Token::GreaterEqual { .. } | Token::Less { .. } | Token::LessEqual { .. } = self.peek() && !self.is_end() {
            self.current += 1;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous(),
                right: Box::new(self.term()?),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ()> {
        let mut expr = self.factor()?;

        while let Token::Minus { .. } | Token::Plus { .. } = self.peek() && !self.is_end() {
            self.current += 1;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator:self.previous(),
                right: Box::new(self.factor()?),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        let mut expr = self.unary()?;

        while let Token::Slash { .. } | Token::Star { .. } = self.peek() && !self.is_end() {
            self.current += 1;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous(),
                right: Box::new(self.unary()?),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        let mut expr = self.call()?;

        if let Token::Bang { .. } | Token::Minus { .. } = self.peek() && !self.is_end() {
            self.current += 1;

            let operator = self.previous();

            expr = Expr::Unary {
                operator,
                right: Box::new(self.unary()?),
            };
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, ()> {
        let mut expr = self.primary()?;

        loop {
            let paren = self.peek();

            if let Token::LeftParen { .. } = paren {
                self.current += 1;

                let mut arguments = Vec::new();

                self.in_function = true;

                while !self.is_end() {
                    let token = self.peek();

                    if let Token::RightParen { .. } = token {
                        expr = Expr::Call {
                            callee: Box::new(expr),
                            paren: token,
                            arguments,
                        };

                        break;
                    } else {
                        if arguments.len() >= 255 {
                            self.error.report(
                                token.location(),
                                ErrorType::ParserError,
                                "Can not have more than 255 arguments.",
                            );
                        }

                        arguments.push(self.parse_token()?);

                        let token = self.peek();

                        if let Token::RightParen { .. } = token {
                            continue;
                        }

                        if let Token::Comma { .. } = token {
                            self.current += 1;
                        } else {
                            self.error.report(
                                token.location(),
                                ErrorType::ParserError,
                                "Expected ')' or ',' after argument.",
                            );
                            self.synchronize();
                            return Err(());
                        }
                    }
                }

                self.in_function = false;

                if let Token::RightParen { .. } = self.peek() {
                    self.current += 1;
                } else {
                    self.error.report(
                        paren.location(),
                        ErrorType::ParserError,
                        "Expected ')' after arguments.",
                    );
                    self.synchronize();
                    return Err(());
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        let token = self.peek();

        self.current += 1;

        match token {
            Token::Number { value, .. } => Ok(Expr::Literal {
                value: Literal::Number(value),
            }),

            Token::String { value, .. } => Ok(Expr::Literal {
                value: Literal::String(value.to_string()),
            }),

            Token::True { .. } => Ok(Expr::Literal {
                value: Literal::Boolean(true),
            }),

            Token::False { .. } => Ok(Expr::Literal {
                value: Literal::Boolean(false),
            }),

            Token::Nil { .. } => Ok(Expr::Literal {
                value: Literal::Nil,
            }),

            Token::Identifier { .. } => Ok(Expr::Variable { name: token }),

            Token::LeftParen { .. } => {
                let mut expr = self.assignment()?;

                let token = self.peek();

                if let Token::RightParen { .. } = token {
                    expr = Expr::Grouping {
                        expression: Box::new(expr),
                    };
                    self.current += 1;
                } else {
                    self.error.report(
                        token.location(),
                        ErrorType::ParserError,
                        &format!("Expected {:?} after expression.", token),
                    );
                    self.synchronize();
                    return Err(());
                }

                Ok(expr)
            }

            _ => {
                self.error.report(
                    token.location(),
                    ErrorType::ParserError,
                    "Expected expression.",
                );
                Err(())
            }
        }
    }
}
