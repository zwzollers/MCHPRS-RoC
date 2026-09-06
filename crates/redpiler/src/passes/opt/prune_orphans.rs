//! # [`PruneOrphans`]
//!
//! This pass removes any nodes in the graph that aren't transitively connected to an output
//! redstone component by using Depth-First-Search.

use crate::CompilerOptions;
use crate::compile_graph::{CompileGraph, Direction};
use crate::passes::Pass;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use itertools::Itertools;
use mchprs_world::World;

pub struct PruneOrphans;

impl<W: World> Pass<W> for PruneOrphans {
    fn run_pass(
        &self,
        _options: CompilerOptions,
        _bounds: (mchprs_blocks::BlockPos, mchprs_blocks::BlockPos),
        data: &mut HashMap<TypeId, Box<dyn Any>>,
    ) {
        let graph = data.get_mut(&TypeId::of::<CompileGraph>()).unwrap().downcast_mut::<CompileGraph>().unwrap();

        // We start searching from output nodes
        let mut worklist = graph
            .node_indices()
            .filter(|&idx| graph[idx].is_output)
            .collect_vec();

        let mut visited = vec![false; graph.node_bound()];

        // Visit initial nodes
        for &idx in &worklist {
            visited[idx.index()] = true;
        }

        while let Some(idx) = worklist.pop() {
            for incoming in graph.neighbors(idx, Direction::Incoming) {
                if !visited[incoming.index()] {
                    visited[incoming.index()] = true;
                    worklist.push(incoming);
                }
            }
        }

        graph.retain_nodes(|g, idx| {
            let visible = visited[idx.index()];
            let is_input = g[idx].is_input;

            // Retain inputs so they can be updated when used
            visible || is_input
        });
    }

    fn status_message(&self) -> &'static str {
        "Pruning orphans"
    }

    fn driver_key(&self) -> &'static str {
        "prune-orphans"
    }
}
