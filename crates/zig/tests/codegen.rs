use std::{path::Path, process::Command};

use heck::ToSnakeCase;

macro_rules! codegen_test {
    ($id:ident $name:tt $test:tt) => {
        #[test]
        fn $id() {
            test_helpers::run_world_codegen_test(
                "guest-zig",
                $test.as_ref(),
                |resolve, world, files| {
                    wit_bindgen_zig::Opts::default()
                        .build()
                        .generate(resolve, world, files)
                        .unwrap()
                },
                verify,
            );
        }
    };
}

test_helpers::codegen_tests!();

fn verify(dir: &Path, name: &str) {
    let mut cmd = Command::new("zig");
    cmd.arg("build-lib");
    cmd.arg(format!("{}.zig", name.to_snake_case()));
    cmd.args(["-target", "wasm32-wasi", "-dynamic", "-rdynamic"]);
    cmd.current_dir(dir);
    test_helpers::run_command(&mut cmd);
}
