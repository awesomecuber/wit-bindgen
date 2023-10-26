use std::fmt::Write;

use wit_bindgen_core::{
    abi::{AbiVariant, Bindgen, Instruction, LiftLower},
    uwrite,
    wit_parser::{Function, InterfaceId, Resolve, SizeAlign, TypeId, WorldId, WorldKey},
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
        files: &mut Files,
    ) {
        let name_raw = &resolve.name_world_key(name);
        self.src
            .push_str(&format!("// Import functions from {name_raw}\n"));

        for (name, func) in resolve.interfaces[iface].functions.iter() {
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
            self.src.push_str(&func_bindgen.src);
        }

        files.push(&format!("{name_raw}.zig"), self.src.as_bytes());
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
        _resolve: &Resolve,
        _world: WorldId,
        _funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) {
        todo!()
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

    fn finish(&mut self, _resolve: &Resolve, _world: WorldId, _files: &mut Files) {
        todo!()
    }
}

struct FunctionBindgen {
    params: Vec<String>,
    src: Source,
    sizes: SizeAlign,
}

impl FunctionBindgen {
    fn new(resolve: &Resolve) -> FunctionBindgen {
        let mut sizes = SizeAlign::default();
        sizes.fill(resolve);
        FunctionBindgen {
            params: Vec::new(),
            src: Source::default(),
            sizes,
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
            Instruction::StringLower { realloc } => {
                let str = &operands[0];
                results.push(format!("{str}.ptr"));
                results.push(format!("{str}.len"));
            }
            Instruction::CallWasm { name, sig } => {
                uwrite!(self.src, "");
            }
            Instruction::Return { amt, func } => {
                dbg!(amt, func);
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

    fn is_list_canonical(
        &self,
        _resolve: &Resolve,
        _element: &wit_bindgen_core::wit_parser::Type,
    ) -> bool {
        todo!()
    }
}
