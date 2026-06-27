use crate::ffi::{CArrowStringView, TwoFlowParsedBatch};
use datafusion::arrow::array::{ArrayRef, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::error::{DataFusionError, Result};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowBridgeMode {
    ZeroCopy,
    CopyingFallback,
}

pub struct TwoFlowArrowBatch {
    owner: Arc<TwoFlowParsedBatch>,
    record_batch: RecordBatch,
    mode: ArrowBridgeMode,
}

impl TwoFlowArrowBatch {
    pub fn record_batch(&self) -> &RecordBatch {
        &self.record_batch
    }

    pub fn into_record_batch(self) -> RecordBatch {
        self.record_batch
    }

    pub fn mode(&self) -> ArrowBridgeMode {
        self.mode
    }

    pub fn owner(&self) -> &Arc<TwoFlowParsedBatch> {
        &self.owner
    }
}

pub fn twoflow_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("subject", DataType::Utf8, false),
        Field::new("predicate", DataType::Utf8, false),
        Field::new("object", DataType::Utf8, false),
    ]))
}

fn string_array_from_zig_copying(view: &CArrowStringView) -> Result<StringArray> {
    if view.offsets_len == 0 {
        return Err(DataFusionError::Execution(
            "invalid Zig string view: empty offsets".to_string(),
        ));
    }
    // SAFETY: Zig guarantees offsets_ptr points to offsets_len i32 values and
    // data_ptr points to data_len bytes while the owner is alive. This function
    // immediately copies both slices into Rust-owned Arrow arrays.
    let offsets = unsafe { std::slice::from_raw_parts(view.offsets_ptr, view.offsets_len) };
    // SAFETY: see the offsets safety note above; the bytes are copied before
    // this function returns, so no Rust Arrow buffer aliases Zig-owned memory.
    let data = unsafe { std::slice::from_raw_parts(view.data_ptr, view.data_len) };

    let values = offsets
        .windows(2)
        .map(|window| {
            let start = usize::try_from(window[0]).map_err(|_| {
                DataFusionError::Execution("negative Arrow string offset from Zig".to_string())
            })?;
            let end = usize::try_from(window[1]).map_err(|_| {
                DataFusionError::Execution("negative Arrow string offset from Zig".to_string())
            })?;
            if start > end || end > data.len() {
                return Err(DataFusionError::Execution(
                    "invalid Arrow string offsets from Zig".to_string(),
                ));
            }
            std::str::from_utf8(&data[start..end]).map_err(|err| {
                DataFusionError::Execution(format!("Zig produced non-UTF8 2flow token: {err}"))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(StringArray::from(values))
}

pub fn record_batch_from_zig(parsed: Arc<TwoFlowParsedBatch>) -> Result<TwoFlowArrowBatch> {
    let raw = parsed.as_raw();
    let schema = twoflow_schema();
    // TODO: replace with Arrow C Data Interface zero-copy. The current MVP is
    // honest CopyingFallback: Zig parses without input token temporaries and
    // builds Arrow-compatible contiguous buffers, then Rust copies those buffers
    // into Arrow-owned arrays to avoid unsound ownership/lifetime hacks.
    let columns: Vec<ArrayRef> = vec![
        Arc::new(string_array_from_zig_copying(&raw.subjects)?),
        Arc::new(string_array_from_zig_copying(&raw.predicates)?),
        Arc::new(string_array_from_zig_copying(&raw.objects)?),
    ];
    let record_batch = RecordBatch::try_new(schema, columns)?;
    Ok(TwoFlowArrowBatch {
        owner: parsed,
        record_batch,
        mode: ArrowBridgeMode::CopyingFallback,
    })
}
