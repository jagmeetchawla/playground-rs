import type { Template } from './templates'

export const ZIG_TEMPLATES: Template[] = [
  {
    id: 'zig_blank',
    name: 'Blank',
    description: 'Empty Zig playground',
    lang: 'zig' as any,
    code: `const std = @import("std");

pub fn main() void {

}
`,
  },
  {
    id: 'zig_hello',
    name: 'Hello World',
    description: 'Print to stderr via debug',
    lang: 'zig' as any,
    code: `const std = @import("std");

pub fn main() void {
    std.debug.print("Hello, {s}!\\n", .{"world"});
}
`,
  },
  {
    id: 'zig_arrays',
    name: 'Arrays & Slices',
    description: 'Working with arrays and slices',
    lang: 'zig' as any,
    code: `const std = @import("std");

pub fn main() void {
    // Fixed-size array
    const nums = [_]i32{ 10, 20, 30, 40, 50 };

    // Slice
    const slice = nums[1..4];
    for (slice) |val| {
        std.debug.print("{d} ", .{val});
    }
    std.debug.print("\\n", .{});

    // Array with sentinel
    const msg: [:0]const u8 = "Hello Zig";
    std.debug.print("Message: {s} (len={d})\\n", .{ msg, msg.len });
}
`,
  },
  {
    id: 'zig_structs',
    name: 'Structs',
    description: 'Define and use structs',
    lang: 'zig' as any,
    code: `const std = @import("std");

const Point = struct {
    x: f64,
    y: f64,

    fn distance(self: Point, other: Point) f64 {
        const dx = self.x - other.x;
        const dy = self.y - other.y;
        return @sqrt(dx * dx + dy * dy);
    }
};

pub fn main() void {
    const a = Point{ .x = 1.0, .y = 2.0 };
    const b = Point{ .x = 4.0, .y = 6.0 };

    std.debug.print("({d:.1}, {d:.1}) -> ({d:.1}, {d:.1})  distance = {d:.2}\\n", .{
        a.x, a.y, b.x, b.y, a.distance(b),
    });
}
`,
  },
  {
    id: 'zig_errors',
    name: 'Error Handling',
    description: 'Zig error unions and try/catch',
    lang: 'zig' as any,
    code: `const std = @import("std");

const ParseError = error{
    InvalidCharacter,
    Overflow,
};

fn parseDigit(c: u8) ParseError!u8 {
    if (c < '0' or c > '9') return ParseError.InvalidCharacter;
    return c - '0';
}

fn parseNumber(s: []const u8) ParseError!u32 {
    var result: u32 = 0;
    for (s) |c| {
        const digit = try parseDigit(c);
        result = result * 10 + digit;
    }
    return result;
}

pub fn main() void {
    const good = parseNumber("42") catch |err| {
        std.debug.print("Error: {any}\\n", .{err});
        return;
    };
    std.debug.print("Parsed: {d}\\n", .{good});

    if (parseNumber("abc")) |val| {
        std.debug.print("Got: {d}\\n", .{val});
    } else |err| {
        std.debug.print("Failed as expected: {any}\\n", .{err});
    }
}
`,
  },
  {
    id: 'zig_input',
    name: 'CLI Input',
    description: 'Read from stdin, parse input',
    lang: 'zig' as any,
    code: `const std = @import("std");

fn readLine(stdin: std.fs.File, buf: []u8) !?[]u8 {
    const n = try stdin.read(buf);
    if (n == 0) return null;
    var line = buf[0..n];
    if (line.len > 0 and line[line.len - 1] == '\\n') line = line[0 .. line.len - 1];
    if (line.len > 0 and line[line.len - 1] == '\\r') line = line[0 .. line.len - 1];
    return line;
}

pub fn main() !void {
    const stdin = std.fs.File.stdin();

    std.debug.print("What is your name? ", .{});

    var buf: [64]u8 = undefined;
    const name = try readLine(stdin, &buf) orelse return;

    std.debug.print("How old are you? ", .{});

    var age_buf: [16]u8 = undefined;
    const age_str = try readLine(stdin, &age_buf) orelse return;

    const age = std.fmt.parseInt(u32, age_str, 10) catch {
        std.debug.print("That's not a valid number, {s}!\\n", .{name});
        return;
    };
    std.debug.print("Hello {s}, you will be {d} next year!\\n", .{ name, age + 1 });
}
`,
  },
  {
    id: 'zig_allocator',
    name: 'Allocators',
    description: 'Dynamic memory with allocators',
    lang: 'zig' as any,
    code: `const std = @import("std");

pub fn main() !void {
    // General purpose allocator
    var gpa: std.heap.GeneralPurposeAllocator(.{}) = .init;
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Dynamic array (ArrayList — unmanaged, pass allocator to each call)
    var list: std.ArrayList(i32) = .empty;
    defer list.deinit(allocator);

    try list.append(allocator, 10);
    try list.append(allocator, 20);
    try list.append(allocator, 30);

    std.debug.print("Items: ", .{});
    for (list.items) |item| {
        std.debug.print("{d} ", .{item});
    }
    std.debug.print("\\nCount: {d}\\n", .{list.items.len});
}
`,
  },
]
