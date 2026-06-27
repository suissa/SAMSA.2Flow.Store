pub const ast = @import("ast.zig");
pub const arrow_layout = @import("arrow_layout.zig");
pub const parser = @import("parser.zig");
pub const json_export = @import("json_export.zig");
pub const ffi = @import("ffi.zig");

pub const TypeHint = ast.TypeHint;
pub const TwoFlowDocument = ast.TwoFlowDocument;
pub const TwoFlowParseStats = ast.TwoFlowParseStats;
pub const ArrowStringArray = arrow_layout.ArrowStringArray;
pub const ArrowRecordBatch2Flow = arrow_layout.ArrowRecordBatch2Flow;
pub const ParseStats = arrow_layout.ParseStats;
pub const CArrowStringView = ffi.CArrowStringView;
pub const CArrowRecordBatch2Flow = ffi.CArrowRecordBatch2Flow;
pub const findCharSIMD = parser.findCharSIMD;
pub const parse2FlowTriples = parser.parse2FlowTriples;
pub const parse2FlowTriplesIntoBatch = parser.parse2FlowTriplesIntoBatch;
pub const parse2FlowSIMD = parser.parse2FlowSIMD;
pub const writeGroupedJson = json_export.writeGroupedJson;
pub const writeCanonicalTriplesJson = json_export.writeCanonicalTriplesJson;

export fn twoflow_parse_buffer(ptr: [*]const u8, len: usize) ?*CArrowRecordBatch2Flow {
    return ffi.twoflow_parse_buffer(ptr, len);
}

export fn twoflow_free_batch(batch: ?*CArrowRecordBatch2Flow) void {
    ffi.twoflow_free_batch(batch);
}

export fn twoflow_parse_grouped_json(ptr: [*]const u8, len: usize, out_len: *usize) ?[*]u8 {
    return ffi.twoflow_parse_grouped_json(ptr, len, out_len);
}

export fn twoflow_parse_triples_json(ptr: [*]const u8, len: usize, out_len: *usize) ?[*]u8 {
    return ffi.twoflow_parse_triples_json(ptr, len, out_len);
}

export fn twoflow_free_json(ptr: ?[*]u8, len: usize) void {
    ffi.twoflow_free_json(ptr, len);
}
