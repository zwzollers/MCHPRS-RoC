mod serial;
pub mod interface;
pub mod compiler;

use interface::{DeviceStatus, Interface};
use compiler::{DeviceConfig, CompilerStatus};

pub struct FPGA {
    pub interface: Option<Interface>,
    pub config: Option<DeviceConfig>,
    pub device_status: DeviceStatus,
    pub compiler_status: CompilerStatus,
    pub hold: u32,
}

pub struct RoC {
    pub hardware: Vec<FPGA>,
}

impl RoC {
    pub fn new() -> RoC {
        RoC{ hardware: Vec::new()}
    }
}

pub struct BinaryIterator {
    data: Vec<u8>,
    index: usize,
}

impl BinaryIterator {
    pub fn new (buffer: Vec<u8>) -> BinaryIterator {
        BinaryIterator{ 
            data: (buffer),
            index: (0) 
        }
    }

    pub fn next (&mut self, size: u8) -> Option<u8> {
        
        let len = self.data.len() * 8;
        let res = if self.index + size as usize > len {
            None
        } else {
            Some((self.data[self.index/8] >> (self.index % 8)) & 0x01)
        };

        self.index += size as usize;
        res
    }
}