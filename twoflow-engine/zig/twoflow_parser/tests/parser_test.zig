const std = @import("std");
const root = @import("twoflow_parser");

test "findCharSIMD finds delimiters" {
    const input = "Torre_Eiffel -> altura:330\n";
    try std.testing.expectEqual(@as(?usize, 13), root.findCharSIMD(input, 0, '-'));
    try std.testing.expectEqual(@as(?usize, 22), root.findCharSIMD(input, 0, ':'));
    try std.testing.expectEqual(@as(?usize, 26), root.findCharSIMD(input, 0, '\n'));
}

test "parser accepts one valid line" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowSIMD(&batch, "Torre_Eiffel -> altura:330");
    try std.testing.expectEqual(@as(usize, 1), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 0), stats.invalid_lines);
    try std.testing.expectEqual(@as(usize, 1), batch.rows());
}

test "parser accepts multiple lines" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const input = "Torre_Eiffel -> altura:330\nTorre_Eiffel -> localizacao:Paris\n";
    const stats = try root.parse2FlowSIMD(&batch, input);
    try std.testing.expectEqual(@as(usize, 2), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 2), batch.rows());
}

test "parser ignores empty line" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowSIMD(&batch, "\nTorre_Eiffel -> altura:330\n");
    try std.testing.expectEqual(@as(usize, 1), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 0), stats.invalid_lines);
}

test "parser ignores hash comment line" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowSIMD(&batch, "# comment\nTorre_Eiffel -> altura:330\n");
    try std.testing.expectEqual(@as(usize, 1), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 0), stats.invalid_lines);
}

test "parser marks line without arrow invalid" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowSIMD(&batch, "Torre_Eiffel altura:330\n");
    try std.testing.expectEqual(@as(usize, 0), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 1), stats.invalid_lines);
}

test "parser marks line without colon invalid" {
    var batch = try root.ArrowRecordBatch2Flow.init(std.testing.allocator);
    defer batch.deinit();
    const stats = try root.parse2FlowSIMD(&batch, "Torre_Eiffel -> altura 330\n");
    try std.testing.expectEqual(@as(usize, 0), stats.valid_lines);
    try std.testing.expectEqual(@as(usize, 1), stats.invalid_lines);
}

test "arrow string offsets match concatenated bytes" {
    var array = try root.ArrowStringArray.init(std.testing.allocator);
    defer array.deinit();
    try array.append("abc");
    try array.append("de");
    try std.testing.expectEqualSlices(i32, &[_]i32{ 0, 3, 5 }, array.offsets.items);
    try std.testing.expectEqualSlices(u8, "abcde", array.data.items);
}
