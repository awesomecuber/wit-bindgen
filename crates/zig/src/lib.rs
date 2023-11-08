use std::fmt::Write;

use heck::{ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use wit_bindgen_core::{
    abi::{AbiVariant, Bindgen, Instruction, LiftLower, WasmType},
    uwriteln,
    wit_parser::{
        Docs, Enum, Flags, Function, InterfaceId, Record, Resolve, Result_, Results, SizeAlign,
        Tuple, Type, TypeId, Variant, WorldId, WorldKey,
    },
    Files, Source, WorldGenerator,
};

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {}

impl Opts {
    pub fn build(&self) -> Box<dyn WorldGenerator> {
        Box::new(Zig {
            _opts: self.clone(),
            ..Zig::default()
        })
    }
}

#[derive(Default)]
pub struct Zig {
    _opts: Opts,
    imports: Source,
    exports: Source,
}

impl WorldGenerator for Zig {
    fn import_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        iface: InterfaceId,
        _files: &mut Files,
    ) {
        let name_raw = &resolve.name_world_key(name);
        // self.imports
        //     .push_str(&format!("// Import functions from {name_raw}\n"));

        let mut extern_decls = Vec::new();
        for (_name, func) in resolve.interfaces[iface].functions.iter() {
            let (import_source, extern_decl) = import(resolve, name_raw, func);
            self.imports.push_str(&import_source);
            extern_decls.push(extern_decl);
        }

        uwriteln!(self.imports, "const wasm_imports = struct {{");
        for extern_decl in extern_decls {
            self.imports.push_str(&extern_decl);
            self.imports.push_str("\n");
        }
        uwriteln!(self.imports, "}};")
    }

    fn export_interface(
        &mut self,
        _resolve: &Resolve,
        _name: &WorldKey,
        _iface: InterfaceId,
        _files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn import_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) {
        let mut extern_decls = Vec::new();
        for (_name, func) in funcs.iter() {
            let (import_source, extern_decl) = import(resolve, "$root", func);
            self.imports.push_str(&import_source);
            extern_decls.push(extern_decl);
        }

        uwriteln!(self.imports, "const wasm_imports = struct {{");
        for extern_decl in extern_decls {
            self.imports.push_str(&extern_decl);
            self.imports.push_str("\n");
        }
        uwriteln!(self.imports, "}};")
    }

    fn export_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) -> anyhow::Result<()> {
        let mut interface_generator = InterfaceGenerator {
            resolve,
            world,
            src: Source::default(),
        };
        for (_name, func) in funcs.iter() {
            interface_generator.export(None, func);
        }

        self.exports.push_str(&interface_generator.src);
        Ok(())
    }

    fn import_types(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        _types: &[(&str, TypeId)],
        _files: &mut Files,
    ) {
        todo!()
    }

    fn finish(&mut self, resolve: &Resolve, world: WorldId, files: &mut Files) {
        let file_name = format!("{}.zig", &resolve.worlds[world].name.to_snake_case());

        files.push(&file_name, b"const exports = @import(\"exports.zig\");\n\n");
        files.push(&file_name, self.imports.as_bytes());
        files.push(&file_name, b"\n");
        files.push(&file_name, self.exports.as_bytes());
    }
}

struct InterfaceGenerator<'a> {
    resolve: &'a Resolve,
    world: WorldId,
    src: Source,
}

fn import(resolve: &Resolve, library_name: &str, func: &Function) -> (Source, String) {
    let extern_statement = get_extern_statement(resolve, library_name, func);

    let mut src = Source::default();

    let result_type = match &func.results {
        Results::Named(params) => match params.len() {
            0 => "void",
            _ => todo!(),
        },
        Results::Anon(t) => wit_type_to_zig_type(*t),
    };
    uwriteln!(
        src,
        "pub fn {}({}) {} {{",
        func.name.to_snake_case(),
        func.params
            .iter()
            .map(|(name, wit_type)| format!("{}: {}", name, wit_type_to_zig_type(*wit_type)))
            .collect::<Vec<_>>()
            .join(", "),
        result_type,
    );

    let mut func_bindgen = FunctionBindgen::new(resolve);
    for (param_name, _param_type) in func.params.iter() {
        func_bindgen.params.push(param_name.clone());
    }
    wit_bindgen_core::abi::call(
        resolve,
        AbiVariant::GuestImport,
        LiftLower::LowerArgsLiftResults,
        func,
        &mut func_bindgen,
    );
    src.push_str(&func_bindgen.src);

    uwriteln!(src, "}}");

    (src, extern_statement)
}

impl<'a> InterfaceGenerator<'a> {
    fn export(&mut self, interface_name: Option<&WorldKey>, func: &Function) {
        let sig = self.resolve.wasm_signature(AbiVariant::GuestExport, func);

        let result_type = match &func.results {
            Results::Named(params) => match params.len() {
                0 => "void",
                _ => todo!(),
            },
            Results::Anon(t) => wit_type_to_zig_type(*t),
        };
        uwriteln!(
            self.src,
            "export fn @\"{}\"({}) {} {{",
            func.name,
            sig.params
                .iter()
                .enumerate()
                .map(|(i, wasm_type)| format!("p{}: {}", i, wasm_type_to_zig_type(*wasm_type)))
                .collect::<Vec<_>>()
                .join(", "),
            result_type,
        );

        let mut func_bindgen = FunctionBindgen::new(self.resolve);
        func_bindgen.export_name = Some("TestWorld".to_string());
        for i in 0..sig.params.len() {
            func_bindgen.params.push(format!("p{i}"));
        }
        wit_bindgen_core::abi::call(
            self.resolve,
            AbiVariant::GuestExport,
            LiftLower::LiftArgsLowerResults,
            func,
            &mut func_bindgen,
        );
        self.src.push_str(&func_bindgen.src);

        uwriteln!(self.src, "}}");
    }
}

fn get_extern_statement(resolve: &Resolve, library_name: &str, func: &Function) -> String {
    let sig = resolve.wasm_signature(AbiVariant::GuestImport, func);
    let result_type = match sig.results.len() {
        0 => "void",
        1 => wasm_type_to_zig_type(sig.results[0]),
        _ => unreachable!(),
    };
    format!(
        "extern \"{}\" fn @\"{}\"({}) {};",
        library_name,
        func.name,
        sig.params
            .iter()
            .enumerate()
            .map(|(i, wasm_type)| format!("p{}: {}", i, wasm_type_to_zig_type(*wasm_type)))
            .collect::<Vec<_>>()
            .join(", "),
        result_type,
    )
}

fn wasm_type_to_zig_type(wasm_type: WasmType) -> &'static str {
    match wasm_type {
        WasmType::I32 => "i32",
        WasmType::I64 => "i64",
        WasmType::F32 => "f32",
        WasmType::F64 => "f64",
    }
}

fn wit_type_to_zig_type(wit_type: Type) -> &'static str {
    match wit_type {
        Type::Bool => todo!(),
        Type::U8 => todo!(),
        Type::U16 => todo!(),
        Type::U32 => "u32",
        Type::U64 => todo!(),
        Type::S8 => todo!(),
        Type::S16 => todo!(),
        Type::S32 => "i32",
        Type::S64 => todo!(),
        Type::Float32 => todo!(),
        Type::Float64 => todo!(),
        Type::Char => todo!(),
        Type::String => "[]const u8",
        Type::Id(_) => todo!(),
    }
}

struct FunctionBindgen {
    params: Vec<String>,
    src: Source,
    sizes: SizeAlign,
    export_name: Option<String>,
}

impl FunctionBindgen {
    fn new(resolve: &Resolve) -> FunctionBindgen {
        let mut sizes = SizeAlign::default();
        sizes.fill(resolve);
        FunctionBindgen {
            params: Vec::new(),
            src: Source::default(),
            sizes,
            export_name: None,
        }
    }
}

impl Bindgen for FunctionBindgen {
    type Operand = String;

    fn emit(
        &mut self,
        resolve: &Resolve,
        inst: &Instruction<'_>,
        operands: &mut Vec<Self::Operand>,
        results: &mut Vec<Self::Operand>,
    ) {
        match inst {
            Instruction::GetArg { nth } => results.push(self.params[*nth].clone()),
            Instruction::StringLower { realloc: _ } => {
                let str = &operands[0];
                results.push(format!("@as(i32, @intCast(@intFromPtr({str}.ptr)))"));
                results.push(format!("@as(i32, @intCast({str}.len))"));
            }
            Instruction::CallWasm { name, sig } => {
                match sig.results.len() {
                    0 => {}
                    1 => {
                        self.src.push_str("const ret = ");
                        results.push("ret".into());
                    }
                    _ => unimplemented!(),
                }

                uwriteln!(
                    self.src,
                    "wasm_imports.@\"{}\"({});",
                    name,
                    operands.join(", ")
                );
            }
            Instruction::CallInterface { func } => {
                match &func.results {
                    Results::Named(params) => match params.len() {
                        0 => {}
                        _ => todo!(),
                    },
                    Results::Anon(_) => {
                        self.src.push_str("const ret = ");
                        results.push("ret".into());
                    }
                }

                uwriteln!(
                    self.src,
                    "exports.{}.{}({});",
                    self.export_name.clone().unwrap(),
                    func.name.to_lower_camel_case(),
                    operands.join(", ")
                );
            }
            Instruction::Return { amt, func: _ } => {
                assert!(*amt <= 1);
                if *amt == 1 {
                    uwriteln!(self.src, "return {};", operands[0]);
                }
            }
            Instruction::I32FromS32 | Instruction::S32FromI32 => {
                results.push(operands[0].clone());
            }
            Instruction::I32FromU32 => results.push(format!("@as(i32, @intCast({}))", operands[0])),
            Instruction::U32FromI32 => results.push(format!("@as(u32, @intCast({}))", operands[0])),
            _ => todo!("{inst:?}"),
        }
    }

    fn return_pointer(&mut self, _size: usize, _align: usize) -> Self::Operand {
        todo!()
    }

    fn push_block(&mut self) {
        todo!()
    }

    fn finish_block(&mut self, _operand: &mut Vec<Self::Operand>) {
        todo!()
    }

    fn sizes(&self) -> &SizeAlign {
        &self.sizes
    }

    fn is_list_canonical(&self, _resolve: &Resolve, _element: &Type) -> bool {
        todo!()
    }
}
