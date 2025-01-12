mod serial;
mod interface;
mod compiler;
mod verilog;

use serialport::SerialPort;

pub enum SerialCommands {
    UpdateLever = 0x02,
    RequestOutputs = 0x01,
    Reset = 0x5A,
    Ping = 0x33,
}

#[derive(Default, Debug)]
pub struct SerialConnection {
    port_name: String,
    baud_rate: u32,
    timeout: u32,
    conn: Option<Box<dyn SerialPort>>
}

#[derive(Default, Debug)]
pub struct FPGAInterface {
    pub serial_conn: SerialConnection,
}

impl FPGAInterface {
    pub fn serial_start(&mut self, name: &str, baud: u32) {
        self.serial_conn = SerialConnection::new(name, baud, 50);
        self.serial_conn.start();
    }
}

#[derive(Default)]
pub struct SerialBuffer {
    buf: Vec<u8>
}

impl IntoIterator for &SerialBuffer {
    type IntoIter = BinaryIterator;
    type Item = u8;
    fn into_iter(self) -> Self::IntoIter {
        BinaryIterator{data: self.buf.clone(), index:0}
    }
}

impl SerialBuffer {
    pub fn new(size: usize) -> SerialBuffer {
        SerialBuffer{ buf: vec![0; size]}
    }
}

pub struct BinaryIterator {
    pub data: Vec<u8>,
    index: usize
}

impl Iterator for BinaryIterator {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let index = self.index/8;
        let byte=  
            if index < self.data.len() {self.data[index]}
            else {return None};
        let res = Some((byte >> (self.index % 8)) & 0x01);
        self.index += 1;
        res
    }
}

impl BinaryIterator {
    pub fn bits(&mut self, num_bits: u8) -> u8{
        let mut res: u8 = 0;
        for i in 0..num_bits {
            res |= self.next().unwrap() << i;
        }
        res
    }
}