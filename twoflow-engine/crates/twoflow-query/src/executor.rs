use crate::ql_ast::{Pattern, ProjectionItem, Query, Term};
use crate::validator::{ValidationError, validate_query};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub value: String,
    pub line: usize,
    pub pair_index: usize,
}

pub type QueryRow = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub rows: Vec<QueryRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryExecutionError {
    Validation(String),
}

impl std::fmt::Display for QueryExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryExecutionError::Validation(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for QueryExecutionError {}

impl From<ValidationError> for QueryExecutionError {
    fn from(value: ValidationError) -> Self {
        QueryExecutionError::Validation(value.to_string())
    }
}

#[derive(Debug, Clone, Default)]
struct BindingState {
    vars: BTreeMap<String, String>,
    last: Option<Triple>,
}

pub async fn execute_2flowql_in_memory(
    triples: &[Triple],
    query: &Query,
) -> Result<QueryResult, QueryExecutionError> {
    validate_query(query)?;
    let mut states = vec![BindingState::default()];
    for pattern in &query.patterns {
        let mut next = Vec::new();
        for state in &states {
            next.extend(match_pattern(triples, pattern, state));
        }
        states = next;
    }

    let mut rows = Vec::new();
    for state in states {
        if matches!(query.projection.as_slice(), [ProjectionItem::Triples]) {
            if let Some(last) = &state.last {
                rows.push(triple_row(last));
            }
            continue;
        }
        let mut row = QueryRow::new();
        for item in &query.projection {
            project_item(item, &state, &mut row);
        }
        rows.push(row);
    }
    Ok(QueryResult { rows })
}

fn match_pattern(triples: &[Triple], pattern: &Pattern, input: &BindingState) -> Vec<BindingState> {
    let mut states = vec![input.clone()];
    let mut current_subject_term = Some(pattern.start.clone());

    for edge in &pattern.edges {
        let mut next = Vec::new();
        for state in &states {
            for triple in triples {
                let mut candidate = state.clone();
                let subject_ok = if let Some(term) = &current_subject_term {
                    match_term(term, &triple.subject, &mut candidate)
                } else if let Some(previous) = &state.last {
                    previous.value == triple.subject
                } else {
                    false
                };
                if !subject_ok {
                    continue;
                }
                if !match_term(&edge.predicate, &triple.predicate, &mut candidate) {
                    continue;
                }
                if let Some(value_term) = &edge.value {
                    if !match_term(value_term, &triple.value, &mut candidate) {
                        continue;
                    }
                }
                candidate.last = Some(triple.clone());
                next.push(candidate);
            }
        }
        states = next;
        current_subject_term = None;
    }

    states
}

fn match_term(term: &Term, actual: &str, state: &mut BindingState) -> bool {
    match term {
        Term::Wildcard => true,
        Term::Variable(name) => match state.vars.get(name) {
            Some(existing) => existing == actual,
            None => {
                state.vars.insert(name.clone(), actual.to_string());
                true
            }
        },
        Term::Identifier(expected)
        | Term::StringLiteral(expected)
        | Term::NumberLiteral(expected) => expected == actual,
        Term::BooleanLiteral(expected) => expected.to_string() == actual,
    }
}

fn project_item(item: &ProjectionItem, state: &BindingState, row: &mut QueryRow) {
    match item {
        ProjectionItem::Subject => {
            if let Some(last) = &state.last {
                row.insert("subject".to_string(), last.subject.clone());
            }
        }
        ProjectionItem::Predicate => {
            if let Some(last) = &state.last {
                row.insert("predicate".to_string(), last.predicate.clone());
            }
        }
        ProjectionItem::Value => {
            if let Some(last) = &state.last {
                row.insert("value".to_string(), last.value.clone());
            }
        }
        ProjectionItem::Triple | ProjectionItem::Triples => {
            if let Some(last) = &state.last {
                for (key, value) in triple_row(last) {
                    row.insert(key, value);
                }
            }
        }
        ProjectionItem::Variable(name) => {
            if let Some(value) = state.vars.get(name) {
                row.insert(name.clone(), value.clone());
            }
        }
    }
}

fn triple_row(triple: &Triple) -> QueryRow {
    QueryRow::from([
        ("subject".to_string(), triple.subject.clone()),
        ("predicate".to_string(), triple.predicate.clone()),
        ("value".to_string(), triple.value.clone()),
    ])
}
