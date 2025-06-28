use super::Pass;
use crate::compile_graph::{Annotations, CompileGraph, CompileLink, CompileNode, NodeIdx, NodeState, NodeType, LinkType};
use crate::{CompilerInput, CompilerOptions};
use mchprs_blocks::blocks::ComparatorMode;
use mchprs_world::World;
use petgraph::adj::EdgeReference;
use petgraph::data::Build;
use petgraph::visit::{EdgeFiltered, EdgeIndexable, EdgeRef, IntoEdgesDirected, NodeIndexable};
use petgraph::Direction;
use crate::redpiler_graph::Link;

pub struct DiscreteComparators;

struct CompInput {
    states: Vec<InputWeight>,
}

enum InputWeight {
    Default (u8),
    Side (u8),
    Both (u8, u8),
}

impl<W: World> Pass<W> for DiscreteComparators {
    fn run_pass(&self, graph: &mut CompileGraph, _: &CompilerOptions, _: &CompilerInput<'_, W>) {
        let mut starting_nodes : Vec<NodeIdx> = Vec::new();
        
        'next: for i in 0..graph.node_bound() {
            let start_idx = NodeIdx::new(i);
            if !graph.contains_node(start_idx) {
                continue 'next;
            }

            match graph[start_idx].ty {
                NodeType::Comparator { mode, far_input:_, facing_diode:_ } =>
                    search(graph, start_idx, mode),
                _ => {}
            }
        }
    }

    fn status_message(&self) -> &'static str {
        "Converting Comparators into LUTs"
    }
}

fn search (graph: &mut CompileGraph, node: NodeIdx, mode: ComparatorMode) {

    // for neighbor in graph.neighbors_directed(node, Direction::Incoming) {
    //     if matches!(graph[neighbor].ty, NodeType::Comparator { .. }) {
    //         return
    //     }
    // }

    // let mut inputs: Vec<CompInput> = Vec::new();

    // let mut direct_const: u8 = 0;
    // let mut side_const: u8 = 0;

    // for i_node in graph.neighbors_directed(node, Direction::Incoming) {
    //     let links: Vec<&CompileLink> = Vec::new(); 
    //     for i_edge in graph.edges_connecting(i_node, node) {
    //         links.push(i_edge.weight());
    //     }

    //     match inputs
            

    //     } else {
    //         let input =  if links.len() == 2 {
    //             if links[0].ty == LinkType::Default {
    //                 InputWeight::Both(links[0].ss, links[1].ss)
    //             } else {
    //                 InputWeight::Both(links[1].ss, links[0].ss)
    //             } 
    //         } else {
    //             if links[0].ty == LinkType::Default {
    //                 InputWeight::Default(links[0].ss)
    //             } else {
    //                 InputWeight::Side(links[0].ss)
    //             } 
    //         };
    
    //         inputs.push(CompInput{id:i_node, w:input});
    //     }
    // }


    // for i in 0..inputs.len() {
    //     let 
    //     for j in i..inputs.len() {

    //     }
    // }

    // let mut edges: Vec<(NodeIdx, NodeIdx, CompileLink)> = Vec::new();
    // for edge in graph.edges(node) {
    //     let source = edge.source();
    //     let target = edge.target();
    //     let weight = edge.weight();

    //     if source == node {
    //         edges.push((d_comp, target, CompileLink{ss:weight.ss, ty:weight.ty}));
    //     }
    //     else {
    //         edges.push((source, d_comp, CompileLink{ss:weight.ss, ty:weight.ty}));
    //     }
    // }
    // for (o_node,i_node, link) in edges {
    //     graph.add_edge(o_node, i_node, link);
    // }

    // graph.remove_node(node);
}

