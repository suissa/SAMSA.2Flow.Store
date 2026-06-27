const std = @import("std");
const ast = @import("ast.zig");

const JsonBufferWriter = struct {
    allocator: std.mem.Allocator,
    out: *std.ArrayList(u8),

    pub fn writeAll(self: *JsonBufferWriter, bytes: []const u8) !void {
        try self.out.appendSlice(self.allocator, bytes);
    }

    pub fn writeByte(self: *JsonBufferWriter, byte: u8) !void {
        try self.out.append(self.allocator, byte);
    }

    pub fn print(self: *JsonBufferWriter, comptime fmt: []const u8, args: anytype) !void {
        try self.out.print(self.allocator, fmt, args);
    }
};

fn writeJsonString(writer: anytype, value: []const u8) !void {
    try writer.writeByte('"');
    for (value) |ch| {
        switch (ch) {
            '"' => try writer.writeAll("\\\""),
            '\\' => try writer.writeAll("\\\\"),
            '\n' => try writer.writeAll("\\n"),
            '\r' => try writer.writeAll("\\r"),
            '\t' => try writer.writeAll("\\t"),
            else => if (ch < 0x20) try writer.print("\\u{x:0>4}", .{ch}) else try writer.writeByte(ch),
        }
    }
    try writer.writeByte('"');
}

fn writeStats(writer: anytype, stats: ast.TwoFlowParseStats) !void {
    try writer.print("{{\"total_lines\":{},\"valid_lines\":{},\"invalid_lines\":{},\"total_triples\":{}}}", .{ stats.total_lines, stats.valid_lines, stats.invalid_lines, stats.total_triples });
}

fn writeErrors(writer: anytype, doc: *const ast.TwoFlowDocument) !void {
    try writer.writeAll("\"errors\":[");
    for (doc.errors.items, 0..) |err, i| {
        if (i != 0) try writer.writeByte(',');
        try writer.print("{{\"line\":{},\"column\":{},\"code\":", .{ err.line, err.column });
        try writeJsonString(writer, err.code);
        try writer.writeAll(",\"message\":");
        try writeJsonString(writer, err.message);
        try writer.writeAll(",\"raw_line\":");
        try writeJsonString(writer, err.raw_line);
        try writer.writeByte('}');
    }
    try writer.writeByte(']');
}

pub fn writeGroupedJson(writer: anytype, doc: *const ast.TwoFlowDocument) !void {
    try writer.writeAll("{\"format\":\"2Flow-Triples\",\"version\":\"0.1\",\"documents\":[");
    for (doc.statements.items, 0..) |stmt, si| {
        if (si != 0) try writer.writeByte(',');
        try writer.print("{{\"line\":{},\"subject\":", .{stmt.line});
        try writeJsonString(writer, stmt.subject);
        try writer.writeAll(",\"pairs\":[");
        for (stmt.pairs.items, 0..) |pair, pi| {
            if (pi != 0) try writer.writeByte(',');
            try writer.writeAll("{\"predicate\":");
            try writeJsonString(writer, pair.predicate);
            try writer.writeAll(",\"value\":");
            try writeJsonString(writer, pair.value);
            try writer.writeAll(",\"raw_value\":");
            try writeJsonString(writer, pair.raw_value);
            try writer.writeAll(",\"type_hint\":");
            try writeJsonString(writer, pair.type_hint.asText());
            try writer.writeByte('}');
        }
        try writer.writeAll("]}");
    }
    try writer.writeAll("],\"stats\":");
    try writeStats(writer, doc.stats);
    try writer.writeByte(',');
    try writeErrors(writer, doc);
    try writer.writeByte('}');
}

pub fn writeCanonicalTriplesJson(writer: anytype, doc: *const ast.TwoFlowDocument) !void {
    try writer.writeAll("{\"format\":\"2Flow-Triples\",\"version\":\"0.1\",\"triples\":[");
    for (doc.triples.items, 0..) |triple, i| {
        if (i != 0) try writer.writeByte(',');
        try writer.writeAll("{\"subject\":");
        try writeJsonString(writer, triple.subject);
        try writer.writeAll(",\"predicate\":");
        try writeJsonString(writer, triple.predicate);
        try writer.writeAll(",\"value\":");
        try writeJsonString(writer, triple.value);
        try writer.print(",\"line\":{},\"pair_index\":{}}}", .{ triple.line, triple.pair_index });
    }
    try writer.writeAll("],\"stats\":");
    try writeStats(writer, doc.stats);
    try writer.writeByte(',');
    try writeErrors(writer, doc);
    try writer.writeByte('}');
}

pub fn groupedJsonAlloc(allocator: std.mem.Allocator, doc: *const ast.TwoFlowDocument) ![]u8 {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);
    var writer = JsonBufferWriter{ .allocator = allocator, .out = &out };
    try writeGroupedJson(&writer, doc);
    return out.toOwnedSlice(allocator);
}

pub fn triplesJsonAlloc(allocator: std.mem.Allocator, doc: *const ast.TwoFlowDocument) ![]u8 {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);
    var writer = JsonBufferWriter{ .allocator = allocator, .out = &out };
    try writeCanonicalTriplesJson(&writer, doc);
    return out.toOwnedSlice(allocator);
}
