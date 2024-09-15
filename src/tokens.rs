// TODO: eventually remove debug

#[derive(Debug, Clone)]
pub enum Token {
    // Single-character tokens
    LeftParen {
        line: usize,
        column: usize,
    },
    RightParen {
        line: usize,
        column: usize,
    },
    LeftBrace {
        line: usize,
        column: usize,
    },
    RightBrace {
        line: usize,
        column: usize,
    },
    Comma {
        line: usize,
        column: usize,
    },
    Dot {
        line: usize,
        column: usize,
    },
    Minus {
        line: usize,
        column: usize,
    },
    Plus {
        line: usize,
        column: usize,
    },
    Semicolon {
        line: usize,
        column: usize,
    },
    Slash {
        line: usize,
        column: usize,
    },
    Star {
        line: usize,
        column: usize,
    },
    Question {
        line: usize,
        column: usize,
    },
    Colon {
        line: usize,
        column: usize,
    },

    // One or two character tokens
    Bang {
        line: usize,
        column: usize,
    },
    BangEqual {
        line: usize,
        column: usize,
    },
    Equal {
        line: usize,
        column: usize,
    },
    EqualEqual {
        line: usize,
        column: usize,
    },
    Greater {
        line: usize,
        column: usize,
    },
    GreaterEqual {
        line: usize,
        column: usize,
    },
    Less {
        line: usize,
        column: usize,
    },
    LessEqual {
        line: usize,
        column: usize,
    },
    PlusEqual {
        line: usize,
        column: usize,
    },
    MinusEqual {
        line: usize,
        column: usize,
    },
    StarEqual {
        line: usize,
        column: usize,
    },

    // Literals
    Identifier {
        value: String,
        line: usize,
        column: usize,
    },
    String {
        value: String,
        line: usize,
        column: usize,
    },
    Number {
        value: f64,
        line: usize,
        column: usize,
    },

    // Keywords
    And {
        line: usize,
        column: usize,
    },
    Class {
        line: usize,
        column: usize,
    },
    Else {
        line: usize,
        column: usize,
    },
    False {
        line: usize,
        column: usize,
    },
    Fun {
        line: usize,
        column: usize,
    },
    For {
        line: usize,
        column: usize,
    },
    If {
        line: usize,
        column: usize,
    },
    Nil {
        line: usize,
        column: usize,
    },
    Or {
        line: usize,
        column: usize,
    },
    Print {
        line: usize,
        column: usize,
    },
    Return {
        line: usize,
        column: usize,
    },
    Break {
        line: usize,
        column: usize,
    },
    Super {
        line: usize,
        column: usize,
    },
    This {
        line: usize,
        column: usize,
    },
    True {
        line: usize,
        column: usize,
    },
    Var {
        line: usize,
        column: usize,
    },
    While {
        line: usize,
        column: usize,
    },

    // End of file
    Eof {
        line: usize,
        column: usize,
    },
}

impl Token {
    pub fn location(&self) -> (&usize, &usize) {
        match self {
            Token::LeftParen { line, column } => (line, column),
            Token::RightParen { line, column } => (line, column),
            Token::LeftBrace { line, column } => (line, column),
            Token::RightBrace { line, column } => (line, column),
            Token::Comma { line, column } => (line, column),
            Token::Dot { line, column } => (line, column),
            Token::Minus { line, column } => (line, column),
            Token::Plus { line, column } => (line, column),
            Token::Semicolon { line, column } => (line, column),
            Token::Slash { line, column } => (line, column),
            Token::Star { line, column } => (line, column),
            Token::Bang { line, column } => (line, column),
            Token::BangEqual { line, column } => (line, column),
            Token::Equal { line, column } => (line, column),
            Token::EqualEqual { line, column } => (line, column),
            Token::Greater { line, column } => (line, column),
            Token::GreaterEqual { line, column } => (line, column),
            Token::Less { line, column } => (line, column),
            Token::LessEqual { line, column } => (line, column),
            Token::PlusEqual { line, column } => (line, column),
            Token::MinusEqual { line, column } => (line, column),
            Token::StarEqual { line, column } => (line, column),
            Token::Identifier { line, column, .. } => (line, column),
            Token::String { line, column, .. } => (line, column),
            Token::Number { line, column, .. } => (line, column),
            Token::And { line, column } => (line, column),
            Token::Class { line, column } => (line, column),
            Token::Else { line, column } => (line, column),
            Token::False { line, column } => (line, column),
            Token::Fun { line, column } => (line, column),
            Token::For { line, column } => (line, column),
            Token::If { line, column } => (line, column),
            Token::Nil { line, column } => (line, column),
            Token::Or { line, column } => (line, column),
            Token::Print { line, column } => (line, column),
            Token::Return { line, column } => (line, column),
            Token::Break { line, column } => (line, column),
            Token::Super { line, column } => (line, column),
            Token::This { line, column } => (line, column),
            Token::True { line, column } => (line, column),
            Token::Var { line, column } => (line, column),
            Token::While { line, column } => (line, column),
            Token::Question { line, column } => (line, column),
            Token::Colon { line, column } => (line, column),
            Token::Eof { line, column } => (line, column),
        }
    }
}
