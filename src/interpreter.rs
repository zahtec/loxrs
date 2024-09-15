use crate::{
    callable::Callable,
    environment::Environment,
    error::{Error, ErrorType},
    expressions::{Expr, Literal},
    statements::Stmt,
    tokens::Token,
};
use std::{
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Interpreter<'src> {
    error: &'src Error,
    environment: Environment,
    pub locals: HashMap<Expr, usize>,
    repl: bool,
    is_loop: bool,
}

impl Interpreter<'_> {
    pub fn new<'src>(
        error: &'src Error,
        environment: Environment,
        repl: bool,
    ) -> Interpreter<'src> {
        let mut environment = Environment::new(Some(Box::new(environment)));

        environment.values.insert(
            String::from("clock"),
            Literal::Callable(Callable::new(
                vec![],
                Rc::new(|_, _, _| {
                    Ok(Literal::Number(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                    ))
                }),
            )),
        );

        Interpreter {
            error,
            environment,
            locals: HashMap::new(),
            repl,
            is_loop: false,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<Literal, ()> {
        let mut result = Ok(Literal::Nil);

        for stmt in statements {
            match stmt {
                Stmt::Print { expr } => {
                    let val = self.evaluate(&expr)?;

                    println!("{val}");
                }
                Stmt::Var { name, expr } => {
                    let val = self.evaluate(&expr)?;

                    self.environment.bind(&name, val);
                }
                Stmt::Block { statements } => {
                    self.environment = Environment::new(Some(Box::new(self.environment.clone())));
                    self.interpret(statements)?;
                    self.environment = *self.environment.parent.clone().unwrap();
                }
                Stmt::Conditional {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let condition_val = self.evaluate(&condition)?;

                    if condition_val.is_truthy() {
                        self.interpret(vec![*then_branch])?;
                    } else if let Some(else_branch) = else_branch {
                        self.interpret(vec![*else_branch])?;
                    }
                }
                Stmt::While { condition, body } => {
                    self.is_loop = true;
                    while let Ok(condition_val) = self.evaluate(&condition) {
                        if condition_val.is_truthy() {
                            if let Err(_) = self.interpret(vec![*body.clone()]) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    self.is_loop = false;
                }
                Stmt::Break { line, column } => {
                    if self.is_loop {
                        return Err(());
                    } else {
                        self.error.report(
                            (&line, &column),
                            ErrorType::RuntimeError,
                            "Can not break outside of a loop.",
                        );
                        return Err(());
                    }
                }
                Stmt::Return { expr } => {
                    let val = self.evaluate(&expr)?;

                    return Ok(val);
                }
                Stmt::Function { name, params, body } => {
                    let func = Literal::Callable(Callable::new(
                        params,
                        Rc::new(move |interpreter, parameters, args| {
                            let original_env = interpreter.environment.clone();

                            let mut environment =
                                Environment::new(Some(Box::new(interpreter.environment.clone())));

                            for (i, param) in parameters.iter().enumerate() {
                                environment.bind(param, args[i].clone());
                            }

                            interpreter.environment = environment;

                            let res = interpreter.interpret(body.clone());

                            interpreter.environment = original_env.clone();

                            res
                        }),
                    ));

                    if let Some(name) = name {
                        self.environment.bind(&name, func);
                    } else {
                        return Ok(func);
                    }
                }
                Stmt::Expression { expr } => {
                    let literal = self.evaluate(&expr)?;

                    if self.repl {
                        println!("{literal}");
                    }

                    result = Ok(literal);
                }
            }
        }

        result
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, ()> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Unary { operator, right } => match operator {
                Token::Minus { line, column } => match self.evaluate(right) {
                    Ok(Literal::Number(number)) => Ok(Literal::Number(-number)),
                    Ok(_) => {
                        self.error.report(
                            (line, column),
                            ErrorType::RuntimeError,
                            "Operator '-' can only be applied to numbers.",
                        );
                        Err(())
                    }
                    Err(_) => Err(()),
                },
                Token::Bang { .. } => match self.evaluate(right) {
                    Ok(Literal::Boolean(value)) => Ok(Literal::Boolean(!value)),
                    Ok(Literal::Nil) => Ok(Literal::Boolean(true)),
                    Ok(_) => Ok(Literal::Boolean(false)),
                    Err(_) => Err(()),
                },
                _ => unreachable!(),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator {
                    Token::Minus { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Number(left - right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '-' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::Plus { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Number(left + right))
                        }
                        (Literal::Number(left), Literal::String(right)) => {
                            Ok(Literal::String(left.to_string() + &right))
                        }
                        (Literal::String(left), Literal::String(right)) => {
                            Ok(Literal::String(left.to_owned() + &right))
                        }
                        (Literal::String(left), Literal::Number(right)) => {
                            Ok(Literal::String(left.to_owned() + &right.to_string()))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '+' can only be applied to numbers or strings",
                            );
                            Err(())
                        }
                    },
                    Token::Slash { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            if left == 0.0 && right == 0.0 {
                                self.error.report(
                                    (line, column),
                                    ErrorType::RuntimeError,
                                    "Can not divide by 0",
                                );
                                Err(())
                            } else {
                                Ok(Literal::Number(left / right))
                            }
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '/' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::Star { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Number(left * right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '*' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::Greater { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left > right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '>' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::GreaterEqual { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left >= right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '>' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::Less { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left < right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '<' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::LessEqual { line, column } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left <= right))
                        }
                        (_, _) => {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                "Operator '<' can only be applied to numbers",
                            );
                            Err(())
                        }
                    },
                    Token::EqualEqual { .. } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left == right))
                        }
                        (Literal::Number(..), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(right))
                        }
                        (Literal::String(left), Literal::String(right)) => {
                            Ok(Literal::Boolean(left == right))
                        }
                        (Literal::String(..), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(right))
                        }
                        (Literal::Boolean(left), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(left == right))
                        }
                        (Literal::Boolean(left), Literal::Number(..)) => Ok(Literal::Boolean(left)),
                        (Literal::Boolean(left), Literal::String(..)) => Ok(Literal::Boolean(left)),
                        (Literal::Nil, Literal::Nil) => Ok(Literal::Boolean(true)),
                        (_, _) => Ok(Literal::Boolean(false)),
                    },
                    Token::BangEqual { .. } => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Boolean(left != right))
                        }
                        (Literal::Number(..), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(!right))
                        }
                        (Literal::String(left), Literal::String(right)) => {
                            Ok(Literal::Boolean(left != right))
                        }
                        (Literal::String(..), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(!right))
                        }
                        (Literal::Boolean(left), Literal::Boolean(right)) => {
                            Ok(Literal::Boolean(left != right))
                        }
                        (Literal::Boolean(left), Literal::Number(..)) => {
                            Ok(Literal::Boolean(!left))
                        }
                        (Literal::Boolean(left), Literal::String(..)) => {
                            Ok(Literal::Boolean(!left))
                        }
                        (Literal::Nil, Literal::Nil) => Ok(Literal::Boolean(false)),
                        (_, _) => Ok(Literal::Boolean(true)),
                    },
                    token => {
                        self.error.report(
                            token.location(),
                            ErrorType::RuntimeError,
                            "Invalid operator.",
                        );
                        Err(())
                    }
                }
            }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Variable { name } => match name {
                Token::Identifier {
                    value,
                    line,
                    column,
                } => match self.environment.get(&value) {
                    Some(value) => Ok(value.clone()),
                    None => {
                        self.error.report(
                            (line, column),
                            ErrorType::RuntimeError,
                            &format!("Undefined variable '{}'", value),
                        );
                        Err(())
                    }
                },
                _ => unreachable!(),
            },
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                match name {
                    Token::Identifier {
                        value: name,
                        line,
                        column,
                    } => {
                        if let Some(_) = self.environment.get(&name) {
                            self.environment.bind(&name, value.clone());
                            Ok(value)
                        } else {
                            self.error.report(
                                (line, column),
                                ErrorType::RuntimeError,
                                &format!("Undefined variable '{}'", name),
                            );
                            Err(())
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator {
                    Token::Or { .. } => {
                        if left.is_truthy() {
                            Ok(left)
                        } else {
                            Ok(right)
                        }
                    }
                    Token::And { .. } => {
                        if left.is_truthy() {
                            Ok(right)
                        } else {
                            Ok(left)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;

                match callee {
                    Literal::Callable(callable) => {
                        let mut evaluated_arguments = Vec::new();
                        for argument in arguments {
                            evaluated_arguments.push(self.interpret(vec![argument.clone()])?);
                        }

                        let actual = evaluated_arguments.len();
                        let expected = callable.arity();

                        if actual != expected {
                            self.error.report(
                                paren.location(),
                                ErrorType::RuntimeError,
                                &format!("Expected {} arguments but got {}.", expected, actual),
                            );
                            Err(())
                        } else {
                            callable.call(self, evaluated_arguments)
                        }
                    }
                    _ => {
                        self.error.report(
                            paren.location(),
                            ErrorType::RuntimeError,
                            "Can only perform calls on functions and classes.",
                        );
                        Err(())
                    }
                }
            }
        }
    }
}
