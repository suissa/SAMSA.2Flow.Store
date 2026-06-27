const std = @import("std");

pub const ParseStats = extern struct {
    valid_lines: usize,
    invalid_lines: usize,
    total_lines: usize,
};

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

    pub fn rollback(self: *ArrowStringArray, offsets_len: usize, data_len: usize) void {
        self.offsets.shrinkRetainingCapacity(offsets_len);
        self.data.shrinkRetainingCapacity(data_len);
    }
};

pub const ArrowRecordBatch2Flow = struct {
    subjects: ArrowStringArray,
    predicates: ArrowStringArray,
    objects: ArrowStringArray,

    pub fn init(allocator: std.mem.Allocator) !ArrowRecordBatch2Flow {
        var subjects = try ArrowStringArray.init(allocator);
        errdefer subjects.deinit();
        var predicates = try ArrowStringArray.init(allocator);
        errdefer predicates.deinit();
        var objects = try ArrowStringArray.init(allocator);
        errdefer objects.deinit();
        return .{ .subjects = subjects, .predicates = predicates, .objects = objects };
    }

    pub fn deinit(self: *ArrowRecordBatch2Flow) void {
        self.subjects.deinit();
        self.predicates.deinit();
        self.objects.deinit();
        self.* = undefined;
    }

    pub fn rows(self: *const ArrowRecordBatch2Flow) usize {
        return self.subjects.len();
    }
};
