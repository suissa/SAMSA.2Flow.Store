#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub store: StoreRef,
    pub patterns: Vec<Pattern>,
    pub projection: Vec<ProjectionItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoreRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub start: Term,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub predicate: Term,
    pub value: Option<Term>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Identifier(String),
    Variable(String),
    Wildcard,
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionItem {
    Subject,
    Predicate,
    Value,
    Triple,
    Triples,
    Variable(String),
}

impl Term {
    pub fn variable_name(&self) -> Option<&str> {
        match self {
            Term::Variable(name) => Some(name),
            _ => None,
        }
    }

    pub fn literal_text(&self) -> Option<String> {
        match self {
            Term::Identifier(value) | Term::StringLiteral(value) | Term::NumberLiteral(value) => {
                Some(value.clone())
            }
            Term::BooleanLiteral(value) => Some(value.to_string()),
            Term::Wildcard | Term::Variable(_) => None,
        }
    }
}
