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
    pub values: CArrowStringView,
    pub rows: usize,
    pub valid_lines: usize,
    pub invalid_lines: usize,
    pub total_lines: usize,
    pub total_triples: usize,
}

unsafe extern "C" {
    pub fn twoflow_parse_buffer(ptr: *const u8, len: usize) -> *mut CArrowRecordBatch2Flow;
    pub fn twoflow_free_batch(batch: *mut CArrowRecordBatch2Flow);
    pub fn twoflow_parse_grouped_json(ptr: *const u8, len: usize, out_len: *mut usize) -> *mut u8;
    pub fn twoflow_parse_triples_json(ptr: *const u8, len: usize, out_len: *mut usize) -> *mut u8;
    pub fn twoflow_free_json(ptr: *mut u8, len: usize);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    pub valid_lines: usize,
    pub invalid_lines: usize,
    pub total_lines: usize,
    pub total_triples: usize,
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
            total_triples: raw.total_triples,
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

pub(crate) fn parse_json_with(
    input: &[u8],
    f: unsafe extern "C" fn(*const u8, usize, *mut usize) -> *mut u8,
) -> Result<String> {
    let mut len = 0usize;
    // SAFETY: input pointer is valid for the call. Zig returns an allocated JSON
    // buffer and length that must be released exactly once with twoflow_free_json.
    let ptr = unsafe { f(input.as_ptr(), input.len(), &mut len as *mut usize) };
    let ptr = NonNull::new(ptr)
        .ok_or_else(|| DataFusionError::Execution("Zig JSON export returned null".to_string()))?;
    // SAFETY: Zig returned ptr/len for a valid allocation. We copy into a Rust
    // String before freeing the Zig buffer, so no borrowed data outlives it.
    let bytes = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) };
    let json = std::str::from_utf8(bytes)
        .map_err(|err| DataFusionError::Execution(format!("Zig JSON was not UTF-8: {err}")))?
        .to_owned();
    // SAFETY: ptr/len came from the Zig JSON allocator and have not been freed.
    unsafe { twoflow_free_json(ptr.as_ptr(), len) };
    Ok(json)
}
