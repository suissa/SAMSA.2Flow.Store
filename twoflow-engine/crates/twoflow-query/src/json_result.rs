use crate::executor::QueryResult;
use datafusion::error::{DataFusionError, Result};

pub fn query_result_to_json(result: &QueryResult) -> Result<String> {
    serde_json::to_string(&result.rows).map_err(|err| {
        DataFusionError::Execution(format!("failed to serialize 2FlowQL result: {err}"))
    })
}
