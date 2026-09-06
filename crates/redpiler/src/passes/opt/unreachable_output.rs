//! # [`UnreachableOutput`]
//!
//! This pass uses the output of SSRangeAnalysis pass to find links that can be removed because the
//! output ss of a node is never higher than the weight of the link.

use crate::CompilerOptions;
use crate::compile_graph::{CompileGraph, Direction, NodeIdx};
use crate::passes::analysis::ss_range_analysis::{SSRangeAnalysis, SSRangeInfo};
use crate::passes::Pass;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use mchprs_world::World;

pub struct UnreachableOutput;

impl<W: World> Pass<W> for UnreachableOutput {
    fn run_pass(
        &self,
        _options: CompilerOptions,
        _bounds: (mchprs_blocks::BlockPos, mchprs_blocks::BlockPos),
        data: &mut HashMap<TypeId, Box<dyn Any>>,
    ) {
        let [range_info, graph] = data.get_disjoint_mut([&TypeId::of::<SSRangeInfo>(), &TypeId::of::<CompileGraph>()]);
        let range_info = range_info.unwrap().downcast_mut::<SSRangeInfo>().unwrap();
        let graph = graph.unwrap().downcast_mut::<CompileGraph>().unwrap();

        for i in 0..graph.node_bound() {
            let idx = NodeIdx::new(i);
            if !graph.contains_node(idx) {
                continue;
            }
            let range = range_info.get_range(idx).unwrap();

            // Now we can go through all the outgoing nodes and remove the ones with a weight that
            // is too high.
            let mut outgoing = graph.neighbors(idx, Direction::Outgoing).detach();
            while let Some((edge_idx, _)) = outgoing.next(graph) {
                if graph[edge_idx].ss >= range.high {
                    graph.remove_edge(edge_idx);
                }
            }
        }
    }

    fn status_message(&self) -> &'static str {
        "Pruning unreachable comparator outputs"
    }

    fn driver_key(&self) -> &'static str {
        "unreachable-output"
    }
}
