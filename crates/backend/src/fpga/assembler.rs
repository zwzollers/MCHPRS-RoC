use mchprs_blocks::blocks::{Block, ComparatorMode};
use petgraph::visit::EdgeRef;
use mchprs_redpiler::compile_graph::{CompileGraph, LinkType, NodeType};
use std::any::Any;
use std::cmp::max;
use std::collections::HashMap;
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
                NodeType::FPGAComparator { mode, outputs, side, back } => {
                    verilog.push_str(&comp_to_str(graph, id,  back, side, outputs));
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
            let src = edge.source();
            let src_node = &graph[src].ty;
            let weight = edge.weight();

            match src_node {
                NodeType::Repeater {..} |
                NodeType::Button |
                NodeType::Lever | 
                NodeType::Torch | 
                NodeType::PressurePlate => {
                    inputs.push_str(&format!("w{}|", src.index()));
                }
                NodeType::FPGAComparator {outputs, side:_, back:_, mode:_} => {
                    inputs.push_str(&format!("w{}[{}]|", src.index(), ss_to_idx(*outputs, 14-weight.ss)));
                }
                _ => {}
            }
            
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

fn comp_to_str(graph: &CompileGraph, node: usize, back: u16, side: u16, out: u16) -> String {
    let mut verilog: String = "".to_string();

    let s_size: usize = side.count_ones() as usize;
    let mut s_inputs: Vec<Vec<(usize, Option<u8>)>> = vec![Vec::new(); s_size];

    let b_size: usize = back.count_ones() as usize;
    let mut b_inputs: Vec<Vec<(usize, Option<u8>)>> = vec![Vec::new(); b_size];

    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming) {
        let src = edge.source();
        let src_node = &graph[src].ty;
        let weight = edge.weight();
        let in_dir = weight.ty;

        match src_node {
            NodeType::Repeater {..} |
            NodeType::Button |
            NodeType::Lever | 
            NodeType::Torch | 
            NodeType::PressurePlate => {
                if in_dir == LinkType::Default {
                    b_inputs[ss_to_idx(back, weight.ss) as usize].push((src.index(), None));
                }
                else {
                    s_inputs[ss_to_idx(side, weight.ss) as usize].push((src.index(), None));
                }
            }
            NodeType::FPGAComparator {outputs, side:_, back:_, mode:_} => {
                for i in weight.ss..16 {
                    if ((outputs << (i-weight.ss)) & 0x8000) == 0x8000 {
                        let src_idx = ss_to_idx(*outputs, i-weight.ss);
                        if in_dir == LinkType::Default {
                            b_inputs[ss_to_idx(back, i) as usize].push((src.index(), Some(src_idx)));
                        }
                        else {
                            s_inputs[ss_to_idx(side, i) as usize].push((src.index(), Some(src_idx)));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if b_inputs.len() > 0 {
        verilog.push_str(&format!("\twire[{}:0] w{}_b = {{", b_size-1, node));
        for i in 0..b_size {
            if b_inputs[(b_size-i-1) as usize].len() == 0 {
                verilog.push_str("1'b0|");
            }
            if i > 0 {
                verilog.push_str(&format!("w{}_b[{}]|", node, b_size-i));
            }
            for (src_node, index) in &b_inputs[(b_size-i-1) as usize] {
                
                if !index.is_none() {
                    verilog.push_str(&format!("w{}[{}]|", src_node, index.unwrap()));
                }
                else {
                    verilog.push_str(&format!("w{}|", src_node));
                }
            }
            verilog.pop();
            verilog.push_str(",");
        }
        verilog.pop();
        verilog.push_str("};\n");
    }

    if s_inputs.len() > 0 {
        verilog.push_str(&format!("\twire[{}:0] w{}_s = {{", s_size-1, node));
        for i in 0..s_size {
            if s_inputs[(s_size-i-1) as usize].len() == 0 {
                verilog.push_str("1'b0|");
            }
            if i > 0 {
                verilog.push_str(&format!("w{}_s[{}]|", node, s_size-i));
            }
            for (src_node, index) in &s_inputs[(s_size-i-1) as usize] {
                
                if !index.is_none() {
                    verilog.push_str(&format!("w{}[{}]|", src_node, index.unwrap()));
                }
                else {
                    verilog.push_str(&format!("w{}|", src_node));
                }
            }
            verilog.pop();
            verilog.push_str(",");
        }
        verilog.pop();
        verilog.push_str("};\n");
    }


    let b_table = get_index_table(back);
    let s_table = get_index_table(side);

    let o_size: usize = out.count_ones() as usize;
    let mut outputs: Vec<Vec<(u8, u8)>> = vec![Vec::new(); o_size]; 

    for i in 0..b_size {
        for j in 0..s_size {
            if b_table[i] > s_table[j] {
                outputs[ss_to_idx(out, 15 - b_table[i] + s_table[j]) as usize].push((i as u8, j as u8));
            } 
        }
    }

    verilog.push_str(&format!("\twire[{}:0] w{} = {{", o_size-1, node));
    for i in 0..o_size {
        if outputs[(o_size-i-1) as usize].len() == 0 {
            verilog.push_str("1'b0|");
        }
        if i > 0 {
            verilog.push_str(&format!("w{}[{}]|", node, o_size-i));
        }
        for (b_idx, s_idx) in &outputs[(o_size-i-1) as usize] {
            verilog.push_str(&format!("(w{}_s[{}]&~w{}_s[{}]&w{}_b[{}])|", node, s_idx, node, s_idx+1, node, b_idx));
        }
        verilog.pop();
        verilog.push_str(",");
    }
    verilog.pop();
    verilog.push_str("};\n");

    verilog
}

fn get_index_table(states: u16) -> Vec<u8> {
    let mut table: Vec<u8> = Vec::new();

    for i in 0..16 {
        if ((states >> i) & 1) == 1 {
            table.push(i);
        }
    }

    table
}

fn ss_to_idx(states: u16, ss: u8) -> u8 {
    let m = states & (0xFFFF >> (ss+1));
    m.count_ones() as u8
}