const std = @import("std");

pub const TypeHint = enum {
    identifier,
    string,
    number,
    boolean,
    date_like,
    datetime_like,
    unknown,

    pub fn asText(self: TypeHint) []const u8 {
        return switch (self) {
            .identifier => "identifier",
            .string => "string",
            .number => "number",
            .boolean => "boolean",
            .date_like => "date_like",
            .datetime_like => "datetime_like",
            .unknown => "unknown",
        };
    }
};

pub const TwoFlowPair = struct {
    predicate: []const u8,
    value: []const u8,
    raw_value: []const u8,
    type_hint: TypeHint,
    pair_index: usize,
};

pub const TwoFlowStatement = struct {
    line: usize,
    subject: []const u8,
    pairs: std.ArrayList(TwoFlowPair),
};

pub const TwoFlowTriple = struct {
    subject: []const u8,
    predicate: []const u8,
    value: []const u8,
    line: usize,
    pair_index: usize,
};

pub const TwoFlowParseError = struct {
    line: usize,
    column: usize,
    code: []const u8,
    message: []const u8,
    raw_line: []const u8,
};

pub const TwoFlowParseStats = extern struct {
    total_lines: usize,
    valid_lines: usize,
    invalid_lines: usize,
    total_triples: usize,
};

pub const TwoFlowDocument = struct {
    allocator: std.mem.Allocator,
    statements: std.ArrayList(TwoFlowStatement),
    triples: std.ArrayList(TwoFlowTriple),
    errors: std.ArrayList(TwoFlowParseError),
    stats: TwoFlowParseStats,

    pub fn init(allocator: std.mem.Allocator) TwoFlowDocument {
        return .{
            .allocator = allocator,
            .statements = .empty,
            .triples = .empty,
            .errors = .empty,
            .stats = .{ .total_lines = 0, .valid_lines = 0, .invalid_lines = 0, .total_triples = 0 },
        };
    }

    pub fn deinit(self: *TwoFlowDocument) void {
        for (self.statements.items) |*stmt| {
            stmt.pairs.deinit(self.allocator);
        }
        self.statements.deinit(self.allocator);
        self.triples.deinit(self.allocator);
        self.errors.deinit(self.allocator);
        self.* = undefined;
    }
};
