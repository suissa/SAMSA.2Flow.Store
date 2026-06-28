#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    At,
    Question,
    Arrow,
    Colon,
    FatArrow,
    Comma,
    Semicolon,
    Dollar,
    Star,
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexErrorKind {
    UnterminatedString,
    InvalidCharacter(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

pub fn lex_2flowql(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token()?;
        let done = matches!(token.kind, TokenKind::Eof);
        tokens.push(token);
        if done {
            return Ok(tokens);
        }
    }
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    fn span_from(&self, start: usize, line: usize, column: usize) -> Span {
        Span {
            start,
            end: self.offset,
            line,
            column,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.input[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.bump();
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_ws();
        let start = self.offset;
        let line = self.line;
        let column = self.column;
        let Some(ch) = self.peek() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: self.span_from(start, line, column),
            });
        };

        let kind = match ch {
            '@' => {
                self.bump();
                TokenKind::At
            }
            '?' => {
                self.bump();
                TokenKind::Question
            }
            ':' => {
                self.bump();
                TokenKind::Colon
            }
            ',' => {
                self.bump();
                TokenKind::Comma
            }
            ';' => {
                self.bump();
                TokenKind::Semicolon
            }
            '$' => {
                self.bump();
                TokenKind::Dollar
            }
            '*' => {
                self.bump();
                TokenKind::Star
            }
            '-' if self.peek_next() == Some('>') => {
                self.bump();
                self.bump();
                TokenKind::Arrow
            }
            '=' if self.peek_next() == Some('>') => {
                self.bump();
                self.bump();
                TokenKind::FatArrow
            }
            '"' | '\'' => self.string_literal(ch, start, line, column)?,
            ch if ch.is_ascii_digit()
                || ((ch == '-' || ch == '+')
                    && self.peek_next().is_some_and(|n| n.is_ascii_digit())) =>
            {
                self.number_or_identifier()
            }
            ch if is_identifier_start(ch) => self.identifier_or_bool(),
            other => {
                self.bump();
                return Err(LexError {
                    kind: LexErrorKind::InvalidCharacter(other),
                    span: self.span_from(start, line, column),
                });
            }
        };

        Ok(Token {
            kind,
            span: self.span_from(start, line, column),
        })
    }

    fn string_literal(
        &mut self,
        quote: char,
        start: usize,
        line: usize,
        column: usize,
    ) -> Result<TokenKind, LexError> {
        self.bump();
        let mut value = String::new();
        while let Some(ch) = self.bump() {
            if ch == quote {
                return Ok(TokenKind::StringLiteral(value));
            }
            if ch == '\\' {
                let Some(escaped) = self.bump() else {
                    break;
                };
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                });
            } else {
                value.push(ch);
            }
        }
        Err(LexError {
            kind: LexErrorKind::UnterminatedString,
            span: Span {
                start,
                end: self.offset,
                line,
                column,
            },
        })
    }

    fn number_or_identifier(&mut self) -> TokenKind {
        let start = self.offset;
        if matches!(self.peek(), Some('-' | '+')) {
            self.bump();
        }
        while matches!(self.peek(), Some(ch) if is_identifier_char(ch)) {
            self.bump();
        }
        let text = &self.input[start..self.offset];
        if is_number_literal_text(text) {
            TokenKind::NumberLiteral(text.to_string())
        } else {
            TokenKind::Identifier(text.to_string())
        }
    }

    fn identifier_or_bool(&mut self) -> TokenKind {
        let start = self.offset;
        while matches!(self.peek(), Some(ch) if is_identifier_char(ch)) {
            self.bump();
        }
        let text = &self.input[start..self.offset];
        match text {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            _ => TokenKind::Identifier(text.to_string()),
        }
    }
}

fn is_identifier_start(ch: char) -> bool {
    !matches!(ch, '@' | '?' | '$' | '*' | ':' | ',' | ';' | '"' | '\'') && is_identifier_char(ch)
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '@')
}

fn is_number_literal_text(text: &str) -> bool {
    let body = text.strip_prefix(['-', '+']).unwrap_or(text);
    if body.is_empty() {
        return false;
    }
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut digits_after_dot = 0usize;
    for ch in body.chars() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            if seen_dot {
                digits_after_dot += 1;
            }
        } else if ch == '.' && !seen_dot {
            seen_dot = true;
        } else {
            return false;
        }
    }
    seen_digit && (!seen_dot || digits_after_dot > 0)
}
