use mchprs_blocks::blocks::Block;
use petgraph::visit::EdgeRef;
use mchprs_redpiler::compile_graph::{CompileGraph, LinkType, NodeType};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn generate_verilog(graph: &CompileGraph, path: &Path) {

    let mut verilog = 
"module RoC #(
    parameter OUTPUTS,
    parameter INPUTS
) (
    input                   tick,
    input   [INPUTS-1:0]    inputs,
    output  [OUTPUTS-1:0]   outputs
);\n\n".to_owned();

    let mut input_count = 0;
    let mut output_count = 0;

    for nodeid in graph.node_indices() {
        let node = &graph[nodeid];
        let id = nodeid.index();
        if let Some((pos, blockid)) = node.block {
            let block = Block::from_id(blockid);
            let state = node.state.powered;

            match node.ty {
                NodeType::Lever | NodeType::PressurePlate | NodeType::Button => {
                    verilog.push_str(&format!("\twire w{id};\n"));
                    verilog.push_str(&format!("\tassign w{id} = inputs[{input_count}];\n"));
                    input_count += 1;
                }
                NodeType::Lamp | NodeType::Trapdoor => {
                    verilog.push_str(&format!("\tassign outputs[{output_count}] = ({});\n", 
                        get_inputs_str(graph, id, Some(LinkType::Default))));
                    output_count += 1;
                }
                NodeType::Repeater { delay, facing_diode: _ } => {
                    verilog.push_str(&format!("\twire w{};\n", id));
                    verilog.push_str(&format!("\trepeater #({}, 1'b{}, {}, {}) c{} (.i_clk(tick), .i_in({}), .i_lock({}), .o_out(w{}));\n",
                        delay,
                        if state {1} else {0},
                        if is_locker(graph, id) {1} else {0},
                        if is_locking(graph, id) {1} else {0},
                        id,
                        get_inputs_str(graph, id, Some(LinkType::Default)),
                        get_inputs_str(graph, id, Some(LinkType::Side)),
                        id));
                }
                NodeType::Torch => {
                    verilog.push_str(&format!("\twire w{};\n", id));
                    verilog.push_str(&format!("\ttorch #(1'b{}) c{} (.i_clk(tick), .i_in({}), .o_out(w{}));\n", 
                        if state {1} else {0},
                        id,
                        get_inputs_str(graph, id, Some(LinkType::Default)),
                        id));
                }
                NodeType::Comparator { mode: _, far_input: _, facing_diode: _} => {

                }
                _ => ()
            }
        }
    }
    verilog.push_str("endmodule");

    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let mut file = File::create(path).unwrap();
    match file.write(verilog.as_bytes()) {
        Err(..) => println!("    Error Writing to file"),
        _ => ()
    }
}

fn get_inputs_str (graph: &CompileGraph, node: usize, ty: Option<LinkType>) -> String {
    let mut inputs = "".to_owned();
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming) {
        let weight = edge.weight(); 
        if ty == None || weight.ty == ty.unwrap() {
            inputs.push_str(&format!("w{}|", edge.source().index()));
        }
    }
    inputs.pop();
    inputs
}

fn is_locking (graph: &CompileGraph, node: usize) -> bool {
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming) {
        let link = &graph[edge.id()];
        if link.ty == LinkType::Side {return true} 
    }
    false
}

fn is_locker (graph: &CompileGraph, node: usize) -> bool {
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Outgoing) {
        let link = &graph[edge.id()];
        if link.ty == LinkType::Side {return true}
    }
    false
}