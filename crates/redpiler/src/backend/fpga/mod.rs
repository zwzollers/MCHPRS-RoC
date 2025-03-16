mod assembler;
mod node;

use super::JITBackend;
use crate::compile_graph::{CompileGraph, NodeType};
use crate::task_monitor::TaskMonitor;
use crate::CompilerOptions;
use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::World;
use mchprs_world::TickEntry;
use node::{FPGAInputs, FPGAOutputs};
use std::sync::Arc;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Error, ErrorKind};

use fpga::FPGAInterface;

#[derive(Default, Debug)]
pub struct FPGABackend {
    fpga: FPGAInterface,
    outputs: FPGAOutputs,
    inputs: FPGAInputs,
}

impl JITBackend for FPGABackend {
    fn inspect(&mut self, _pos: BlockPos) {

    }

    fn reset<W: World>(&mut self, _world: &mut W, _io_only: bool) {
        self.inputs = FPGAInputs::default();
        self.outputs = FPGAOutputs::default();
        self.fpga.reset();
    }

    fn on_use_block(&mut self, pos: BlockPos) {
        if let Some(input) = self.inputs.get_mut(pos) {
            input.set_state(!input.state);
            self.fpga.set_input_state(input.id, input.state);
        }  
    }

    fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool) {
        
    }

    fn tick(&mut self) {

    }

    fn flush<W: World>(&mut self, world: &mut W, _io_only: bool) { 
        let data  = &mut self.fpga.get_output_data(self.outputs.get_num_bytes()).unwrap().into_iter();
        for (pos, block) in self.outputs.get_blocks_to_change(data) {
            world.set_block(pos, block);
        }
        for (pos, block) in self.inputs.get_blocks_to_change() {
            world.set_block(pos, block);
        }
    }

    fn compile(
        &mut self,
        graph: CompileGraph,
        _ticks: Vec<TickEntry>,
        options: &CompilerOptions,
        _monitor: Arc<TaskMonitor>,
    ) {
        for nodeid in graph.node_indices() {
            let node = &graph[nodeid];
            if let Some((pos, blockid)) = node.block {
                let block = Block::from_id(blockid);
                match node.ty
                {
                    NodeType::Lamp | NodeType::Trapdoor => 
                        self.outputs.add(block, pos),
                    NodeType::Lever | NodeType::Button | NodeType::PressurePlate => 
                        self.inputs.add(block, pos),
                    _ => ()
                }
            }
        }
        
        println!("{:#?}", graph);

        if !options.compile_verilog {
            assembler::generate_verilog(&graph, "../../FPGA/Quartus/Verilog/redstone.v");

            let stdout = Command::new("cmd")
                .args(&["/C", r"..\..\FPGA\Quartus\Commands\Windows\compile"])
                .stdout(Stdio::piped())
                .spawn().unwrap()
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output.")).unwrap();

            let reader = BufReader::new(stdout);
            
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}", line));


            let stdout = Command::new("cmd")
                .args(&["/C", r"..\..\FPGA\Quartus\Commands\Windows\program"])
                .stdout(Stdio::piped())
                .spawn().unwrap()
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output.")).unwrap();

            let reader = BufReader::new(stdout);
            
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}", line));

            self.fpga.serial_start("COM4", 2500000);
        }
    }

    fn has_pending_ticks(&self) -> bool {false}
}