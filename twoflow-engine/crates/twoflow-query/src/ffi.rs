use datafusion::error::{DataFusionError, Result};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(C)]
pub struct CArrowStringView {
    pub offsets_ptr: *const i32,
    pub offsets_len: usize,
    pub data_ptr: *const u8,
    pub data_len: usize,
}

#[repr(C)]
pub struct CArrowRecordBatch2Flow {
    pub subjects: CArrowStringView,
    pub predicates: CArrowStringView,
    pub objects: CArrowStringView,
    pub rows: usize,
    pub valid_lines: usize,
    pub invalid_lines: usize,
    pub total_lines: usize,
}

unsafe extern "C" {
    pub fn twoflow_parse_buffer(ptr: *const u8, len: usize) -> *mut CArrowRecordBatch2Flow;
    pub fn twoflow_free_batch(batch: *mut CArrowRecordBatch2Flow);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    pub valid_lines: usize,
    pub invalid_lines: usize,
    pub total_lines: usize,
}

/// RAII owner for a Zig-allocated CArrowRecordBatch2Flow.
///
/// The raw pointer and all nested offsets/data pointers remain valid until this
/// value is dropped. They must not be stored or dereferenced after Drop calls
/// `twoflow_free_batch`.
pub struct TwoFlowParsedBatch {
    raw: NonNull<CArrowRecordBatch2Flow>,
    // Keep this !Send and !Sync for the MVP. The Zig owner is immutable after
    // construction, but we avoid cross-thread FFI lifetime promises until the
    // Arrow C Data Interface owner model is implemented.
    _not_send_sync: PhantomData<*mut ()>,
}

impl TwoFlowParsedBatch {
    pub fn as_raw(&self) -> &CArrowRecordBatch2Flow {
        // SAFETY: self.raw is NonNull and owned by this RAII wrapper; Drop is
        // the only place that frees it, so the reference is valid for &self.
        unsafe { self.raw.as_ref() }
    }

    pub fn stats(&self) -> ParseStats {
        let raw = self.as_raw();
        ParseStats {
            valid_lines: raw.valid_lines,
            invalid_lines: raw.invalid_lines,
            total_lines: raw.total_lines,
        }
    }
}

impl Drop for TwoFlowParsedBatch {
    fn drop(&mut self) {
        // SAFETY: self.raw was returned by twoflow_parse_buffer and has not
        // been freed before because this type owns the unique free operation.
        unsafe { twoflow_free_batch(self.raw.as_ptr()) }
    }
}

pub fn parse_2flow_buffer(input: &[u8]) -> Result<TwoFlowParsedBatch> {
    // SAFETY: input.as_ptr() is valid for input.len() bytes for the duration of
    // the call. Zig copies accepted token bytes into its own columnar buffers
    // before returning and does not retain this input pointer.
    let raw = unsafe { twoflow_parse_buffer(input.as_ptr(), input.len()) };
    let raw = NonNull::new(raw).ok_or_else(|| {
        DataFusionError::Execution("Zig twoflow parser returned null".to_string())
    })?;
    Ok(TwoFlowParsedBatch {
        raw,
        _not_send_sync: PhantomData,
    })
}
