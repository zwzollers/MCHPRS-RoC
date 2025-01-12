use crate::SerialBuffer;

use super::{FPGAInterface, SerialConnection, SerialCommands};

impl FPGAInterface {
    pub fn new(name: &str, baud: u32, timeout: u32) -> FPGAInterface{
        FPGAInterface{serial_conn: SerialConnection::new(name, baud, timeout)}
    }

    pub fn get_output_data(&mut self, bytes: usize) -> SerialBuffer
    {
        let mut buffer: SerialBuffer = SerialBuffer::new(bytes);
        self.serial_conn.write(SerialCommands::RequestOutputs as u8);
        self.serial_conn.read(&mut buffer.buf);
        self.serial_conn.clear_buffer();
        buffer
    }

    pub fn set_input_state(&mut self, id: u16, state: u8) -> bool
    {
        self.serial_conn.write(SerialCommands::UpdateLever as u8) &
        self.serial_conn.write(((id >> 8) & 0xFF) as u8) &
        self.serial_conn.write((id & 0xFF) as u8) &
        self.serial_conn.write(state)
    }

    pub fn reset(&mut self) -> bool
    {
        //self.serial_conn.write(SerialCommands::Reset as u8)
        true
    }

    pub fn ping(&mut self) -> bool
    {
        self.serial_conn.write(SerialCommands::Ping as u8);
        let res: &mut Vec<u8> = &mut vec![0];
        self.serial_conn.read(res);

        res[0] == SerialCommands::Ping as u8
    }
}