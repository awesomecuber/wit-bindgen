use std::fmt::Write;

use heck::ToSnakeCase;
use wit_bindgen_core::{
    abi::{AbiVariant, Bindgen, Instruction, LiftLower, WasmType},
    uwrite, uwriteln,
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
    src: Source,
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
        self.src
            .push_str(&format!("// Import functions from {name_raw}\n"));

        let mut interface_generator = InterfaceGenerator {
            resolve,
            src: Source::default(),
        };
        for (_name, func) in resolve.interfaces[iface].functions.iter() {
            interface_generator.import(Some(name), func);
        }

        self.src.push_str(&interface_generator.src);
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
        _world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) {
        let mut interface_generator = InterfaceGenerator {
            resolve,
            src: Source::default(),
        };
        for (_name, func) in funcs.iter() {
            interface_generator.import(None, func);
        }

        self.src.push_str(&interface_generator.src);
        dbg!(self.src.as_mut_string());
    }

    fn export_funcs(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        _funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
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
        let file_name = &resolve.worlds[world].name.to_snake_case();

        files.push(&format!("{file_name}.zig"), self.src.as_bytes());
    }
}

struct InterfaceGenerator<'a> {
    resolve: &'a Resolve,
    src: Source,
}

impl<'a> InterfaceGenerator<'a> {
    fn import(&mut self, interface_name: Option<&WorldKey>, func: &Function) {
        let link_against = match interface_name {
            Some(name) => self.resolve.name_world_key(name),
            None => "$root".to_string(),
        };
        let sig = self.resolve.wasm_signature(AbiVariant::GuestImport, func);
        let result_type = match sig.results.len() {
            0 => "void",
            1 => wasm_type_to_zig_type(sig.results[0]),
            _ => unreachable!(),
        };
        uwriteln!(
            self.src,
            "extern \"{}\" fn @\"{}\"({}) {};",
            link_against,
            func.name,
            func.params
                .iter()
                .map(|(name, _type)| name)
                .zip(sig.params)
                .map(|(name, wasm_type)| format!("{}: {}", name, wasm_type_to_zig_type(wasm_type)))
                .collect::<Vec<_>>()
                .join(", "),
            result_type,
        );

        let result_type = match func.results {
            Results::Named(_) => todo!(),
            Results::Anon(t) => wit_type_to_zig_type(t),
        };
        uwriteln!(
            self.src,
            "pub fn {}({}) {} {{",
            func.name.to_snake_case(),
            func.params
                .iter()
                .map(|(name, wit_type)| format!("{}: {}", name, wit_type_to_zig_type(*wit_type)))
                .collect::<Vec<_>>()
                .join(", "),
            result_type,
        );

        let mut func_bindgen = FunctionBindgen::new(self.resolve, &func.name);
        for (param_name, _param_type) in func.params.iter() {
            func_bindgen.params.push(param_name.clone());
        }
        wit_bindgen_core::abi::call(
            self.resolve,
            AbiVariant::GuestImport,
            LiftLower::LowerArgsLiftResults,
            func,
            &mut func_bindgen,
        );
        self.src.push_str(&func_bindgen.src);

        uwriteln!(self.src, "}}");
    }
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
        Type::U32 => todo!(),
        Type::U64 => todo!(),
        Type::S8 => todo!(),
        Type::S16 => todo!(),
        Type::S32 => "i32",
        Type::S64 => todo!(),
        Type::Float32 => todo!(),
        Type::Float64 => todo!(),
        Type::Char => todo!(),
        Type::String => todo!(),
        Type::Id(_) => todo!(),
    }
}

impl<'a> wit_bindgen_core::InterfaceGenerator<'a> for InterfaceGenerator<'a> {
    fn resolve(&self) -> &'a Resolve {
        todo!()
    }

    fn type_record(&mut self, id: TypeId, name: &str, record: &Record, docs: &Docs) {
        todo!()
    }

    fn type_resource(&mut self, id: TypeId, name: &str, docs: &Docs) {
        todo!()
    }

    fn type_flags(&mut self, id: TypeId, name: &str, flags: &Flags, docs: &Docs) {
        todo!()
    }

    fn type_tuple(&mut self, id: TypeId, name: &str, flags: &Tuple, docs: &Docs) {
        todo!()
    }

    fn type_variant(&mut self, id: TypeId, name: &str, variant: &Variant, docs: &Docs) {
        todo!()
    }

    fn type_option(&mut self, id: TypeId, name: &str, payload: &Type, docs: &Docs) {
        todo!()
    }

    fn type_result(&mut self, id: TypeId, name: &str, result: &Result_, docs: &Docs) {
        todo!()
    }

    fn type_enum(&mut self, id: TypeId, name: &str, enum_: &Enum, docs: &Docs) {
        todo!()
    }

    fn type_alias(&mut self, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        todo!()
    }

    fn type_list(&mut self, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        todo!()
    }

    fn type_builtin(&mut self, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        todo!()
    }
}

struct FunctionBindgen<'a> {
    params: Vec<String>,
    src: Source,
    sizes: SizeAlign,
    func_to_call: &'a str,
}

impl FunctionBindgen<'_> {
    fn new<'a>(resolve: &Resolve, func_to_call: &'a str) -> FunctionBindgen<'a> {
        let mut sizes = SizeAlign::default();
        sizes.fill(resolve);
        FunctionBindgen {
            params: Vec::new(),
            src: Source::default(),
            sizes,
            func_to_call,
        }
    }
}

impl Bindgen for FunctionBindgen<'_> {
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
            Instruction::StringLower { realloc } => {
                let str = &operands[0];
                results.push(format!("{str}.ptr"));
                results.push(format!("{str}.len"));
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
                    "@\"{}\"({});",
                    self.func_to_call,
                    operands.join(", ")
                );
            }
            Instruction::Return { amt, func } => {
                assert!(*amt <= 1);
                if *amt == 1 {
                    uwriteln!(self.src, "return {};", operands[0]);
                }
            }
            Instruction::I32FromS32 | Instruction::S32FromI32 => {
                results.push(operands[0].clone());
            }
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
