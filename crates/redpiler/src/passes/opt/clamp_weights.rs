use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::compile_graph::CompileGraph;
use crate::passes::Pass;
use crate::CompilerOptions;
use mchprs_world::World;

pub struct ClampWeights;

impl<W: World> Pass<W> for ClampWeights {
    fn run_pass(
        &self,
        _options: CompilerOptions,
        _bounds: (mchprs_blocks::BlockPos, mchprs_blocks::BlockPos),
        data: &mut HashMap<TypeId, Box<dyn Any>>,
    ) {
        let graph = data.get_mut(&TypeId::of::<CompileGraph>()).unwrap().downcast_mut::<CompileGraph>().unwrap();

        graph.retain_edges(|g, edge| g[edge].ss < 15);
    }

    fn status_message(&self) -> &'static str {
        "Clamping weights"
    }

    fn driver_key(&self) -> &'static str {
        "clamp-weights"
    }
}
