use std::time::Duration;
use serialport::SerialPort;



pub enum FPGACommand {
    Reset,
    Ping,
    GetOutupts, 
    Capture,
    SetInputs(u32,u8,u8) ,
    SetRTPS(u32), 
    LoadROM(u32,u8), 
    DebugLED,
    FailAck,             
}


#[derive(Default, Debug)]
pub struct Interface {
    pub serial_conn: SerialConnection,
    pub outputs: Vec<u8>,
}

impl Interface {
    pub fn new(name: &str, baud: u32, timeout: u32, outputs: usize) -> Interface{
        Interface{
            serial_conn: SerialConnection::new(name, baud, timeout), 
            outputs: Vec::with_capacity(outputs)
        }
    }

    pub fn serial_start(&mut self, name: &str, baud: u32) {
        self.serial_conn = SerialConnection::new(name, baud, 20);
        self.serial_conn.start();
    }

    pub fn send_command(&mut self, cmd: FPGACommand) -> bool {

        self.serial_conn.clear_buffer();

        let bytes: Vec<u8> = match cmd {
            FPGACommand::Reset =>                                
                vec![0xC0,0,0,0,0,0xA5],
            FPGACommand::Ping =>                                 
                vec![0xC1,0,0,0,0,0xA5],
            FPGACommand::GetOutupts =>                           
                vec![0xC2,0,0,0,0,0xA5], 
            FPGACommand::Capture =>                              
                vec![0xC3,0,0,0,0,0xA5],
            FPGACommand::SetInputs(id,ty,state)  => 
                vec![0xC4,
                    (id>>16 & 0xFF) as u8,
                    (id>>8 & 0xFF) as u8,
                    (id & 0xFF) as u8,
                    ((ty & 0x01 << 7) | (state & 0x0F)) as u8,
                    0xA5],
            FPGACommand::SetRTPS(rtps) =>                   
                vec![0xC5,
                    (rtps>>24 & 0xFF) as u8,
                    (rtps>>16 & 0xFF) as u8,
                    (rtps>>8 & 0xFF) as u8,
                    (rtps & 0xFF) as u8,
                    0xA5],
            FPGACommand::LoadROM(addr,data) =>          
                vec![0xC6,
                    (addr>>16 & 0xFF) as u8,
                    (addr>>8 & 0xFF) as u8,
                    (addr & 0xFF) as u8,
                    data,
                    0xA5],
            FPGACommand::DebugLED =>                             
                vec![0xC7,0,0,0,0,0xA5],
            FPGACommand::FailAck =>                              
                vec![0xC8,0,0,0,0,0xA5],
        };
        self.serial_conn.write(&bytes);

        let mut response: Vec<u8> = vec![0; 6];
        self.serial_conn.read(&mut response); 

        if response != bytes {
            //println!("CMD doesnt match");
            return false;
        }

        match cmd {
            FPGACommand::GetOutupts => 
                self.serial_conn.read(&mut self.outputs),                      
            _ => false
        }        
    }

    pub fn stop(&mut self) {
        self.serial_conn.stop();
    }
}


pub struct BinaryIterator {
    pub data: Vec<u8>,
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


#[derive(Default, Debug)]
pub struct SerialConnection {
    port_name: String,
    baud_rate: u32,
    timeout: u32,
    conn: Option<Box<dyn SerialPort>>,

}

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

    pub fn stop (&mut self) {
        drop(self.conn.as_mut().unwrap())
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

#[derive(Default, Clone, Copy, PartialEq)]
pub enum DeviceStatus {
    #[default]
    Inactive,
    Programming,
    Connected,
    Disconnected,
    Failed
}

impl DeviceStatus {
    pub fn to_str(self) -> &'static str {
        match self {
            DeviceStatus::Inactive => "&cInactive",
            DeviceStatus::Programming => "&9Programming",
            DeviceStatus::Connected => "&aConnected",
            DeviceStatus::Disconnected => "&cDisconnected",
            DeviceStatus::Failed => "&cStopped",
        }
    }
}