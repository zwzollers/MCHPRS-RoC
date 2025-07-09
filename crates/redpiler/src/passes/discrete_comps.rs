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

            match graph[start_idx].ty {
                NodeType::Comparator { mode, far_input:_, facing_diode:_ } => {
                    let mut side_states = 1_u16;
                    let mut back_states = 1_u16;

                    for edge in graph.edges_directed(start_idx, petgraph::Direction::Incoming) {
                        let weight = edge.weight(); 
                        let source = &graph[edge.source()].ty;

                        let mut temp_states = 0_u16;
                        let mut const_input = false;

                        match source {
                            NodeType::Repeater {..} |
                            NodeType::Torch |
                            NodeType::Button |
                            NodeType::Lever |
                            NodeType::PressurePlate => {
                                temp_states = 0x8000 >> weight.ss;
                            }
                            NodeType::Comparator {..} |
                            NodeType::FPGAComparator {..} => {
                                temp_states = 0xFFFF >> (weight.ss);
                            }
                            NodeType::Constant => {
                                const_input = true;
                                temp_states = 0x8000 >> weight.ss;
                            }
                            _ => {} 
                        } 

                        match weight.ty {
                            LinkType::Default => {
                                if const_input {
                                    back_states &= 0xFFFE;
                                }
                                back_states |= temp_states;
                            }
                            LinkType::Side => {
                                if const_input {
                                    back_states &= 0xFFFE;
                                }
                                side_states |= temp_states;
                            }
                        }
                    }

                    let mut states = 0_u16;

                    match mode {
                        ComparatorMode::Compare => {
                            let magic_mask = !((side_states - 1) & !side_states);
                            states = back_states & magic_mask;
                        }
                        ComparatorMode::Subtract => {
                            for i in 0..15 {
                                let bit = (side_states >> i) & 0x01;
                                if bit == 1 {
                                    states |= back_states >> i;
                                }
                            }
                        }
                    }
                    graph[start_idx].ty = NodeType::FPGAComparator { 
                        mode: mode,
                        outputs: states,
                        side: side_states,
                        back: back_states,
                    };
                }
                _ => {}
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

