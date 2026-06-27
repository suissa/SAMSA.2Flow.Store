pub const arrow_layout = @import("arrow_layout.zig");
pub const parser = @import("parser.zig");
pub const ffi = @import("ffi.zig");

pub const ArrowStringArray = arrow_layout.ArrowStringArray;
pub const ArrowRecordBatch2Flow = arrow_layout.ArrowRecordBatch2Flow;
pub const ParseStats = arrow_layout.ParseStats;
pub const CArrowStringView = ffi.CArrowStringView;
pub const CArrowRecordBatch2Flow = ffi.CArrowRecordBatch2Flow;
pub const findCharSIMD = parser.findCharSIMD;
pub const parse2FlowSIMD = parser.parse2FlowSIMD;

export fn twoflow_parse_buffer(ptr: [*]const u8, len: usize) ?*CArrowRecordBatch2Flow {
    return ffi.twoflow_parse_buffer(ptr, len);
}

export fn twoflow_free_batch(batch: ?*CArrowRecordBatch2Flow) void {
    ffi.twoflow_free_batch(batch);
}
