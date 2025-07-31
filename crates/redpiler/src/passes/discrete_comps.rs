use std::u16::MAX;

use super::Pass;
use crate::compile_graph::{CompileGraph, LinkType, NodeIdx, NodeType};
use crate::{BackendVariant, CompilerInput, CompilerOptions};
use mchprs_blocks::blocks::ComparatorMode;
use mchprs_world::World;
use petgraph::visit::{EdgeRef, NodeIndexable};
pub struct DiscreteComparators;

impl<W: World> Pass<W> for DiscreteComparators {
    fn run_pass(&self, graph: &mut CompileGraph, _: &CompilerOptions, _: &CompilerInput<'_, W>) {
        for i in 0..graph.node_bound() {
            let start_idx = NodeIdx::new(i);

            if !graph.contains_node(start_idx) {
                continue;
            }

            if let NodeType::Comparator { mode, far_input, facing_diode, states } = graph[start_idx].ty {
                let mut states: u16 = 0x8000; 

                for edge in graph.edges_directed(start_idx, petgraph::Direction::Outgoing) {
                    let weight = edge.weight(); 
                    let output = &graph[edge.target()].ty;

                    match output {
                        NodeType::Repeater {..} |
                        NodeType::Torch |
                        NodeType::Lamp | 
                        NodeType::Trapdoor=> {
                            states |= 0x1 << weight.ss;
                        }
                        NodeType::Comparator {..} => {
                            states |= 0x7FFF >> weight.ss;
                        }
                        _ => {} 
                    } 
                }

                graph[start_idx].ty = NodeType::Comparator {
                    mode,
                    far_input,
                    facing_diode,
                    states: Some(states),
                };
            }   
        }
    }

    fn status_message(&self) -> &'static str {
        "Optimizing Comparators for FPGA"
    }

    fn should_run(&self, options: &CompilerOptions) -> bool {
        options.backend_variant == BackendVariant::FPGA
    }
}

