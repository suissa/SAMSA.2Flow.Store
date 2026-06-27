const std = @import("std");
const root = @import("twoflow_parser");

fn parse(input: []const u8) !root.TwoFlowDocument {
    return root.parse2FlowTriples(std.testing.allocator, input);
}

test "findCharSIMD finds delimiters" {
    const input = "Torre_Eiffel -> altura:330\n";
    try std.testing.expectEqual(@as(?usize, 13), root.findCharSIMD(input, 0, '-'));
    try std.testing.expectEqual(@as(?usize, 22), root.findCharSIMD(input, 0, ':'));
    try std.testing.expectEqual(@as(?usize, 26), root.findCharSIMD(input, 0, '\n'));
}

test "one line one pair" {
    var doc = try parse("Torre_Eiffel -> altura:330");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 1), doc.stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 1), doc.stats.total_triples);
    try std.testing.expectEqualStrings("Torre_Eiffel", doc.triples.items[0].subject);
    try std.testing.expectEqualStrings("altura", doc.triples.items[0].predicate);
    try std.testing.expectEqualStrings("330", doc.triples.items[0].value);
}

test "one line multiple pairs expands triples" {
    var doc = try parse("Torre_Eiffel -> localizacao:Paris, altura:330, tipo:Monumento");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 1), doc.statements.items.len);
    try std.testing.expectEqual(@as(usize, 3), doc.statements.items[0].pairs.items.len);
    try std.testing.expectEqual(@as(usize, 3), doc.triples.items.len);
    try std.testing.expectEqual(@as(usize, 1), doc.triples.items[1].line);
    try std.testing.expectEqual(@as(usize, 1), doc.triples.items[1].pair_index);
}

test "multiple lines and spaces" {
    var doc = try parse("Torre_Eiffel   ->   localizacao : Paris , altura : 330 , tipo : Monumento\nParis -> pais:Franca, continente:Europa\n");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 2), doc.stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 5), doc.stats.total_triples);
    try std.testing.expectEqualStrings("Paris", doc.triples.items[0].value);
}

test "line comments and inline comments" {
    var doc = try parse("# comment\nTorre_Eiffel -> localizacao:Paris, altura:330 # monumento frances\n");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 1), doc.stats.total_lines);
    try std.testing.expectEqual(@as(usize, 2), doc.stats.total_triples);
}

test "strings can contain comma colon arrow hash and escaped quotes" {
    var doc = try parse("Texto_1 -> conteudo:\"isso tem vírgula, dois pontos: e seta -> dentro # ok\", autor:\"Jean \\\"JC\\\"\"\n");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 2), doc.stats.total_triples);
    try std.testing.expectEqual(root.TypeHint.string, doc.statements.items[0].pairs.items[0].type_hint);
    try std.testing.expect(std.mem.indexOf(u8, doc.triples.items[0].value, ", dois pontos:") != null);
    try std.testing.expect(std.mem.indexOf(u8, doc.triples.items[0].value, "->") != null);
}

test "type hints" {
    var doc = try parse("Pedido_999 -> status:Pago, total:89.90, ativo:true, data:2026-06-26, ts:2026-06-26T20:00Z\n");
    defer doc.deinit();
    const pairs = doc.statements.items[0].pairs.items;
    try std.testing.expectEqual(root.TypeHint.identifier, pairs[0].type_hint);
    try std.testing.expectEqual(root.TypeHint.number, pairs[1].type_hint);
    try std.testing.expectEqual(root.TypeHint.boolean, pairs[2].type_hint);
    try std.testing.expectEqual(root.TypeHint.date_like, pairs[3].type_hint);
    try std.testing.expectEqual(root.TypeHint.datetime_like, pairs[4].type_hint);
}

test "invalid missing arrow and colon" {
    var doc = try parse("Torre_Eiffel altura:330\nTorre_Eiffel -> altura\n");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 0), doc.stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 2), doc.stats.invalid_lines);
    try std.testing.expectEqualStrings("MISSING_ARROW", doc.errors.items[0].code);
    try std.testing.expectEqualStrings("MISSING_COLON", doc.errors.items[1].code);
}

test "invalid empty subject predicate value and empty pair" {
    var doc = try parse("-> altura:330\nTorre_Eiffel -> :330\nTorre_Eiffel -> altura:\nTorre_Eiffel -> localizacao:Paris, , altura:330\n");
    defer doc.deinit();
    try std.testing.expectEqual(@as(usize, 4), doc.stats.invalid_lines);
    try std.testing.expectEqualStrings("EMPTY_SUBJECT", doc.errors.items[0].code);
    try std.testing.expectEqualStrings("EMPTY_PREDICATE", doc.errors.items[1].code);
    try std.testing.expectEqualStrings("EMPTY_VALUE", doc.errors.items[2].code);
    try std.testing.expectEqualStrings("EMPTY_PAIR", doc.errors.items[3].code);
}

test "grouped json and triples json include expected structure" {
    var doc = try parse("Torre_Eiffel -> localizacao:Paris, altura:330, tipo:Monumento\n");
    defer doc.deinit();
    const grouped = try root.json_export.groupedJsonAlloc(std.testing.allocator, &doc);
    defer std.testing.allocator.free(grouped);
    try std.testing.expect(std.mem.indexOf(u8, grouped, "\"documents\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, grouped, "\"pairs\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, grouped, "\"total_triples\":3") != null);

    const triples = try root.json_export.triplesJsonAlloc(std.testing.allocator, &doc);
    defer std.testing.allocator.free(triples);
    try std.testing.expect(std.mem.indexOf(u8, triples, "\"triples\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, triples, "\"pair_index\":2") != null);
}

test "arrow batch has one row per pair" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowTriplesIntoBatch(&batch, "Torre_Eiffel -> localizacao:Paris, altura:330, tipo:Monumento");
    try std.testing.expectEqual(@as(usize, 3), stats.total_triples);
    try std.testing.expectEqual(@as(usize, 3), batch.rows());
    try std.testing.expectEqualSlices(i32, &[_]i32{ 0, 12, 24, 36 }, batch.subjects.offsets.items);
}
