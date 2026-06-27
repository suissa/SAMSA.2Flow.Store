const std = @import("std");
const ast = @import("ast.zig");
const layout = @import("arrow_layout.zig");

pub const ArrowRecordBatch2Flow = layout.ArrowRecordBatch2Flow;
pub const ParseStats = ast.TwoFlowParseStats;
pub const TwoFlowDocument = ast.TwoFlowDocument;

const LineParseError = error{
    MissingArrow,
    EmptySubject,
    EmptyPairList,
    EmptyPair,
    MissingColon,
    EmptyPredicate,
    EmptyValue,
    UnterminatedString,
};

const ParsedPair = struct {
    predicate: []const u8,
    value: []const u8,
    raw_value: []const u8,
    type_hint: ast.TypeHint,
};

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
        // A lane loop keeps the SIMD compare and avoids relying on @bitCast masks.
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

fn columnInLine(line: []const u8, sub: []const u8) usize {
    if (sub.len == 0) return 1;
    const base = @intFromPtr(line.ptr);
    const ptr = @intFromPtr(sub.ptr);
    if (ptr < base) return 1;
    return (ptr - base) + 1;
}

fn isEscaped(bytes: []const u8, idx: usize) bool {
    if (idx == 0) return false;
    var count: usize = 0;
    var i = idx;
    while (i > 0) {
        i -= 1;
        if (bytes[i] == '\\') count += 1 else break;
    }
    return (count % 2) == 1;
}

fn stripInlineComment(line: []const u8) ![]const u8 {
    var in_string = false;
    var i: usize = 0;
    while (i < line.len) : (i += 1) {
        const ch = line[i];
        if (ch == '"' and !isEscaped(line, i)) {
            in_string = !in_string;
            continue;
        }
        if (!in_string and ch == '#' and i > 0 and (line[i - 1] == ' ' or line[i - 1] == '\t')) {
            return trimSpaces(line[0..i]);
        }
    }
    if (in_string) return LineParseError.UnterminatedString;
    return trimSpaces(line);
}

fn findArrow(line: []const u8) ?usize {
    var start: usize = 0;
    while (findCharSIMD(line, start, '-')) |dash| {
        if (dash + 1 < line.len and line[dash + 1] == '>') return dash;
        start = dash + 1;
    }
    return null;
}

fn findDelimiterOutsideString(bytes: []const u8, char: u8) !?usize {
    var in_string = false;
    var i: usize = 0;
    while (i < bytes.len) : (i += 1) {
        const ch = bytes[i];
        if (ch == '"' and !isEscaped(bytes, i)) {
            in_string = !in_string;
            continue;
        }
        if (!in_string and ch == char) return i;
    }
    if (in_string) return LineParseError.UnterminatedString;
    return null;
}

fn normalizeValue(raw: []const u8) !struct { value: []const u8, raw_value: []const u8, type_hint: ast.TypeHint } {
    const trimmed = trimSpaces(raw);
    if (trimmed.len == 0) return LineParseError.EmptyValue;
    if (trimmed[0] == '"') {
        if (trimmed.len < 2 or trimmed[trimmed.len - 1] != '"' or isEscaped(trimmed, trimmed.len - 1)) {
            return LineParseError.UnterminatedString;
        }
        return .{ .value = trimmed[1 .. trimmed.len - 1], .raw_value = trimmed, .type_hint = .string };
    }
    return .{ .value = trimmed, .raw_value = trimmed, .type_hint = inferTypeHint(trimmed) };
}

fn parsePair(line: []const u8, pair_raw: []const u8) !ParsedPair {
    const pair = trimSpaces(pair_raw);
    if (pair.len == 0) return LineParseError.EmptyPair;
    const colon = (try findDelimiterOutsideString(pair, ':')) orelse return LineParseError.MissingColon;
    const predicate = trimSpaces(pair[0..colon]);
    if (predicate.len == 0) return LineParseError.EmptyPredicate;
    const normalized = try normalizeValue(pair[colon + 1 ..]);
    _ = line;
    return .{ .predicate = predicate, .value = normalized.value, .raw_value = normalized.raw_value, .type_hint = normalized.type_hint };
}

fn isDigits(bytes: []const u8) bool {
    if (bytes.len == 0) return false;
    for (bytes) |ch| if (ch < '0' or ch > '9') return false;
    return true;
}

fn isNumber(bytes: []const u8) bool {
    if (bytes.len == 0) return false;
    var i: usize = if (bytes[0] == '-' or bytes[0] == '+') 1 else 0;
    if (i >= bytes.len) return false;
    var saw_digit = false;
    var saw_dot = false;
    while (i < bytes.len) : (i += 1) {
        const ch = bytes[i];
        if (ch >= '0' and ch <= '9') {
            saw_digit = true;
        } else if (ch == '.' and !saw_dot) {
            saw_dot = true;
        } else return false;
    }
    return saw_digit;
}

fn isIdentifierChar(ch: u8) bool {
    return std.ascii.isAlphanumeric(ch) or ch == '_' or ch == '-' or ch == '.' or ch == '/' or ch == '@';
}

fn inferTypeHint(value: []const u8) ast.TypeHint {
    if (std.mem.eql(u8, value, "true") or std.mem.eql(u8, value, "false")) return .boolean;
    if (isNumber(value)) return .number;
    if (value.len >= 11 and value[4] == '-' and value[7] == '-' and (value[10] == 'T' or value[10] == 't')) return .datetime_like;
    if (value.len == 10 and value[4] == '-' and value[7] == '-' and isDigits(value[0..4]) and isDigits(value[5..7]) and isDigits(value[8..10])) return .date_like;
    for (value) |ch| if (!isIdentifierChar(ch)) return .unknown;
    return .identifier;
}

fn errorInfo(err: LineParseError) struct { code: []const u8, message: []const u8 } {
    return switch (err) {
        error.MissingArrow => .{ .code = "MISSING_ARROW", .message = "Expected '->' between subject and pair list." },
        error.EmptySubject => .{ .code = "EMPTY_SUBJECT", .message = "Subject cannot be empty." },
        error.EmptyPairList => .{ .code = "EMPTY_PAIR_LIST", .message = "Pair list cannot be empty." },
        error.EmptyPair => .{ .code = "EMPTY_PAIR", .message = "Pair cannot be empty." },
        error.MissingColon => .{ .code = "MISSING_COLON", .message = "Expected ':' between predicate and value." },
        error.EmptyPredicate => .{ .code = "EMPTY_PREDICATE", .message = "Predicate cannot be empty." },
        error.EmptyValue => .{ .code = "EMPTY_VALUE", .message = "Value cannot be empty." },
        error.UnterminatedString => .{ .code = "UNTERMINATED_STRING", .message = "String literal is missing a closing quote." },
    };
}

fn appendParseError(doc: *TwoFlowDocument, line_no: usize, column: usize, err: LineParseError, raw_line: []const u8) !void {
    const info = errorInfo(err);
    try doc.errors.append(doc.allocator, .{ .line = line_no, .column = column, .code = info.code, .message = info.message, .raw_line = raw_line });
    doc.stats.invalid_lines += 1;
}

fn parseStatement(doc: *TwoFlowDocument, line_no: usize, raw_line: []const u8, clean_line: []const u8) !void {
    const arrow = findArrow(clean_line) orelse return appendParseError(doc, line_no, 1, error.MissingArrow, raw_line);
    const subject = trimSpaces(clean_line[0..arrow]);
    if (subject.len == 0) return appendParseError(doc, line_no, 1, error.EmptySubject, raw_line);
    const pair_list = trimSpaces(clean_line[arrow + 2 ..]);
    if (pair_list.len == 0) return appendParseError(doc, line_no, arrow + 3, error.EmptyPairList, raw_line);

    var stmt = ast.TwoFlowStatement{ .line = line_no, .subject = subject, .pairs = .empty };
    var committed = false;
    defer if (!committed) stmt.pairs.deinit(doc.allocator);

    var pair_start: usize = 0;
    var pair_index: usize = 0;
    while (pair_start <= pair_list.len) {
        var in_string = false;
        var end = pair_list.len;
        var i = pair_start;
        while (i < pair_list.len) : (i += 1) {
            const ch = pair_list[i];
            if (ch == '"' and !isEscaped(pair_list, i)) {
                in_string = !in_string;
            } else if (!in_string and ch == ',') {
                end = i;
                break;
            }
        }
        if (in_string) return appendParseError(doc, line_no, columnInLine(clean_line, pair_list[pair_start..]), error.UnterminatedString, raw_line);
        const pair_slice = pair_list[pair_start..end];
        const parsed = parsePair(clean_line, pair_slice) catch |err| {
            return appendParseError(doc, line_no, columnInLine(clean_line, pair_slice), err, raw_line);
        };
        try stmt.pairs.append(doc.allocator, .{ .predicate = parsed.predicate, .value = parsed.value, .raw_value = parsed.raw_value, .type_hint = parsed.type_hint, .pair_index = pair_index });
        try doc.triples.append(doc.allocator, .{ .subject = subject, .predicate = parsed.predicate, .value = parsed.value, .line = line_no, .pair_index = pair_index });
        pair_index += 1;
        if (end == pair_list.len) break;
        pair_start = end + 1;
    }

    doc.stats.valid_lines += 1;
    doc.stats.total_triples += stmt.pairs.items.len;
    try doc.statements.append(doc.allocator, stmt);
    committed = true;
}

pub fn parse2FlowTriples(allocator: std.mem.Allocator, input: []const u8) !TwoFlowDocument {
    var doc = ast.TwoFlowDocument.init(allocator);
    errdefer doc.deinit();
    var line_no: usize = 1;
    var start: usize = 0;
    while (start <= input.len) {
        const nl = findCharSIMD(input, start, '\n');
        const end = nl orelse input.len;
        const raw_line = trimLineEnd(input[start..end]);
        const stripped = stripInlineComment(raw_line) catch |err| {
            doc.stats.total_lines += 1;
            try appendParseError(&doc, line_no, 1, err, raw_line);
            if (nl == null) break;
            line_no += 1;
            start = end + 1;
            continue;
        };
        if (stripped.len != 0 and stripped[0] != '#') {
            doc.stats.total_lines += 1;
            try parseStatement(&doc, line_no, raw_line, stripped);
        }
        if (nl == null) break;
        line_no += 1;
        start = end + 1;
    }
    return doc;
}

pub fn parse2FlowTriplesIntoBatch(batch: *ArrowRecordBatch2Flow, input: []const u8) !ParseStats {
    var doc = try parse2FlowTriples(batch.subjects.allocator, input);
    defer doc.deinit();
    for (doc.triples.items) |triple| {
        try batch.appendTriple(triple.subject, triple.predicate, triple.value);
    }
    return doc.stats;
}

// Backwards-compatible alias for the first MVP API name.
pub fn parse2FlowSIMD(batch: *ArrowRecordBatch2Flow, buffer: []const u8) !ParseStats {
    return parse2FlowTriplesIntoBatch(batch, buffer);
}
