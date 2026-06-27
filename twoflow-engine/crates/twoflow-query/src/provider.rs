use async_trait::async_trait;
use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::Session;
use datafusion::common::Result;
use datafusion::datasource::{TableProvider, TableType};
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_plan::memory::MemoryExec;
use std::any::Any;
use std::sync::Arc;

pub struct TwoFlowTableProvider {
    schema: SchemaRef,
    batch: RecordBatch,
}

impl TwoFlowTableProvider {
    pub fn new(batch: RecordBatch) -> Self {
        Self {
            schema: batch.schema(),
            batch,
        }
    }

    pub fn batch(&self) -> &RecordBatch {
        &self.batch
    }
}

#[async_trait]
impl TableProvider for TwoFlowTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            // DataFusion will apply filters above this provider because we mark
            // pushdown unsupported below.
        }
        let projected_batch = if let Some(indices) = projection {
            self.batch.project(indices)?
        } else {
            self.batch.clone()
        };
        let projected_schema = projected_batch.schema();
        let batches = match limit {
            Some(limit) => vec![projected_batch.slice(0, limit.min(projected_batch.num_rows()))],
            None => vec![projected_batch],
        };
        Ok(Arc::new(MemoryExec::try_new(
            &[batches],
            projected_schema,
            None,
        )?))
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        Ok(filters
            .iter()
            .map(|_| TableProviderFilterPushDown::Unsupported)
            .collect())
    }
}
