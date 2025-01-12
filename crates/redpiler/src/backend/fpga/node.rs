use std::collections::HashMap;

use fpga::BinaryIterator;
use mchprs_blocks::{blocks::{Block, ButtonFace, Lever, LeverFace, RedstoneWire, RedstoneWireSide, StoneButton, TrapdoorHalf}, BlockDirection, BlockPos};

#[derive(Default, Debug)]
pub struct FPGAOutputs {
    pub outputs: Vec<Output>,
    pub bits: usize,
}

impl FPGAOutputs {
    pub fn add(&mut self, block: Block, pos: BlockPos) {
        let out = Output::new(block, pos).unwrap();
        self.bits += out.bit_count() as usize;
        self.outputs.push(out);
    }

    pub fn get_num_bytes(&self) -> usize {
        (self.bits + 7) / 8
    }

    pub fn get_blocks_to_change(&mut self, data: &mut BinaryIterator) -> Vec<(BlockPos, Block)> {
        let mut res: Vec<(BlockPos, Block)> = Vec::new();
        for output in &mut self.outputs {
            let state = data.bits(output.bit_count());
            output.set_state(state);
            res.push((output.pos, output.get_block()));
        }
        res
    }
}

#[derive(Debug)]
pub struct Output {
    ty: OutputType,
    pos: BlockPos,
    state: u8,
}

impl Output {
    fn new(block: Block, pos: BlockPos) -> Option<Output> {
        match block {
            Block::RedstoneLamp {lit:l} => 
                Some(Output{ ty: OutputType::Lamp, pos: pos, state: if l {1} else {0}}),
            Block::IronTrapdoor { facing:f, half:h, powered:p } =>
                Some(Output { ty: OutputType::Trapdoor { facing: f, half: h }, pos: pos, state: if p {1} else {0} }),
            Block::RedstoneWire { wire: RedstoneWire{north:RedstoneWireSide::None, east:RedstoneWireSide::None, south:RedstoneWireSide::None, west:RedstoneWireSide::None, power: p} } =>
                Some(Output { ty: OutputType::HexLamp, pos: pos, state: p }),
            _ => None
        }
    }

    pub fn bit_count(&self) -> u8 {
        match self.ty {
            OutputType::Lamp | OutputType::Trapdoor {..} => 1,
            OutputType::HexLamp => 4,
        }
    }

    pub fn set_state(&mut self, state: u8) {
        let state = match self.ty {
            OutputType::Lamp | OutputType::Trapdoor {..} => state & 0x01,
            OutputType::HexLamp => state & 0x0F,
        };
        self.state = state;
    }

    pub fn get_block(&self) -> Block {
        match self.ty {
            OutputType::Lamp => 
                Block::RedstoneLamp { lit: self.state == 1 },
            OutputType::Trapdoor {facing:f, half:h} => 
                Block:: IronTrapdoor { facing: f, half: h, powered: self.state == 1 },
            OutputType::HexLamp => {
                let n = RedstoneWireSide::None;
                Block::RedstoneWire { wire: RedstoneWire::new(n, n, n, n, self.state) }
            }
        }
    }
}

#[derive(Debug)]
enum OutputType {
    Lamp,
    Trapdoor {facing: BlockDirection, half: TrapdoorHalf},
    HexLamp
}

#[derive(Default, Debug)]
pub struct FPGAInputs {
    inputs: HashMap<BlockPos, Input>,
    num_inputs: u16,
}

impl FPGAInputs {
    pub fn add(&mut self, block: Block, pos: BlockPos) {
        self.inputs.insert(pos, Input::new(block, self.num_inputs).unwrap());
        self.num_inputs += 1;
    }

    pub fn get_blocks_to_change(&mut self) -> Vec<(BlockPos, Block)> {
        let mut res: Vec<(BlockPos, Block)> = Vec::new();
        for (pos, input) in &mut self.inputs {
            if input.has_changed() {
                res.push((*pos, input.get_block()));
                input.changed = false;
            }
        } 
        res
    }

    pub fn get_mut(&mut self, pos: BlockPos) -> Option<&mut Input> {
        self.inputs.get_mut(&pos)
    }


}

#[derive(Debug)]
pub struct Input {
    ty: InputType,
    pub state: u8,
    changed: bool,
    pub id: u16,
}

impl Input {
    fn new(block: Block, id: u16) -> Option<Input> {
        match block {
            Block::Lever { lever: Lever { face:f, facing:fa, powered:p } } =>
                Some(Input { ty: InputType::Lever{face:f, facing:fa}, state: if p {1} else {0}, changed: true, id:id }),
            Block::StoneButton { button: StoneButton { face:f, facing:fa, powered:p } } =>
                Some(Input { ty: InputType::Button{face:f, facing:fa}, state: if p {1} else {0}, changed: true, id:id }),
            Block::StonePressurePlate { powered:p } =>
                Some(Input { ty: InputType::PressurePlate, state: if p {1} else {0}, changed: true, id:id }),
            _ => None
        }
    }

    pub fn set_state(&mut self, state: u8) {
        self.state = match self.ty {
            InputType::Lever {..} | InputType::PressurePlate | InputType::Button {..} | InputType::BinROM => 
                state & 0x01,
            InputType::HexROM =>
                state & 0x0F,

        };
        self.changed = true;
    }

    pub fn has_changed(&self) -> bool {
        self.changed
    }

    pub fn get_block(&self) -> Block {
        match self.ty {
            InputType::Lever { face:f, facing :fa} =>
                Block::Lever { lever: Lever { face:f, facing:fa, powered:self.state == 1 } },
            InputType::Button { face:f, facing:fa } =>
                Block::StoneButton { button: StoneButton { face:f, facing:fa, powered:self.state == 1 } },
            InputType::PressurePlate =>
                Block::StonePressurePlate { powered:self.state == 1 },
            InputType::BinROM =>
                todo!("BinROM"),
            InputType::HexROM =>
                todo!("BinROM"),
        }
    }
}

#[derive(Debug)]enum InputType {
    Lever {face: LeverFace, facing: BlockDirection},
    Button {face: ButtonFace, facing: BlockDirection},
    PressurePlate,
    BinROM, //TODO
    HexROM, //TODO
}