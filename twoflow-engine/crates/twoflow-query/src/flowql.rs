use crate::executor::{QueryExecutionError, QueryResult, Triple, execute_2flowql_in_memory};
use crate::json_result::query_result_to_json;
use crate::ql_ast::Query;
use crate::ql_parser::{QueryParseError, parse_2flowql};
use crate::twoflow_triples::parse_twoflow_triples;
use crate::validator::{ValidationError, validate_query};
use datafusion::error::{DataFusionError, Result};

pub fn parse_query(input: &str) -> std::result::Result<Query, QueryParseError> {
    parse_2flowql(input)
}

pub async fn execute_query_on_triples(triples: &[Triple], query: &str) -> Result<QueryResult> {
    let parsed = parse_2flowql(query).map_err(parse_err)?;
    validate_query(&parsed).map_err(validation_err)?;
    execute_2flowql_in_memory(triples, &parsed)
        .await
        .map_err(exec_err)
}

pub async fn query_2flowql_json(twoflow_input: &[u8], query: &str) -> Result<String> {
    let triples = parse_twoflow_triples(twoflow_input)?;
    let result = execute_query_on_triples(&triples, query).await?;
    query_result_to_json(&result)
}

fn parse_err(err: QueryParseError) -> DataFusionError {
    DataFusionError::Execution(format!("2FlowQL parse error: {err}"))
}

fn validation_err(err: ValidationError) -> DataFusionError {
    DataFusionError::Execution(format!("2FlowQL validation error: {err}"))
}

fn exec_err(err: QueryExecutionError) -> DataFusionError {
    DataFusionError::Execution(format!("2FlowQL execution error: {err}"))
}
