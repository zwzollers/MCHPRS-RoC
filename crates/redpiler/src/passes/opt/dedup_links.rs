//! # [`DedupLinks`]
//!
//! This pass removes duplicate edges from the graph, or parallel edges that have higher weight.
//!
//! For example, if two nodes are connected with two links of weights 13 and 15, the link with
//! weight 15 is removed.

use crate::CompilerOptions;
use crate::compile_graph::{CompileGraph, Direction, NodeIdx};
use crate::passes::Pass;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use mchprs_world::World;

pub struct DedupLinks;

impl<W: World> Pass<W> for DedupLinks {
    fn run_pass(
        &self,
        _options: CompilerOptions,
        _bounds: (mchprs_blocks::BlockPos, mchprs_blocks::BlockPos),
        data: &mut HashMap<TypeId, Box<dyn Any>>,
    ) {
        let graph = data.get_mut(&TypeId::of::<CompileGraph>()).unwrap().downcast_mut::<CompileGraph>().unwrap();

        for i in 0..graph.node_bound() {
            let idx = NodeIdx::new(i);
            if !graph.contains_node(idx) {
                continue;
            }

            let mut edges = graph.neighbors(idx, Direction::Incoming).detach();
            while let Some(edge_idx) = edges.next_edge(graph) {
                let edge = &graph[edge_idx];
                let source_idx = graph.edge_endpoints(edge_idx).unwrap().0;

                let mut should_remove = false;
                for other_edge in graph.edges(idx, Direction::Incoming) {
                    if other_edge.id() != edge_idx
                        && other_edge.source() == source_idx
                        && other_edge.weight().ty == edge.ty
                        && other_edge.weight().ss <= edge.ss
                    {
                        should_remove = true;
                        break;
                    }
                }

                if should_remove {
                    graph.remove_edge(edge_idx);
                }
            }
        }
    }

    fn status_message(&self) -> &'static str {
        "Deduplicating links"
    }

    fn driver_key(&self) -> &'static str {
        "dedup-links"
    }
}
