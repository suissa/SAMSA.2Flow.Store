const std = @import("std");
const ast = @import("ast.zig");

pub const ParseStats = ast.TwoFlowParseStats;

pub const ArrowStringArray = struct {
    allocator: std.mem.Allocator,
    offsets: std.ArrayList(i32),
    data: std.ArrayList(u8),

    pub fn init(allocator: std.mem.Allocator) !ArrowStringArray {
        var offsets: std.ArrayList(i32) = .empty;
        errdefer offsets.deinit(allocator);
        try offsets.append(allocator, 0);
        return .{ .allocator = allocator, .offsets = offsets, .data = .empty };
    }

    pub fn deinit(self: *ArrowStringArray) void {
        self.offsets.deinit(self.allocator);
        self.data.deinit(self.allocator);
        self.* = undefined;
    }

    pub fn append(self: *ArrowStringArray, value: []const u8) !void {
        if (value.len > @as(usize, @intCast(std.math.maxInt(i32)))) return error.ArrowUtf8ValueTooLarge;
        try self.data.appendSlice(self.allocator, value);
        if (self.data.items.len > @as(usize, @intCast(std.math.maxInt(i32)))) return error.ArrowUtf8ColumnTooLarge;
        try self.offsets.append(self.allocator, @intCast(self.data.items.len));
    }

    pub fn len(self: *const ArrowStringArray) usize {
        return if (self.offsets.items.len == 0) 0 else self.offsets.items.len - 1;
    }
};

pub const ArrowRecordBatch2Flow = struct {
    subjects: ArrowStringArray,
    predicates: ArrowStringArray,
    values: ArrowStringArray,

    pub fn init(allocator: std.mem.Allocator) !ArrowRecordBatch2Flow {
        var subjects = try ArrowStringArray.init(allocator);
        errdefer subjects.deinit();
        var predicates = try ArrowStringArray.init(allocator);
        errdefer predicates.deinit();
        var values = try ArrowStringArray.init(allocator);
        errdefer values.deinit();
        return .{ .subjects = subjects, .predicates = predicates, .values = values };
    }

    pub fn deinit(self: *ArrowRecordBatch2Flow) void {
        self.subjects.deinit();
        self.predicates.deinit();
        self.values.deinit();
        self.* = undefined;
    }

    pub fn rows(self: *const ArrowRecordBatch2Flow) usize {
        return self.subjects.len();
    }

    pub fn appendTriple(self: *ArrowRecordBatch2Flow, subject: []const u8, predicate: []const u8, value: []const u8) !void {
        const s_offsets = self.subjects.offsets.items.len;
        const s_data = self.subjects.data.items.len;
        const p_offsets = self.predicates.offsets.items.len;
        const p_data = self.predicates.data.items.len;
        const v_offsets = self.values.offsets.items.len;
        const v_data = self.values.data.items.len;
        errdefer self.subjects.offsets.shrinkRetainingCapacity(s_offsets);
        errdefer self.subjects.data.shrinkRetainingCapacity(s_data);
        errdefer self.predicates.offsets.shrinkRetainingCapacity(p_offsets);
        errdefer self.predicates.data.shrinkRetainingCapacity(p_data);
        errdefer self.values.offsets.shrinkRetainingCapacity(v_offsets);
        errdefer self.values.data.shrinkRetainingCapacity(v_data);
        try self.subjects.append(subject);
        try self.predicates.append(predicate);
        try self.values.append(value);
    }
};
