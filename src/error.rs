#[derive(Debug)]
pub enum ErrorType {
    TokenError,
    ParserError,
    ResolverError,
    RuntimeError,
}

#[derive(Debug)]
pub struct Error {
    file: String,
    lines: Option<Vec<String>>,
}

impl Error {
    pub fn new(file: &str, source: Option<String>) -> Error {
        Error {
            file: file.to_owned(),
            lines: source.map(|s| s.lines().map(|l| l.to_owned()).collect()),
        }
    }

    pub fn report(&self, (line, column): (&usize, &usize), typ: ErrorType, message: &str) {
        if let Some(lines) = &self.lines {
            println!("{}", lines[*line - 1].trim());
        }

        println!(
            "{}^ -- Here",
            " ".repeat(column + self.lines.is_none() as usize)
        );

        println!("{} @ Line {line} - {typ:?}: {message}", &self.file);
    }
}
