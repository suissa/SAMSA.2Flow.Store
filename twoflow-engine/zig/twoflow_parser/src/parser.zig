const std = @import("std");
const layout = @import("arrow_layout.zig");

pub const ArrowRecordBatch2Flow = layout.ArrowRecordBatch2Flow;
pub const ParseStats = layout.ParseStats;

pub inline fn findCharSIMD(buffer: []const u8, start: usize, char: u8) ?usize {
    const lanes = 16;
    const Vec = @Vector(lanes, u8);
    var i = start;
    const needle: Vec = @splat(char);
    while (i + lanes <= buffer.len) : (i += lanes) {
        var block: Vec = undefined;
        inline for (0..lanes) |lane| block[lane] = buffer[i + lane];
        const matches = block == needle;
        // Zig dev builds have changed boolean-vector mask casts several times.
        // A small lane loop is stable, keeps @Vector SIMD comparison, and avoids
        // assuming any particular @bitCast representation for bool vectors.
        inline for (0..lanes) |lane| {
            if (matches[lane]) return i + lane;
        }
    }
    while (i < buffer.len) : (i += 1) {
        if (buffer[i] == char) return i;
    }
    return null;
}

fn trimSpaces(bytes: []const u8) []const u8 {
    return std.mem.trim(u8, bytes, " \t");
}

fn trimLineEnd(bytes: []const u8) []const u8 {
    return std.mem.trimEnd(u8, bytes, "\r");
}

fn parseLineInto(batch: *ArrowRecordBatch2Flow, line_raw: []const u8) !bool {
    const line_without_cr = trimLineEnd(line_raw);
    const line = trimSpaces(line_without_cr);
    if (line.len == 0 or line[0] == '#') return false;

    const dash = findCharSIMD(line, 0, '-') orelse return error.InvalidLine;
    if (dash + 1 >= line.len or line[dash + 1] != '>') return error.InvalidLine;

    const subject = trimSpaces(line[0..dash]);
    if (subject.len == 0) return error.InvalidLine;

    const rest = trimSpaces(line[dash + 2 ..]);
    const colon = findCharSIMD(rest, 0, ':') orelse return error.InvalidLine;
    const predicate = trimSpaces(rest[0..colon]);
    const object = trimSpaces(trimLineEnd(rest[colon + 1 ..]));
    if (predicate.len == 0 or object.len == 0) return error.InvalidLine;

    const s_offsets = batch.subjects.offsets.items.len;
    const s_data = batch.subjects.data.items.len;
    const p_offsets = batch.predicates.offsets.items.len;
    const p_data = batch.predicates.data.items.len;
    const o_offsets = batch.objects.offsets.items.len;
    const o_data = batch.objects.data.items.len;
    errdefer batch.subjects.rollback(s_offsets, s_data);
    errdefer batch.predicates.rollback(p_offsets, p_data);
    errdefer batch.objects.rollback(o_offsets, o_data);

    try batch.subjects.append(subject);
    try batch.predicates.append(predicate);
    try batch.objects.append(object);
    return true;
}

pub fn parse2FlowSIMD(batch: *ArrowRecordBatch2Flow, buffer: []const u8) !ParseStats {
    var stats = ParseStats{ .valid_lines = 0, .invalid_lines = 0, .total_lines = 0 };
    var start: usize = 0;
    while (start <= buffer.len) {
        const maybe_nl = findCharSIMD(buffer, start, '\n');
        const end = maybe_nl orelse buffer.len;
        const line = buffer[start..end];
        if (line.len != 0 or maybe_nl != null) {
            const parsed = parseLineInto(batch, line) catch |err| switch (err) {
                error.InvalidLine => blk: {
                    stats.total_lines += 1;
                    stats.invalid_lines += 1;
                    break :blk false;
                },
                else => return err,
            };
            if (parsed) {
                stats.total_lines += 1;
                stats.valid_lines += 1;
            } else {
                const line_without_cr = trimLineEnd(line);
                const trimmed = trimSpaces(line_without_cr);
                if (!(trimmed.len == 0 or trimmed[0] == '#')) stats.total_lines += 1;
            }
        }
        if (maybe_nl == null) break;
        start = end + 1;
    }
    return stats;
}
