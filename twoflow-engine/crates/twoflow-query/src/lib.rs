pub mod arrow_bridge;
pub mod executor;
pub mod ffi;
pub mod flowql;
pub mod json;
pub mod json_result;
pub mod lexer;
pub mod provider;
pub mod ql_ast;
pub mod ql_parser;
pub mod query;
pub mod twoflow_triples;
pub mod validator;

pub use arrow_bridge::{ArrowBridgeMode, TwoFlowArrowBatch, record_batch_from_zig};
pub use ffi::{ParseStats, TwoFlowParsedBatch, parse_2flow_buffer};
pub use provider::TwoFlowTableProvider;
pub use query::{query_2flow_sql, register_2flow_buffer_as_table};

#[cfg(test)]
mod flowql_tests;
#[cfg(test)]
mod tests;

pub use executor::{QueryResult, QueryRow, Triple, execute_2flowql_in_memory};
pub use flowql::{execute_query_on_triples, parse_query, query_2flowql_json};
pub use lexer::{Span, Token, TokenKind, lex_2flowql};
pub use ql_ast::{Edge, Pattern, ProjectionItem, Query, StoreRef, Term};
pub use ql_parser::{QueryParseError, QueryParseErrorKind, parse_2flowql};
pub use validator::{ValidationError, ValidationErrorKind, validate_query};
