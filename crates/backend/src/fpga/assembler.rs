use itertools::Itertools;
use mchprs_blocks::blocks::{Block, ComparatorMode};
use petgraph::graph::NodeIndex;
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

    let mut input_id = 0;
    let mut output_id = 0;

    for nodeid in graph.node_indices() {
        let node = &graph[nodeid];
        let id = nodeid.index();
        let state = node.state.powered;

        if !node.block.is_none() {
            verilog.push_str(&format!("//block pos {},{},{}\n", node.block.unwrap().0.x, node.block.unwrap().0.y, node.block.unwrap().0.z));
        }

        match node.ty {
            NodeType::Lever | NodeType::PressurePlate | NodeType::Button => {
                verilog.push_str(&format!("\twire w{id} = inputs[{input_id}];\n"));
                input_id += 1;
            }
            NodeType::Lamp | NodeType::Trapdoor => {
                verilog.push_str(&format!("\tassign outputs[{output_id}] = ({});\n", 
                    get_inputs_str(graph, id, Some(LinkType::Default)))
                );
                output_id += 1;
            }
            NodeType::Repeater { delay, facing_diode: _ } => {
                verilog.push_str(&format!("\twire w{};\n", id));
                verilog.push_str(&format!("\trepeater #(.t({}), .state(1'b{}), .lock_out({}), .lockable({})) c{} (.i_clk(tick), .i_in({}), .i_lock({}), .o_out(w{}));\n",
                    delay,
                    if state {1} else {0},
                    if is_locker(graph, id) {1} else {0},
                    if is_locking(graph, id) {1} else {0},
                    id,
                    get_inputs_str(graph, id, Some(LinkType::Default)),
                    get_inputs_str(graph, id, Some(LinkType::Side)),
                    id
                ));
            }
            NodeType::Torch => {
                verilog.push_str(&format!("\twire w{};\n", id));
                verilog.push_str(&format!("\ttorch #(.state(1'b{})) c{} (.i_clk(tick), .i_in({}), .o_out(w{}));\n", 
                    if state {0} else {1},
                    id,
                    get_inputs_str(graph, id, Some(LinkType::Default)),
                    id
                ));
            }
            NodeType::Comparator { mode, far_input, facing_diode, states } => {
                let mut s_const = 15;
                let mut b_const = 15;

                let mut s_inputs: Vec<Vec<(usize, Option<u8>)>> = vec![Vec::new(); 16];
                let mut b_inputs: Vec<Vec<(usize, Option<u8>)>> = vec![Vec::new(); 16];

                let mut s_in_cnt = 0;
                let mut b_in_cnt = 0;

                for edge in graph.edges_directed(nodeid, petgraph::Direction::Incoming) {
                    let src = edge.source();
                    let src_id = src.index();
                    let src_node = &graph[src];
                    let weight = edge.weight();
                    let link_ty = weight.ty;
                    let dist = weight.ss as usize;

                    match src_node.ty {
                        NodeType::Repeater {..} |
                        NodeType::Button |
                        NodeType::Lever | 
                        NodeType::Torch | 
                        NodeType::PressurePlate => {
                            match link_ty {
                                LinkType::Default => {
                                    b_inputs[dist].push((src_id, None));
                                    b_in_cnt += 1;
                                }
                                LinkType::Side => {
                                    s_inputs[dist].push((src_id, None));
                                    s_in_cnt += 1;
                                }
                            }
                        }
                        NodeType::Comparator { mode, far_input, facing_diode, states } => {
                            //println!("0b{:b} {:?}", states.unwrap(), states_iter(states.unwrap()));
                            for idx in states_iter(states.unwrap()) {
                                if idx + dist as u8 >= 15 {
                                    continue;
                                } 
                                match link_ty {     
                                    LinkType::Default => b_inputs[idx as usize + dist].push((src_id, Some(15 - idx - dist as u8))),
                                    LinkType::Side    => s_inputs[idx as usize + dist].push((src_id, Some(15 - idx - dist as u8))),
                                }
                            }
                            match link_ty {     
                                LinkType::Default => b_in_cnt += 1,
                                LinkType::Side    => s_in_cnt += 1,
                            }
                        }
                        NodeType::Constant => {
                            let const_dist = 15 - src_node.state.output_strength;
                            match link_ty {     
                                LinkType::Default => b_const = b_const.min(dist + const_dist as usize),
                                LinkType::Side    => s_const = s_const.min(dist + const_dist as usize),
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(far_input_ss) = far_input {
                    b_const = (15 - far_input_ss) as usize;
                    for i in 1..16 {
                        b_inputs[i].clear();
                    }
                }

                println!("states: {:b}, inputs: {b_const},{b_inputs:?} | {s_const}, {s_inputs:?}", states.unwrap());

                let mut b_size = 0;
                let mut b_table: Vec<u8> = Vec::new();
                for dist in 0..b_const {
                    if b_inputs[dist].len() > 0 {
                        b_size += 1; 
                        b_table.push(dist as u8);
                    }
                }
                b_table.push(b_const as u8);
                b_size += 1;
                let mut i = 0;
                verilog.push_str(&format!("\twire[{}:0] w{}_b = {{", b_size-1, id));
                for dist in 0..b_const {
                    if b_inputs[dist].len() == 0 {
                        continue; 
                    }
                    if i > 0 && b_in_cnt > 1 {
                        verilog.push_str(&format!("w{}_b[{}]|", id, b_size-i));
                    }
                    for (src_node, index) in &b_inputs[dist] {
                        if !index.is_none() {
                            verilog.push_str(&format!("w{}[{}]|", src_node, index.unwrap()));
                        }
                        else {
                            verilog.push_str(&format!("w{}|", src_node));
                        }
                    }
                    i += 1;
                    verilog.pop();
                    verilog.push_str(",");
                }
                verilog.push_str("1'b1};\n");


                let mut s_size = 0;
                let mut s_table: Vec<u8> = Vec::new();
                for dist in 0..s_const {
                    if s_inputs[dist].len() > 0 {
                        s_size += 1; 
                        s_table.push(dist as u8);
                    }
                }
                s_table.push(s_const as u8);
                s_size += 1;
                let mut i = 0;
                verilog.push_str(&format!("\twire[{}:0] w{}_s = {{", s_size-1, id));
                for dist in 0..s_const {
                    if s_inputs[dist].len() == 0 {
                        continue; 
                    }
                    if i > 0 && s_in_cnt > 1 {
                        verilog.push_str(&format!("w{}_s[{}]|", id, s_size-i));
                    }
                    for (src_node, index) in &s_inputs[dist] {
                        if !index.is_none() {
                            verilog.push_str(&format!("w{}[{}]|", src_node, index.unwrap()));
                        }
                        else {
                            verilog.push_str(&format!("w{}|", src_node));
                        }
                    }
                    i += 1;
                    verilog.pop();
                    verilog.push_str(",");
                }
                verilog.push_str("1'b1};\n");

                println!("inputs: {b_inputs:?}, {s_inputs:?}");
                println!("input tables: {b_table:?}, {s_table:?}");

                let o_size: usize = states.unwrap().count_ones() as usize - 1;

                verilog.push_str(&format!("\twire[{}:0] w{}_o = {{\n\t\t", o_size, id));

                let mut o_lut: Vec<Vec<(u8, u8)>> = vec![Vec::new(); o_size]; 
                
                match mode {
                    ComparatorMode::Compare => {
                        let mut o_cnt = o_size-1;
                        for o in states_iter(states.unwrap()) {
                            if o >= 15 {
                                continue;
                            }
                            let o_dist = 15 - o as u8 - 1;
                            'b: for b in (0..b_size).rev() {
                                let b_dist = b_table[b];

                                if b_dist + o_dist >= 15 {
                                    //println!("o:{o_dist} b:{b_dist} {b_dist} + {o_dist} >= 15");
                                    continue;
                                }

                                for s in 0..s_size {
                                    let s_dist = s_table[s];

                                    if s_dist >= b_dist {
                                        //println!("o:{o_dist} b:{} s:{} adding to lut", b_size-b-1, s_size-s);
                                        o_lut[o_cnt].push(((b_size-b-1) as u8, (s_size-s) as u8));
                                        break 'b;
                                    }
                                    //println!("o:{o} b:{b} s:{s} {s_dist} <= {b_dist} + {o_dist}");
                                }
                            }
                            if o_cnt == 0 {
                                break;
                            }
                            o_cnt -= 1;
                        }
                    }
                    ComparatorMode::Subtract => {
                        let mut o_cnt = 0;
                        for o in states_iter(states.unwrap()) {
                            if o >= 15 {
                                continue;
                            }
                            let o_dist = o as u8;
                            'b: for b in 0..b_size {
                                let b_dist = b_table[b];

                                for i in 0..o_lut[o_cnt].len() {
                                    if o_lut[o_cnt][i].0 as usize == b_size-b-1 {
                                        continue 'b;
                                    }
                                }

                                if b_dist + o_dist >= 15 {
                                    //println!("\to:{o} b:{b} b_dist + o_dist >= 15");
                                    continue;
                                }

                                for s in 0..s_size {
                                    let s_dist = s_table[s];

                                    if s_dist > b_dist + o_dist {
                                        for i in 0..o_lut[o_cnt].len() {
                                            if o_lut[o_cnt][i].1 as usize == s_size-s {
                                                //println!("\to:{o} b:{} s:{} removing", o_lut[o][i].0, o_lut[o][i].1);
                                                o_lut[o_cnt].remove(i);
                                                break;
                                            }
                                        }
                                        o_lut[o_cnt].push(((b_size-b-1) as u8, (s_size-s) as u8));
                                        println!("\to:{o} b:{} s:{} adding to lut", b_size-b-1, s_size-s);
                                        break;
                                    }
                                    //println!("\to:{o} b:{b} s:{s} {s_dist} <= {b_dist} + {o_dist}");
                                }
                            }
                            o_cnt += 1;
                        }
                    }
                }

                //println!("lut: {o_lut:?}");

                for i in 0..o_size {
                    if (o_lut[(o_size-i-1) as usize].len() == 0 || mode == ComparatorMode::Compare) && i > 0 {
                        verilog.push_str(&format!("w{}_o[{}]|", id, o_size-i+1));
                    }
                    if o_lut[(o_size-i-1) as usize].len() == 0 && i == 0 {
                        verilog.push_str("1'b0|");
                    }
                    for (b_idx, s_idx) in &o_lut[(o_size-i-1) as usize] {
                        if *s_idx >= s_size as u8 {
                            verilog.push_str(&format!("w{}_b[{}]|", id, b_idx));
                        }
                        else {
                            verilog.push_str(&format!("(w{}_b[{}]&~w{}_s[{}])|", id, b_idx, id, s_idx));
                        }
                        
                    }
                    verilog.pop();
                    verilog.push_str(",\n\t\t");
                } 
                verilog.push_str("1'b1\n\t};\n");

                let mut init_str = "".to_string();
                let s = states.unwrap() as u32 >> (16 - node.state.output_strength);

                println!("{} {:b}", node.state.output_strength, s);

                for i in (0..o_size).rev() {
                    
                    if i >= s.count_ones() as usize {
                        init_str.push('0');
                    }
                    else {
                        init_str.push('1');
                    }
                }

                init_str.push('1');

                verilog.push_str(&format!("\twire[{}:0] w{};\n", o_size, id));
                verilog.push_str(&format!("\tcomp #(.size({}), .state({}'b{})) c{} (.i_clk(tick), .i_in(w{}_o), .o_out(w{}));\n",
                    o_size+1,
                    o_size+1,
                    init_str,
                    id,
                    id,
                    id
                ));
            }
            _ => ()
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

fn get_out_idx(states: u16, dist: u8) -> Option<u8> {
    let r_dist = 15 - dist - 1;
    let trimmed = states & ((0x1 << (r_dist+1)) - 1);
    let idx = trimmed.count_ones() as u8;
    if idx == 0 {
        None
    }
    else {
        Some(idx-1)
    }
}

fn states_iter(states: u16) -> Vec<u8> {
    let mut iter: Vec<u8> = Vec::new();
    for i in 0..16 {
        if (states >> i) & 1 == 1 {
            iter.push(i);
        }
    }
    iter
}

fn get_inputs_str (graph: &CompileGraph, node: usize, ty: Option<LinkType>) -> String {
    let mut inputs = "1'b0|".to_owned();
    for edge in graph.edges_directed((node as u32).into(), petgraph::Direction::Incoming) {
        let weight = edge.weight(); 
        if ty == None || weight.ty == ty.unwrap() {
            let src = edge.source();
            let src_node = &graph[src];
            let weight = edge.weight();

            match src_node.ty {
                NodeType::Repeater {..} |
                NodeType::Button |
                NodeType::Lever | 
                NodeType::Torch | 
                NodeType::PressurePlate => {
                    inputs.push_str(&format!("w{}|", src.index()));
                }
                NodeType::Comparator { mode, far_input, facing_diode, states } => {
                    inputs.push_str(&format!("w{}[{}]|", src.index(), ss_to_idx(states.unwrap(), 14-weight.ss)));
                }
                NodeType::Constant => {
                    if src_node.state.output_strength > weight.ss {
                        inputs.push_str("1'b1|");
                    }
                }
                _ => {
                    println!("not input {src_node:?}");
                }
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
        if link.ty == LinkType::Side && matches!(graph[edge.target()].ty, NodeType::Repeater { .. }) {return true}
    }
    false
}

fn ss_to_idx(states: u16, ss: u8) -> u8 {
    let m = states & (0xFFFF_u32 >> (ss+1)) as u16;
    m.count_ones() as u8
}