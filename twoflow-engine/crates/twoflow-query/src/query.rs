use crate::arrow_bridge::record_batch_from_zig;
use crate::ffi::parse_2flow_buffer;
use crate::provider::TwoFlowTableProvider;
use datafusion::arrow::array::RecordBatch;
use datafusion::error::Result;
use datafusion::prelude::SessionContext;
use std::sync::Arc;

pub async fn register_2flow_buffer_as_table(
    ctx: &SessionContext,
    table_name: &str,
    input: &[u8],
) -> Result<Arc<TwoFlowTableProvider>> {
    let parsed = Arc::new(parse_2flow_buffer(input)?);
    let arrow_batch = record_batch_from_zig(parsed)?;
    let provider = Arc::new(TwoFlowTableProvider::new(
        arrow_batch.record_batch().clone(),
    ));
    ctx.register_table(table_name, provider.clone())?;
    Ok(provider)
}

pub async fn query_2flow_sql(input: &[u8], sql: &str) -> Result<Vec<RecordBatch>> {
    let ctx = SessionContext::new();
    register_2flow_buffer_as_table(&ctx, "graph", input).await?;
    let df = ctx.sql(sql).await?;
    df.collect().await
}
