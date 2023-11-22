const smoke = @import("smoke.zig");

pub const TestWorld = struct {
    pub fn thunk() void {
        smoke.imports.thunk();
    }
};
