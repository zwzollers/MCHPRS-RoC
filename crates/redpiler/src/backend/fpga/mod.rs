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
use std::thread;
use std::time::Duration;

use fpga::interface::{Interface, FPGACommand};
use fpga::BinaryIterator;

use std::fs::File;
use std::io::prelude::*;

#[derive(Default, Debug)]
pub struct FPGABackend {
    outputs: FPGAOutputs,
    inputs: FPGAInputs,
}

impl JITBackend for FPGABackend {
    fn inspect(&mut self, _pos: BlockPos) {}

    fn reset<W: World>(&mut self, _world: &mut W, _io_only: bool) {}

    fn on_use_block(&mut self, pos: BlockPos) {
        // if let Some(input) = self.inputs.get_mut(pos) {
        //     input.set_state(!input.state);
        //     self.fpga.send_command(FPGACommand::SetInputs(input.id, 0, input.state));
        // }  
    }

    fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool) {}

    fn tick(&mut self) {}

    fn flush<W: World>(&mut self, world: &mut W, _io_only: bool) { 
        // self.fpga.send_command(FPGACommand::Capture);
        // self.fpga.send_command(FPGACommand::GetOutupts);
        // let mut output_iter: BinaryIterator = BinaryIterator::new(self.fpga.outputs.clone());
        // for (pos, block) in self.outputs.get_blocks_to_change(&mut output_iter) {
        //     world.set_block(pos, block);
        // }
        // for (pos, block) in self.inputs.get_blocks_to_change() {
        //     world.set_block(pos, block);
        // }
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
        let parameters = format!("parameter\n\tROC_INPUTS={},\n\tROC_OUTPUTS={};", self.inputs.num_inputs, self.outputs.bits).to_owned();

        let mut file = File::create("FPGA/src/parameters.vh").unwrap();
        match file.write_all(parameters.as_bytes()) {
            Err(..) => println!("    Error Writing to file"),
            _ => ()
        }
        assembler::generate_verilog(&graph, "FPGA/src/redstone/RoC.sv");
    }

    fn set_rtps(&mut self, rtps: u32) {
        // self.fpga.send_command(FPGACommand::SetRTPS(rtps));
    }

    fn has_pending_ticks(&self) -> bool {false}
}