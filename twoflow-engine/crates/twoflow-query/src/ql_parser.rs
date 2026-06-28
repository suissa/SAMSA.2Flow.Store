use crate::lexer::{LexError, Span, Token, TokenKind, lex_2flowql};
use crate::ql_ast::{Edge, Pattern, ProjectionItem, Query, StoreRef, Term};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParseErrorKind {
    UnexpectedToken,
    UnexpectedEof,
    ExpectedStore,
    ExpectedQueryMarker,
    ExpectedPattern,
    ExpectedArrow,
    ExpectedProjection,
    ExpectedTerm,
    ExpectedSemicolon,
    UnterminatedString,
    InvalidCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryParseError {
    pub kind: QueryParseErrorKind,
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}:{}: {}",
            self.kind, self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for QueryParseError {}

impl From<LexError> for QueryParseError {
    fn from(value: LexError) -> Self {
        let kind = match value.kind {
            crate::lexer::LexErrorKind::UnterminatedString => {
                QueryParseErrorKind::UnterminatedString
            }
            crate::lexer::LexErrorKind::InvalidCharacter(_) => {
                QueryParseErrorKind::InvalidCharacter
            }
        };
        Self {
            kind,
            span: value.span,
            message: "lexer failed".to_string(),
        }
    }
}

pub fn parse_2flowql(input: &str) -> Result<Query, QueryParseError> {
    let tokens = lex_2flowql(input)?;
    Parser { tokens, pos: 0 }.parse_query()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn parse_query(&mut self) -> Result<Query, QueryParseError> {
        self.expect_symbol(
            |k| matches!(k, TokenKind::At),
            QueryParseErrorKind::ExpectedStore,
        )?;
        let store = match self.bump().kind.clone() {
            TokenKind::Identifier(name) => StoreRef { name },
            _ => {
                return Err(
                    self.error_here(QueryParseErrorKind::ExpectedStore, "expected store name")
                );
            }
        };
        self.expect_symbol(
            |k| matches!(k, TokenKind::Question),
            QueryParseErrorKind::ExpectedQueryMarker,
        )?;
        let mut patterns = Vec::new();
        while !self.at_fat_arrow() {
            if self.at_eof() {
                return Err(self.error_here(
                    QueryParseErrorKind::ExpectedProjection,
                    "expected => projection",
                ));
            }
            patterns.push(self.parse_pattern()?);
        }
        if patterns.is_empty() {
            return Err(self.error_here(
                QueryParseErrorKind::ExpectedPattern,
                "expected at least one pattern",
            ));
        }
        self.expect_symbol(
            |k| matches!(k, TokenKind::FatArrow),
            QueryParseErrorKind::ExpectedProjection,
        )?;
        let projection = self.parse_projection_list()?;
        self.expect_symbol(
            |k| matches!(k, TokenKind::Semicolon),
            QueryParseErrorKind::ExpectedSemicolon,
        )?;
        if !self.at_eof() {
            return Err(self.error_here(
                QueryParseErrorKind::UnexpectedToken,
                "unexpected token after ;",
            ));
        }
        Ok(Query {
            store,
            patterns,
            projection,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, QueryParseError> {
        let start = self.parse_term()?;
        let mut edges = Vec::new();
        loop {
            if !self.at_arrow() {
                if edges.is_empty() {
                    return Err(self.error_here(
                        QueryParseErrorKind::ExpectedArrow,
                        "expected -> after pattern start",
                    ));
                }
                break;
            }
            self.bump();
            let predicate = self.parse_term()?;
            let value = if self.at_colon() {
                self.bump();
                Some(self.parse_term()?)
            } else {
                None
            };
            edges.push(Edge { predicate, value });
            if self.at_fat_arrow() || self.at_eof() || self.starts_new_pattern() {
                break;
            }
        }
        Ok(Pattern { start, edges })
    }

    fn starts_new_pattern(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Identifier(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::NumberLiteral(_)
                | TokenKind::BooleanLiteral(_)
                | TokenKind::Dollar
                | TokenKind::Star
        ) && !self.at_arrow()
    }

    fn parse_projection_list(&mut self) -> Result<Vec<ProjectionItem>, QueryParseError> {
        let mut items = Vec::new();
        loop {
            items.push(self.parse_projection_item()?);
            if self.at_comma() {
                self.bump();
                continue;
            }
            break;
        }
        if items.is_empty() {
            return Err(self.error_here(
                QueryParseErrorKind::ExpectedProjection,
                "expected projection item",
            ));
        }
        Ok(items)
    }

    fn parse_projection_item(&mut self) -> Result<ProjectionItem, QueryParseError> {
        if self.at_dollar() {
            self.bump();
            return match self.bump().kind.clone() {
                TokenKind::Identifier(name) => Ok(ProjectionItem::Variable(name)),
                _ => Err(self.error_here(
                    QueryParseErrorKind::ExpectedProjection,
                    "expected variable name",
                )),
            };
        }
        match self.bump().kind.clone() {
            TokenKind::Identifier(name) => match name.as_str() {
                "subject" => Ok(ProjectionItem::Subject),
                "predicate" => Ok(ProjectionItem::Predicate),
                "value" => Ok(ProjectionItem::Value),
                "triple" => Ok(ProjectionItem::Triple),
                "triples" => Ok(ProjectionItem::Triples),
                _ => Err(self.error_here(
                    QueryParseErrorKind::ExpectedProjection,
                    "unknown projection item",
                )),
            },
            _ => Err(self.error_here(
                QueryParseErrorKind::ExpectedProjection,
                "expected projection item",
            )),
        }
    }

    fn parse_term(&mut self) -> Result<Term, QueryParseError> {
        if self.at_dollar() {
            self.bump();
            return match self.bump().kind.clone() {
                TokenKind::Identifier(name) => Ok(Term::Variable(name)),
                _ => {
                    Err(self
                        .error_here(QueryParseErrorKind::ExpectedTerm, "expected variable name"))
                }
            };
        }
        match self.bump().kind.clone() {
            TokenKind::Identifier(value) => Ok(Term::Identifier(value)),
            TokenKind::StringLiteral(value) => Ok(Term::StringLiteral(value)),
            TokenKind::NumberLiteral(value) => Ok(Term::NumberLiteral(value)),
            TokenKind::BooleanLiteral(value) => Ok(Term::BooleanLiteral(value)),
            TokenKind::Star => Ok(Term::Wildcard),
            TokenKind::Eof => {
                Err(self.error_here(QueryParseErrorKind::UnexpectedEof, "unexpected eof"))
            }
            _ => Err(self.error_here(QueryParseErrorKind::ExpectedTerm, "expected term")),
        }
    }

    fn expect_symbol(
        &mut self,
        f: impl FnOnce(&TokenKind) -> bool,
        kind: QueryParseErrorKind,
    ) -> Result<(), QueryParseError> {
        if f(&self.peek().kind) {
            self.bump();
            Ok(())
        } else {
            Err(self.error_here(kind, "unexpected token"))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn bump(&mut self) -> &Token {
        let pos = self.pos;
        if !self.at_eof() {
            self.pos += 1;
        }
        &self.tokens[pos]
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
    fn at_arrow(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Arrow)
    }
    fn at_fat_arrow(&self) -> bool {
        matches!(self.peek().kind, TokenKind::FatArrow)
    }
    fn at_colon(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Colon)
    }
    fn at_comma(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Comma)
    }
    fn at_dollar(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Dollar)
    }

    fn error_here(&self, kind: QueryParseErrorKind, message: impl Into<String>) -> QueryParseError {
        QueryParseError {
            kind,
            span: self.peek().span.clone(),
            message: message.into(),
        }
    }
}
