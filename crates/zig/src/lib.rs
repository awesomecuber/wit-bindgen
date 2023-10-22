use wit_bindgen_core::{
    wit_parser::{Function, InterfaceId, Resolve, TypeId, WorldId, WorldKey},
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
            dbg!(name, func);
        }
        todo!()
    }

    fn export_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        iface: InterfaceId,
        files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn import_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        files: &mut Files,
    ) {
        todo!()
    }

    fn export_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        files: &mut Files,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn import_types(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        types: &[(&str, TypeId)],
        files: &mut Files,
    ) {
        todo!()
    }

    fn finish(&mut self, resolve: &Resolve, world: WorldId, files: &mut Files) {
        todo!()
    }
}
