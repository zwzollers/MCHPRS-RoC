use mchprs_blocks::{blocks::Block, BlockPos};
use petgraph::visit::EdgeRef;
use crate::compile_graph::{CompileGraph, LinkType, NodeType};
use std::fs::File;
use std::io::prelude::*;

pub fn generate_verilog(graph: &CompileGraph, path: &str) {

    let mut verilog = 
    "module redstone (tick, inputs, outputs);
        \tinput tick;
        \tinput [num_inputs-1:0] inputs;
        \toutput [num_outputs:0] outputs;\n
        
    parameter num_outputs = 1, num_inputs = 1;\n\n".to_owned();

    let mut input_count = 0;
    let mut output_count = 0;

    for nodeid in graph.node_indices() {
        let node = &graph[nodeid];
        let id = nodeid.index();
        if let Some((pos, blockid)) = node.block {
            let block = Block::from_id(blockid);
            let state = node.state.powered;

            verilog.push_str(&format!("\twire w{};\n", hash_pos(pos)));
            match node.ty
            {
                NodeType::Lever | NodeType::PressurePlate | NodeType::Button =>
                {
                    verilog.push_str(&format!("\tassign w{} = inputs[{input_count}];\n", 
                        hash_pos(pos)));
                    input_count += 1;
                }
                NodeType::Lamp | NodeType::Trapdoor =>
                {
                    verilog.push_str(&format!("\tassign outputs[{output_count}] = ({});\n", 
                        get_inputs_str(graph, id, Some(LinkType::Default))));
                    output_count += 1;
                }
                NodeType::Repeater { delay, facing_diode } =>
                {
                    
                    verilog.push_str(&format!("\trepeater #({}, 1'b{}, {}, {}) c{} (.i_clk(tick), .i_in({}), .i_lock({}), .o_out(w{}));\n",
                        delay,
                        if state {1} else {0},
                        if is_locker(graph, id) {1} else {0},
                        if is_locking(graph, id) {1} else {0},
                        hash_pos(pos),
                        get_inputs_str(graph, id, Some(LinkType::Default)),
                        get_inputs_str(graph, id, Some(LinkType::Side)),
                        hash_pos(pos)));
                }
                NodeType::Torch =>
                {
                    verilog.push_str(&format!("\ttorch #(1'b{}) c{} (.i_clk(tick), .i_in({}), .o_out(w{}));\n", 
                        if state {1} else {0},
                        hash_pos(pos),
                        get_inputs_str(graph, id, Some(LinkType::Default)),
                        hash_pos(pos)));
                }
                NodeType::Comparator {..} =>
                {
                    todo!("Do Comparators");
                }
                _ => ()
            }
        }
    }
    verilog.push_str("endmodule");
    let mut file = File::create(path).unwrap();
    match file.write_all(verilog.as_bytes())
    {
        Err(..) => println!("    Error Writing to file"),
        _ => ()
    }
}

fn hash_pos (pos: BlockPos) -> String {
    format!("{}_{}_{}", pos.x, pos.y, pos.z).to_string()
}

pub fn get_inputs_str (graph: &CompileGraph, node: usize, ty: Option<LinkType>) -> String
{
    let mut inputs = "".to_owned();
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming)
    {
        let link = &graph[edge.id()];

        if ty == None || link.ty == ty.unwrap() {
            let s_node = &graph[edge.source()];
            inputs.push('w');
            inputs.push_str(&hash_pos(s_node.block.unwrap().0));
            inputs.push('|');
        }
    }
    inputs.pop();
    inputs
}

pub fn is_locking (graph: &CompileGraph, node: usize) -> bool
{
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming)
    {
        let link = &graph[edge.id()];
        if link.ty == LinkType::Side {return true} 
    }
    false
}

pub fn is_locker (graph: &CompileGraph, node: usize) -> bool
{
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Outgoing)
    {
        let link = &graph[edge.id()];
        if link.ty == LinkType::Side {return true}
    }
    false
}



