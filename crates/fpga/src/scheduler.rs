use crate::interface::Interface;
use crate::compiler::DeviceConfig;



#[derive(Default)]
pub struct FPGAScheduler {
    pub fpgas: Vec<FPGA>
}


impl FPGAScheduler {
    pub fn add(&mut self, config: &str, com: &str, program: &str) {
        self.fpgas.push(FPGA { 
            config: config.to_string(),
            com: com.to_string(),
            program: program.to_string() ,
            owner: None
        });
    }

    // pub fn lock(&mut self, x:u32, z:u32) -> Option<(String, String, String)> {
    //     for fpga in &self.fpgas {
    //         if fpga.owner == None {
    //             fpga.owner = Some((x,z));
    //             return Some((fpga.config.clone(),fpga.com.clone(),fpga.program.clone()));
    //         }
            
    //     }
    //     None
    // }
}

pub struct FPGA {
    pub config: String,
    pub com: String,
    pub program: String,
    owner: Option<(u32,u32)>,
}

impl FPGA {

}