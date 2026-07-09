use crate::ql_ast::{ProjectionItem, Query, Term};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    EmptyStore,
    NoPatterns,
    NoProjection,
    UnboundVariable(String),
    TriplesProjectionMixed,
    WildcardPathValueProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_query(query: &Query) -> Result<(), ValidationError> {
    if query.store.name.is_empty() {
        return Err(err(
            ValidationErrorKind::EmptyStore,
            "store cannot be empty",
        ));
    }
    if query.patterns.is_empty() {
        return Err(err(
            ValidationErrorKind::NoPatterns,
            "query must have a pattern",
        ));
    }
    if query.projection.is_empty() {
        return Err(err(
            ValidationErrorKind::NoProjection,
            "query must have a projection",
        ));
    }
    if query.projection.len() > 1
        && query
            .projection
            .iter()
            .any(|p| matches!(p, ProjectionItem::Triples))
    {
        return Err(err(
            ValidationErrorKind::TriplesProjectionMixed,
            "triples cannot be combined with other projections in the MVP",
        ));
    }

    let mut bound = BTreeSet::new();
    for pattern in &query.patterns {
        collect_term_vars(&pattern.start, &mut bound);
        for edge in &pattern.edges {
            collect_term_vars(&edge.predicate, &mut bound);
            if let Some(value) = &edge.value {
                collect_term_vars(value, &mut bound);
            }
        }
        if matches!(pattern.start, Term::Wildcard)
            && pattern.edges.len() > 1
            && query
                .projection
                .iter()
                .any(|p| matches!(p, ProjectionItem::Value))
        {
            return Err(err(
                ValidationErrorKind::WildcardPathValueProjection,
                "wildcard path cannot project value without a concrete start in the MVP",
            ));
        }
    }

    for item in &query.projection {
        if let ProjectionItem::Variable(name) = item {
            if !bound.contains(name) {
                return Err(err(
                    ValidationErrorKind::UnboundVariable(name.clone()),
                    format!("Unbound variable ${name} in projection."),
                ));
            }
        }
    }
    Ok(())
}

fn collect_term_vars(term: &Term, out: &mut BTreeSet<String>) {
    if let Term::Variable(name) = term {
        out.insert(name.clone());
    }
}

fn err(kind: ValidationErrorKind, message: impl Into<String>) -> ValidationError {
    ValidationError {
        kind,
        message: message.into(),
    }
}
