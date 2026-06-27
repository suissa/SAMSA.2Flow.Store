use crate::arrow_bridge::{ArrowBridgeMode, record_batch_from_zig};
use crate::ffi::parse_2flow_buffer;
use crate::query::query_2flow_sql;
use datafusion::arrow::array::{Array, StringArray};
use datafusion::arrow::datatypes::DataType;
use std::sync::Arc;

const INPUT: &[u8] = b"Torre_Eiffel -> altura:330\nTorre_Eiffel -> localizacao:Paris\nParis -> pais:Franca\nUsuario_123 -> intent:ComprarPizza\n";

#[test]
fn ffi_returns_non_null_and_rows() {
    let parsed = parse_2flow_buffer(INPUT).expect("parse should succeed");
    assert_eq!(parsed.as_raw().rows, 4);
    assert_eq!(parsed.stats().valid_lines, 4);
}

#[test]
fn arrow_conversion_schema_and_columns() {
    let parsed = Arc::new(parse_2flow_buffer(INPUT).expect("parse should succeed"));
    let batch = record_batch_from_zig(parsed).expect("arrow conversion should succeed");
    assert_eq!(batch.mode(), ArrowBridgeMode::CopyingFallback);
    let record_batch = batch.record_batch();
    assert_eq!(record_batch.num_columns(), 3);
    assert_eq!(record_batch.schema().field(0).name(), "subject");
    assert_eq!(record_batch.schema().field(1).name(), "predicate");
    assert_eq!(record_batch.schema().field(2).name(), "object");
    assert_eq!(record_batch.schema().field(0).data_type(), &DataType::Utf8);
}

#[test]
fn drop_releases_zig_batch_without_crash() {
    let parsed = parse_2flow_buffer(INPUT).expect("parse should succeed");
    drop(parsed);
}

#[test]
fn invalid_partial_input_keeps_valid_lines_and_counts_invalid() {
    let input = b"bad line\nTorre_Eiffel -> altura:330\nno_colon -> altura 330\n";
    let parsed = parse_2flow_buffer(input).expect("parse should succeed");
    assert_eq!(parsed.stats().valid_lines, 1);
    assert_eq!(parsed.stats().invalid_lines, 2);
    assert_eq!(parsed.as_raw().rows, 1);
}

#[tokio::test]
async fn datafusion_filters_predicate_altura() {
    let batches = query_2flow_sql(
        INPUT,
        "SELECT subject, predicate, object FROM graph WHERE predicate = 'altura'",
    )
    .await
    .expect("query should succeed");
    let batch = &batches[0];
    assert_eq!(batch.num_rows(), 1);
    let object = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("object should be Utf8");
    assert_eq!(object.value(0), "330");
}

#[tokio::test]
async fn end_to_end_query_returns_330() {
    let batches = query_2flow_sql(
        INPUT,
        "SELECT object FROM graph WHERE subject = 'Torre_Eiffel' AND predicate = 'altura'",
    )
    .await
    .expect("query should succeed");
    let batch = &batches[0];
    let object = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("object should be Utf8");
    assert_eq!(object.len(), 1);
    assert_eq!(object.value(0), "330");
}
