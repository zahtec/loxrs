use crate::{error::Error, tokens::Token};

pub struct Scanner<'src> {
    source: String,
    error: &'src Error,
    had_error: bool,
    start: usize,
    current: usize,
    column: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Scanner<'_> {
    pub fn new<'src>(error: &'src Error) -> Scanner<'src> {
        Scanner {
            source: String::new(),
            error: error,
            had_error: false,
            start: 0,
            current: 0,
            column: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    fn error(&mut self, message: &str) {
        self.error.report(
            (&self.line, &self.column),
            crate::error::ErrorType::TokenError,
            message,
        );
        self.had_error = true;
    }

    fn is_end(&mut self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self, source: String) -> Result<Vec<Token>, ()> {
        self.current = 0;
        self.start = 0;
        self.column = 0;
        self.line = 1;
        self.tokens = Vec::new();
        self.had_error = false;

        self.source = source;

        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::Eof {
            line: self.line,
            column: self.column,
        });

        if self.had_error {
            Err(())
        } else {
            Ok(self.tokens.to_owned())
        }
    }

    fn increment_current(&mut self) {
        self.current += 1;
        self.column += 1;
    }

    fn increment_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    fn scan_token(&mut self) {
        let scan = self.source.get(self.current..self.current + 1).unwrap();

        self.current += 1;
        self.column += 1;

        match scan {
            "(" => self.tokens.push(Token::LeftParen {
                line: self.line,
                column: self.column,
            }),
            ")" => self.tokens.push(Token::RightParen {
                line: self.line,
                column: self.column,
            }),
            "{" => self.tokens.push(Token::LeftBrace {
                line: self.line,
                column: self.column,
            }),
            "}" => self.tokens.push(Token::RightBrace {
                line: self.line,
                column: self.column,
            }),
            "," => self.tokens.push(Token::Comma {
                line: self.line,
                column: self.column,
            }),
            "." => self.tokens.push(Token::Dot {
                line: self.line,
                column: self.column,
            }),
            ";" => self.tokens.push(Token::Semicolon {
                line: self.line,
                column: self.column,
            }),
            "?" => self.tokens.push(Token::Question {
                line: self.line,
                column: self.column,
            }),
            ":" => self.tokens.push(Token::Colon {
                line: self.line,
                column: self.column,
            }),

            "-" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::MinusEqual {
                        line: self.line,
                        column: self.column,
                    });
                    self.increment_current();
                } else {
                    self.tokens.push(Token::Minus {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "+" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::PlusEqual {
                        line: self.line,
                        column: self.column,
                    });
                    self.increment_current();
                } else {
                    self.tokens.push(Token::Plus {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "*" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::StarEqual {
                        line: self.line,
                        column: self.column,
                    });
                    self.increment_current();
                } else {
                    self.tokens.push(Token::Star {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "!" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::BangEqual {
                        line: self.line,
                        column: self.column,
                    });
                } else {
                    self.tokens.push(Token::Bang {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "=" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::EqualEqual {
                        line: self.line,
                        column: self.column,
                    });
                } else {
                    self.tokens.push(Token::Equal {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "<" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::LessEqual {
                        line: self.line,
                        column: self.column,
                    });
                } else {
                    self.tokens.push(Token::Less {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            ">" => {
                if self.look_ahead('=') {
                    self.tokens.push(Token::GreaterEqual {
                        line: self.line,
                        column: self.column,
                    });
                } else {
                    self.tokens.push(Token::Greater {
                        line: self.line,
                        column: self.column,
                    });
                }
            }
            "/" => {
                if self.look_ahead('/') {
                    while self.peek(1) != "\n" && !self.is_end() {
                        self.increment_current();
                    }
                } else {
                    self.tokens.push(Token::Slash {
                        line: self.line,
                        column: self.column,
                    });
                }
            }

            "\"" => self.scan_string("\""),
            "'" => self.scan_string("'"),

            "\r" | "\t" => (),
            "\n" => self.increment_line(),
            "   " | " " => self.column -= 1,

            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => self.scan_number(),

            ident @ _ if ident.chars().next().unwrap().is_alphanumeric() => self.scan_identifier(),

            c => self.error(&format!("Unexpected character: {}", c)),
        }
    }

    fn look_ahead(&mut self, char: char) -> bool {
        if self.source.chars().nth(self.current).unwrap() == char {
            self.increment_current();
            true
        } else {
            false
        }
    }

    fn peek(&mut self, amount: usize) -> &str {
        if self.current + amount >= self.source.len() {
            "\0"
        } else {
            self.source
                .get(self.current..self.current + amount)
                .unwrap()
        }
    }

    fn scan_string(&mut self, specific: &str) {
        while self.peek(1) != specific && !self.is_end() {
            if self.peek(1) == "\n" {
                self.increment_line();
            }

            self.increment_current();
        }

        self.increment_current();

        if self
            .source
            .get(self.current..self.current + 1)
            .unwrap_or("\0")
            != specific
            && self.is_end()
        {
            self.error(&format!("Unterminated string. Expected: {}", specific));
        } else {
            self.tokens.push(Token::String {
                value: self
                    .source
                    .get(self.start + 1..self.current - 1)
                    .unwrap()
                    .to_owned(),
                line: self.line,
                column: self.column,
            });
        }
    }

    fn scan_number(&mut self) {
        while self.peek(1).chars().next().unwrap().is_digit(10) {
            self.increment_current();
        }

        if self.peek(1) == "." && self.peek(2).chars().nth(1).unwrap().is_digit(10) {
            self.increment_current();
            while self.peek(1).chars().next().unwrap().is_digit(10) {
                self.increment_current();
            }
        }

        self.tokens.push(Token::Number {
            value: self
                .source
                .get(self.start..self.current)
                .unwrap()
                .parse::<f64>()
                .unwrap(),
            line: self.line,
            column: self.column,
        });
    }

    fn scan_identifier(&mut self) {
        while self.peek(1).chars().next().unwrap().is_alphanumeric() || self.peek(1) == "_" {
            self.increment_current();
        }

        let ident = self.source.get(self.start..self.current).unwrap();

        match ident {
            "and" => self.tokens.push(Token::And {
                line: self.line,
                column: self.column,
            }),
            "class" => self.tokens.push(Token::Class {
                line: self.line,
                column: self.column,
            }),
            "else" => self.tokens.push(Token::Else {
                line: self.line,
                column: self.column,
            }),
            "false" => self.tokens.push(Token::False {
                line: self.line,
                column: self.column,
            }),
            "fun" => self.tokens.push(Token::Fun {
                line: self.line,
                column: self.column,
            }),
            "for" => self.tokens.push(Token::For {
                line: self.line,
                column: self.column,
            }),
            "if" => self.tokens.push(Token::If {
                line: self.line,
                column: self.column,
            }),
            "nil" => self.tokens.push(Token::Nil {
                line: self.line,
                column: self.column,
            }),
            "or" => self.tokens.push(Token::Or {
                line: self.line,
                column: self.column,
            }),
            "print" => self.tokens.push(Token::Print {
                line: self.line,
                column: self.column,
            }),
            "return" => self.tokens.push(Token::Return {
                line: self.line,
                column: self.column,
            }),
            "break" => self.tokens.push(Token::Break {
                line: self.line,
                column: self.column,
            }),
            "super" => self.tokens.push(Token::Super {
                line: self.line,
                column: self.column,
            }),
            "this" => self.tokens.push(Token::This {
                line: self.line,
                column: self.column,
            }),
            "true" => self.tokens.push(Token::True {
                line: self.line,
                column: self.column,
            }),
            "var" => self.tokens.push(Token::Var {
                line: self.line,
                column: self.column,
            }),
            "while" => self.tokens.push(Token::While {
                line: self.line,
                column: self.column,
            }),
            _ => self.tokens.push(Token::Identifier {
                value: ident.to_owned(),
                line: self.line,
                column: self.column - (ident.len() - 1),
            }),
        }
    }
}
