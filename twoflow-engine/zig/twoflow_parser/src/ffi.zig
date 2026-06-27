const std = @import("std");
const layout = @import("arrow_layout.zig");
const parser = @import("parser.zig");
const json_export = @import("json_export.zig");

pub const CArrowStringView = extern struct {
    offsets_ptr: [*]const i32,
    offsets_len: usize,
    data_ptr: [*]const u8,
    data_len: usize,
};

pub const CArrowRecordBatch2Flow = extern struct {
    subjects: CArrowStringView,
    predicates: CArrowStringView,
    values: CArrowStringView,
    rows: usize,
    valid_lines: usize,
    invalid_lines: usize,
    total_lines: usize,
    total_triples: usize,
};

const OwnedCRecordBatch2Flow = struct {
    batch: layout.ArrowRecordBatch2Flow,
    c_view: CArrowRecordBatch2Flow,
};

fn stringView(array: *const layout.ArrowStringArray) CArrowStringView {
    return .{ .offsets_ptr = array.offsets.items.ptr, .offsets_len = array.offsets.items.len, .data_ptr = array.data.items.ptr, .data_len = array.data.items.len };
}

fn refreshCView(owned: *OwnedCRecordBatch2Flow, stats: layout.ParseStats) void {
    owned.c_view = .{
        .subjects = stringView(&owned.batch.subjects),
        .predicates = stringView(&owned.batch.predicates),
        .values = stringView(&owned.batch.values),
        .rows = owned.batch.rows(),
        .valid_lines = stats.valid_lines,
        .invalid_lines = stats.invalid_lines,
        .total_lines = stats.total_lines,
        .total_triples = stats.total_triples,
    };
}

pub fn twoflow_parse_buffer(ptr: [*]const u8, len: usize) ?*CArrowRecordBatch2Flow {
    const allocator = std.heap.c_allocator;
    const owned = allocator.create(OwnedCRecordBatch2Flow) catch return null;
    errdefer allocator.destroy(owned);
    owned.batch = layout.ArrowRecordBatch2Flow.init(allocator) catch return null;
    errdefer owned.batch.deinit();
    const stats = parser.parse2FlowTriplesIntoBatch(&owned.batch, ptr[0..len]) catch return null;
    refreshCView(owned, stats);
    return &owned.c_view;
}

pub fn twoflow_free_batch(batch: ?*CArrowRecordBatch2Flow) void {
    const c_ptr = batch orelse return;
    const owned: *OwnedCRecordBatch2Flow = @fieldParentPtr("c_view", c_ptr);
    owned.batch.deinit();
    std.heap.c_allocator.destroy(owned);
}

fn jsonAlloc(ptr: [*]const u8, len: usize, out_len: *usize, comptime grouped: bool) ?[*]u8 {
    const allocator = std.heap.c_allocator;
    var doc = parser.parse2FlowTriples(allocator, ptr[0..len]) catch return null;
    defer doc.deinit();
    const bytes = if (grouped) json_export.groupedJsonAlloc(allocator, &doc) catch return null else json_export.triplesJsonAlloc(allocator, &doc) catch return null;
    out_len.* = bytes.len;
    return bytes.ptr;
}

pub fn twoflow_parse_grouped_json(ptr: [*]const u8, len: usize, out_len: *usize) ?[*]u8 {
    return jsonAlloc(ptr, len, out_len, true);
}

pub fn twoflow_parse_triples_json(ptr: [*]const u8, len: usize, out_len: *usize) ?[*]u8 {
    return jsonAlloc(ptr, len, out_len, false);
}

pub fn twoflow_free_json(ptr: ?[*]u8, len: usize) void {
    const p = ptr orelse return;
    std.heap.c_allocator.free(p[0..len]);
}
