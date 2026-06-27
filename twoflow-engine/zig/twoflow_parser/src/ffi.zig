const std = @import("std");
const layout = @import("arrow_layout.zig");
const parser = @import("parser.zig");

pub const CArrowStringView = extern struct {
    offsets_ptr: [*]const i32,
    offsets_len: usize,
    data_ptr: [*]const u8,
    data_len: usize,
};

pub const CArrowRecordBatch2Flow = extern struct {
    subjects: CArrowStringView,
    predicates: CArrowStringView,
    objects: CArrowStringView,
    rows: usize,
    valid_lines: usize,
    invalid_lines: usize,
    total_lines: usize,
};

const OwnedCRecordBatch2Flow = struct {
    batch: layout.ArrowRecordBatch2Flow,
    c_view: CArrowRecordBatch2Flow,
};

fn stringView(array: *const layout.ArrowStringArray) CArrowStringView {
    return .{
        .offsets_ptr = array.offsets.items.ptr,
        .offsets_len = array.offsets.items.len,
        .data_ptr = array.data.items.ptr,
        .data_len = array.data.items.len,
    };
}

fn refreshCView(owned: *OwnedCRecordBatch2Flow, stats: layout.ParseStats) void {
    owned.c_view = .{
        .subjects = stringView(&owned.batch.subjects),
        .predicates = stringView(&owned.batch.predicates),
        .objects = stringView(&owned.batch.objects),
        .rows = owned.batch.rows(),
        .valid_lines = stats.valid_lines,
        .invalid_lines = stats.invalid_lines,
        .total_lines = stats.total_lines,
    };
}

pub fn twoflow_parse_buffer(ptr: [*]const u8, len: usize) ?*CArrowRecordBatch2Flow {
    const allocator = std.heap.c_allocator;
    const owned = allocator.create(OwnedCRecordBatch2Flow) catch return null;
    errdefer allocator.destroy(owned);
    owned.batch = layout.ArrowRecordBatch2Flow.init(allocator) catch return null;
    errdefer owned.batch.deinit();

    const input = ptr[0..len];
    const stats = parser.parse2FlowSIMD(&owned.batch, input) catch return null;
    refreshCView(owned, stats);
    return &owned.c_view;
}

pub fn twoflow_free_batch(batch: ?*CArrowRecordBatch2Flow) void {
    const c_ptr = batch orelse return;
    const owned: *OwnedCRecordBatch2Flow = @fieldParentPtr("c_view", c_ptr);
    owned.batch.deinit();
    std.heap.c_allocator.destroy(owned);
}
