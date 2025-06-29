use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::fpga::{compiler::DeviceConfig, interface::{BinaryIterator, Interface}, FPGABackend};
use mchprs_blocks::{blocks::{Block, ButtonFace, Lever, LeverFace, RedstoneWire, RedstoneWireSide, StoneButton, TrapdoorHalf}, BlockDirection, BlockPos};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

impl FPGABackend {
    pub fn from_link_file(link: Linker, path: String, config: DeviceConfig) -> FPGABackend {
        FPGABackend { 
            fpga: Default::default(),
            path: path,
            config: config,
            link: link
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Linker {
    pub name: String,
    pub outputs: Vec<IntfBlock>,
    pub output_bits: u32,
    pub inputs: Vec<IntfBlock>,
    pub input_bits: u32,
}

impl Linker {

    pub fn generate_link_file(&self, path: &Path) {
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let json = to_string_pretty(&self).unwrap();

        let mut file = File::create(path).unwrap();
        match file.write(json.as_bytes()) {
            Err(..) => println!("    Error Writing to file"),
            _ => ()
        }
    }

    pub fn add_block(&mut self, block: Block, pos: BlockPos) {
        if let Some(intf) = IntfBlock::new(block, pos) {
            if IntfBlock::is_input(block) {
                self.input_bits += intf.bit_count() as u32;
                self.inputs.push(intf);
            }
            else {
                self.output_bits += intf.bit_count() as u32;
                self.outputs.push(intf);
            }
        }
    }

    pub fn get_output_bytes(&self) -> usize {
        ((self.output_bits + 7) / 8 )as usize
    }

    pub fn get_input_bytes(&self) -> usize {
        ((self.output_bits + 7) / 8) as usize
    }

    pub fn toggle_input(&mut self, pos: BlockPos) -> (u32, u8, u8) {
        let mut id = 0;
        for input in &mut self.inputs {
            if input.pos == pos {
                input.set_state(!input.state);
                return (id, 0, input.state);
            }
            id += input.bit_count() as u32;
        }
        (0,0,0)
    }

    pub fn get_blocks_to_change(&mut self, data: &mut BinaryIterator) -> Vec<(BlockPos, Block)> {
        let mut res: Vec<(BlockPos, Block)> = Vec::new();
        for output in &mut self.outputs {
            let state = match data.next(output.bit_count()) {
                Some(data) => {
                    data
                }
                None => 0,
            };
            output.set_state(state);
            res.push((output.pos, output.get_block()));
        }
        for input in &self.inputs {
            res.push((input.pos, input.get_block()));
        }
        res
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntfBlock {
    ty: IntfType,
    pos: BlockPos,
    state: u8,
}

impl IntfBlock {
    fn new(block: Block, pos: BlockPos) -> Option<IntfBlock> {
        match block {
            Block::RedstoneLamp {lit:l} => 
                Some(IntfBlock{ 
                    ty: IntfType::Lamp, 
                    pos: pos, 
                    state: if l {1} else {0}
                }),
            Block::IronTrapdoor { facing:f, half:h, powered:p } =>
                Some(IntfBlock{ 
                    ty: IntfType::Trapdoor { facing: f, half: h }, 
                    pos: pos, 
                    state: if p {1} else {0} 
                }),
            Block::RedstoneWire { wire: RedstoneWire{north:RedstoneWireSide::None, east:RedstoneWireSide::None, south:RedstoneWireSide::None, west:RedstoneWireSide::None, power: p} } =>
                Some(IntfBlock{ 
                    ty: IntfType::HexLamp, 
                    pos: pos, 
                    state: p 
                }),
            Block::Lever { lever: Lever { face:f, facing:fa, powered:p } } =>
                Some(IntfBlock{ 
                    ty: IntfType::Lever{face:f, facing:fa}, 
                    pos: pos, 
                    state: if p {1} else {0} 
                }),
            Block::StoneButton { button: StoneButton { face:f, facing:fa, powered:p } } =>
                Some(IntfBlock{ 
                    ty: IntfType::Button{face:f, facing:fa}, 
                    pos: pos, 
                    state: if p {1} else {0} 
                }),
            Block::StonePressurePlate { powered:p } =>
                Some(IntfBlock{ 
                    ty: IntfType::PressurePlate, 
                    pos: pos, 
                    state: if p {1} else {0} 
                }),
            _ => None
        }
    }

    pub fn bit_count(&self) -> u8 {
        match self.ty {
            IntfType::Lamp | 
            IntfType::Trapdoor {..} | 
            IntfType::Lever {..} | 
            IntfType::PressurePlate | 
            IntfType::Button {..} |
            IntfType::BinROM => 
                1,
            IntfType::HexLamp |
            IntfType::HexROM => 
                4,
        }
    }

    pub fn set_state(&mut self, state: u8) {
        let state = match self.ty {
            IntfType::Lamp | 
            IntfType::Trapdoor {..} |
            IntfType::Lever {..} | 
            IntfType::PressurePlate | 
            IntfType::Button {..} | 
            IntfType::BinROM => 
                state & 0x01,
            IntfType::HexROM | 
            IntfType::HexLamp =>
                state & 0x0F,
        };
        self.state = state;
    }

    pub fn get_block(&self) -> Block {
        match self.ty {
            IntfType::Lamp => 
                Block::RedstoneLamp { lit: self.state == 1 },
            IntfType::Trapdoor {facing:f, half:h} => 
                Block:: IronTrapdoor { facing: f, half: h, powered: self.state == 1 },
            IntfType::HexLamp => {
                let n = RedstoneWireSide::None;
                Block::RedstoneWire { wire: RedstoneWire::new(n, n, n, n, self.state) }
            },
            IntfType::Lever { face:f, facing :fa} =>
                Block::Lever { lever: Lever { face:f, facing:fa, powered:self.state == 1 } },
            IntfType::Button { face:f, facing:fa } =>
                Block::StoneButton { button: StoneButton { face:f, facing:fa, powered:self.state == 1 } },
            IntfType::PressurePlate =>
                Block::StonePressurePlate { powered:self.state == 1 },
            IntfType::BinROM =>
                todo!("BinROM"),
            IntfType::HexROM =>
                todo!("BinROM"),
        }
    }

    pub fn is_input (block: Block) -> bool {
        match block {
            Block::RedstoneLamp { .. } | 
            Block::IronTrapdoor { .. } |
            Block::RedstoneWire { .. } =>
                false,
            Block::Lever { .. } |
            Block::StoneButton { .. } |
            Block::StonePressurePlate { .. } =>
                true,
            _ => true
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum IntfType {
    Lamp,
    Trapdoor {facing: BlockDirection, half: TrapdoorHalf},
    HexLamp,
    Lever {face: LeverFace, facing: BlockDirection},
    Button {face: ButtonFace, facing: BlockDirection},
    PressurePlate,
    BinROM, //TODO
    HexROM, //TODO
}