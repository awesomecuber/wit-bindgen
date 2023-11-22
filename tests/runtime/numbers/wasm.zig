const std = @import("std");
const math = std.math;
const numbers = @import("numbers.zig");

fn expectEqual(left: anytype, right: @TypeOf(left)) void {
    if (left != right) {
        std.debug.panic("expected equal, but: {} != {}\n", .{ left, right });
    }
}

pub const TestWorld = struct {
    pub fn testImports() void {
        expectEqual(roundtripU8(1), 1);
        expectEqual(roundtripU8(0), 0);
        expectEqual(roundtripU8(math.maxInt(u8)), math.maxInt(u8));

        expectEqual(roundtripS8(1), 1);
        expectEqual(roundtripS8(math.minInt(i8)), math.minInt(i8));
        expectEqual(roundtripS8(math.maxInt(i8)), math.maxInt(i8));

        expectEqual(roundtripU16(1), 1);
        expectEqual(roundtripU16(0), 0);
        expectEqual(roundtripU16(math.maxInt(u16)), math.maxInt(u16));

        expectEqual(roundtripS16(1), 1);
        expectEqual(roundtripS16(math.minInt(i16)), math.minInt(i16));
        expectEqual(roundtripS16(math.maxInt(i16)), math.maxInt(i16));

        expectEqual(roundtripU32(1), 1);
        expectEqual(roundtripU32(0), 0);
        expectEqual(roundtripU32(math.maxInt(u32)), math.maxInt(u32));

        expectEqual(roundtripS32(1), 1);
        expectEqual(roundtripS32(math.minInt(i32)), math.minInt(i32));
        expectEqual(roundtripS32(math.maxInt(i32)), math.maxInt(i32));

        expectEqual(roundtripU64(1), 1);
        expectEqual(roundtripU64(0), 0);
        expectEqual(roundtripU64(math.maxInt(u64)), math.maxInt(u64));

        expectEqual(roundtripS64(1), 1);
        expectEqual(roundtripS64(math.minInt(i64)), math.minInt(i64));
        expectEqual(roundtripS64(math.maxInt(i64)), math.maxInt(i64));

        expectEqual(roundtripFloat32(1.0), 1.0);
        expectEqual(roundtripFloat32(math.inf(f32)), math.inf(f32));
        expectEqual(roundtripFloat32(-math.inf(f32)), -math.inf(f32));
        if (!math.isNan(roundtripFloat32(math.nan(f32)))) {
            std.debug.panic("expected nan", .{});
        }

        expectEqual(roundtripFloat64(1.0), 1.0);
        expectEqual(roundtripFloat64(math.inf(f64)), math.inf(f64));
        expectEqual(roundtripFloat64(-math.inf(f64)), -math.inf(f64));
        if (!math.isNan(roundtripFloat64(math.nan(f64)))) {
            std.debug.panic("expected nan", .{});
        }

        expectEqual(roundtripChar('a'), 'a');
        expectEqual(roundtripChar(' '), ' ');
        expectEqual(roundtripChar('🚩'), '🚩');
    }

    pub fn roundtripU8(a: u8) u8 {
        return a;
    }
    pub fn roundtripS8(a: i8) i8 {
        return a;
    }
    pub fn roundtripU16(a: u16) u16 {
        return a;
    }
    pub fn roundtripS16(a: i16) i16 {
        return a;
    }
    pub fn roundtripU32(a: u32) u32 {
        return a;
    }
    pub fn roundtripS32(a: i32) i32 {
        return a;
    }
    pub fn roundtripU64(a: u64) u64 {
        return a;
    }
    pub fn roundtripS64(a: i64) i64 {
        return a;
    }
    pub fn roundtripFloat32(a: f32) f32 {
        return a;
    }
    pub fn roundtripFloat64(a: f64) f64 {
        return a;
    }
    pub fn roundtripChar(a: u32) u32 {
        return a;
    }
    var val: u32 = 0;
    pub fn setScalar(a: u32) void {
        val = a;
    }
    pub fn getScalar() u32 {
        return val;
    }
};
