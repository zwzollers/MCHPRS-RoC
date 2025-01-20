use std::time::Duration;

use super::SerialConnection;

impl SerialConnection {
    pub fn new (name: &str, baud: u32, timeout: u32) -> SerialConnection {
        SerialConnection{port_name: name.to_string(), baud_rate: baud, timeout: timeout, conn: None}
    }

    pub fn start (&mut self) -> bool{
        self.conn = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(self.timeout as u64))
            .parity(serialport::Parity::None)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .open()
            .ok();

        !self.conn.is_none()
    }

    pub fn read (&mut self, buffer: &mut Vec<u8>) -> bool {
        self.conn.as_mut().unwrap().read_exact(buffer).is_ok()
    }

    pub fn clear_buffer (&mut self) -> bool {
        self.conn.as_mut().unwrap().clear(serialport::ClearBuffer::Input).is_ok()
    }

    pub fn write (&mut self, data: &Vec<u8>) -> bool {
        self.conn.as_mut().unwrap().write(data).is_ok()
    }

    pub fn write_byte (&mut self, data: u8) -> bool {
        self.conn.as_mut().unwrap().write(&vec![data]).is_ok()
    }
}

