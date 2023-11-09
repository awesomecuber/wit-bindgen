use std::fmt::{Debug, Write};

use heck::{ToLowerCamelCase, ToSnakeCase};
use wit_bindgen_core::{
    abi::{AbiVariant, Bindgen, Instruction, LiftLower, WasmType},
    uwriteln,
    wit_parser::{
        Function, InterfaceId, Resolve, Results, SizeAlign, Type, TypeDefKind, TypeId, WorldId,
        WorldKey,
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
        self.imports
            .push_str(&format!("// Import functions from {name_raw}\n"));

        let interface = &resolve.interfaces[iface];

        uwriteln!(
            self.imports,
            "pub const {} = struct {{",
            interface.name.as_ref().unwrap()
        );

        let mut extern_decls = Vec::new();
        for (_name, func) in interface.functions.iter() {
            let (import_source, extern_decl) = import(resolve, name_raw, func);
            self.imports.push_str(&import_source);
            extern_decls.push(extern_decl);
        }

        uwriteln!(self.imports, "const wasm_imports = struct {{");
        for extern_decl in extern_decls {
            self.imports.push_str(&extern_decl);
            self.imports.push_str("\n");
        }
        uwriteln!(self.imports, "}};");
        uwriteln!(self.imports, "}};");
    }

    fn export_interface(
        &mut self,
        resolve: &Resolve,
        _name: &WorldKey,
        iface: InterfaceId,
        _files: &mut Files,
    ) -> anyhow::Result<()> {
        let interface = &resolve.interfaces[iface];

        uwriteln!(
            self.exports,
            "pub const {} = struct {{",
            interface.name.as_ref().unwrap()
        );
        for (_name, func) in interface.functions.iter() {
            self.exports.push_str(&export(resolve, func));
        }
        uwriteln!(self.exports, "}};");
        Ok(())
    }

    fn import_funcs(
        &mut self,
        resolve: &Resolve,
        _world: WorldId,
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
        _world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) -> anyhow::Result<()> {
        for (_name, func) in funcs.iter() {
            self.exports.push_str(&export(resolve, func));
        }
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

        files.push(&file_name, self.imports.as_bytes());
        files.push(&file_name, b"\n");

        let mut exports = Source::default();
        exports.push_str("pub const exports = struct {\n");
        exports.push_str("const __user_exports = @import(\"exports.zig\");\n\n");
        exports.push_str(&self.exports);
        exports.push_str("};");
        files.push(&file_name, exports.as_bytes());
    }
}

fn import(resolve: &Resolve, library_name: &str, func: &Function) -> (Source, String) {
    let extern_statement = get_extern_statement(resolve, library_name, func);

    let mut src = Source::default();

    let result_type = match &func.results {
        Results::Named(params) => match params.len() {
            0 => "void".to_owned(),
            _ => todo!(),
        },
        Results::Anon(t) => wit_type_to_zig_type(resolve, *t),
    };
    uwriteln!(
        src,
        "pub fn {}({}) {} {{",
        func.name.to_snake_case(),
        func.params
            .iter()
            .map(|(name, wit_type)| format!(
                "{}: {}",
                name,
                wit_type_to_zig_type(resolve, *wit_type)
            ))
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

fn export(resolve: &Resolve, func: &Function) -> Source {
    let sig = resolve.wasm_signature(AbiVariant::GuestExport, func);

    let mut src = Source::default();

    let result_type = match &sig.results.len() {
        0 => "void",
        1 => wasm_type_to_zig_type(sig.results[0]),
        _ => todo!(),
    };
    uwriteln!(
        src,
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

    let mut func_bindgen = FunctionBindgen::new(resolve);
    func_bindgen.export_name = Some("TestWorld".to_string());
    for i in 0..sig.params.len() {
        func_bindgen.params.push(format!("p{i}"));
    }
    wit_bindgen_core::abi::call(
        resolve,
        AbiVariant::GuestExport,
        LiftLower::LiftArgsLowerResults,
        func,
        &mut func_bindgen,
    );
    src.push_str(&func_bindgen.src);

    uwriteln!(src, "}}");
    src
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

fn wit_type_to_zig_type(resolve: &Resolve, wit_type: Type) -> String {
    match wit_type {
        Type::Bool => todo!(),
        Type::U8 => "u8".to_owned(),
        Type::U16 => "u16".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::U64 => "u64".to_owned(),
        Type::S8 => "i8".to_owned(),
        Type::S16 => "i16".to_owned(),
        Type::S32 => "i32".to_owned(),
        Type::S64 => "i64".to_owned(),
        Type::Float32 => "f32".to_owned(),
        Type::Float64 => "f64".to_owned(),
        Type::Char => todo!(),
        Type::String => "[]const u8".to_owned(),
        Type::Id(id) => {
            let ty = &resolve.types[id];
            match &ty.kind {
                TypeDefKind::Record(_) => todo!(),
                TypeDefKind::Resource => todo!(),
                TypeDefKind::Handle(_) => todo!(),
                TypeDefKind::Flags(_) => todo!(),
                TypeDefKind::Tuple(t) => {
                    format!(
                        "struct{{ {} }}",
                        t.types
                            .iter()
                            .map(|t| wit_type_to_zig_type(resolve, *t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
                TypeDefKind::Variant(_) => todo!(),
                TypeDefKind::Enum(_) => todo!(),
                TypeDefKind::Option(_) => todo!(),
                TypeDefKind::Result(_) => todo!(),
                TypeDefKind::List(_) => todo!(),
                TypeDefKind::Future(_) => todo!(),
                TypeDefKind::Stream(_) => todo!(),
                TypeDefKind::Type(_) => todo!(),
                TypeDefKind::Unknown => todo!(),
            }
        }
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
        _resolve: &Resolve,
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
                    "__user_exports.{}.{}({});",
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
            Instruction::I32FromS32
            | Instruction::S32FromI32
            | Instruction::I64FromS64
            | Instruction::S64FromI64
            | Instruction::F32FromFloat32
            | Instruction::F64FromFloat64
            | Instruction::Float32FromF32
            | Instruction::Float64FromF64 => {
                results.push(operands[0].clone());
            }
            Instruction::I32FromU8
            | Instruction::I32FromU16
            | Instruction::I32FromU32
            | Instruction::I32FromS8
            | Instruction::I32FromS16 => {
                results.push(format!("@as(i32, @intCast({}))", operands[0]))
            }
            Instruction::I64FromU64 => results.push(format!("@as(i64, @intCast({}))", operands[0])),
            Instruction::S8FromI32 => results.push(format!("@as(i8, @intCast({}))", operands[0])),
            Instruction::U8FromI32 => results.push(format!("@as(u8, @intCast({}))", operands[0])),
            Instruction::S16FromI32 => results.push(format!("@as(i16, @intCast({}))", operands[0])),
            Instruction::U16FromI32 => results.push(format!("@as(u16, @intCast({}))", operands[0])),
            Instruction::U32FromI32 => results.push(format!("@as(u32, @intCast({}))", operands[0])),
            Instruction::U64FromI64 => results.push(format!("@as(u64, @intCast({}))", operands[0])),

            Instruction::I32Load8U { offset } => {
                uwriteln!(
                    self.src,
                    "const tmp = @as(i32, @intCast(@as(*const u8, @ptrFromInt({} + {})).*));",
                    operands[0],
                    offset,
                );
                results.push("tmp".to_owned());
            }
            Instruction::I64Load { offset } => {
                uwriteln!(
                    self.src,
                    "const tmp = @as(*const i64, @ptrFromInt({} + {})).*;",
                    operands[0],
                    offset,
                );
                results.push("tmp".to_owned());
            }

            Instruction::I32Store { offset } => {
                uwriteln!(
                    self.src,
                    "@as(*i32, @ptrFromInt({} + {})).* = {};",
                    operands[1],
                    offset,
                    operands[0]
                );
            }
            Instruction::I32Store8 { offset } => {
                uwriteln!(
                    self.src,
                    "@as(*u8, @ptrFromInt({} + {})).* = @as(u8, @intCast({}));",
                    operands[1],
                    offset,
                    operands[0]
                );
            }
            Instruction::I32Store16 { offset } => {
                uwriteln!(
                    self.src,
                    "@as(*u16, @ptrFromInt({} + {})).* = @as(u16, @intCast({}));",
                    operands[1],
                    offset,
                    operands[0]
                );
            }
            Instruction::I64Store { offset } => {
                uwriteln!(
                    self.src,
                    "@as(*i64, @ptrFromInt({} + {})).* = {};",
                    operands[1],
                    offset,
                    operands[0]
                );
            }

            Instruction::TupleLift { .. } => {
                results.push(format!(".{{ {} }}", operands.join(", ")))
            }
            Instruction::TupleLower { tuple, .. } => {
                for result in (0..tuple.types.len()).map(|i| format!("{}{}", operands[0], i)) {
                    results.push(result);
                }
            }
            _ => todo!("{inst:?}"),
        }
    }

    fn return_pointer(&mut self, _size: usize, _align: usize) -> Self::Operand {
        "ptr0".to_string()
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
