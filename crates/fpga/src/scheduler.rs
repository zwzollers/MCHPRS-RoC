use mchprs_backend::fpga::compiler::DeviceConfig;
use crate::scheduler;
use std::{fs, path::Path};

#[derive(Default)]
pub struct FPGAScheduler {
    pub fpgas: Vec<FPGA>

}


impl FPGAScheduler {

    pub fn load_from_config(path: &str) -> FPGAScheduler {
        let config_str = fs::read_to_string(path).unwrap();
        let configs: Vec<DeviceConfig> = serde_json::from_str(&config_str).unwrap();

        let mut fpgas: Vec<FPGA> = Vec::new();

        for cfg in configs {
            fpgas.push(FPGA { 
                config: cfg,
                owner: None, 
            });
        }

        return FPGAScheduler {fpgas: fpgas};
    }

    pub fn get_config(&self) -> DeviceConfig {
        self.fpgas[0].config.clone()
    }

    pub fn lock(&mut self, plot: (i32,i32)) -> bool {
        let mut i = 0;
        for fpga in &self.fpgas {
            if fpga.owner == None {
                self.fpgas[i].owner = Some(plot);
                return true;
            }
            i += 1;
        }
        false
    }

    pub fn free(&mut self, plot: (i32, i32)) {
        let mut i = 0;
        for fpga in &self.fpgas {
            if fpga.owner == Some(plot) {
                self.fpgas[i].owner = None;
                break;
            }
            i += 1;
        }
    }
}

pub struct FPGA {
    pub config: DeviceConfig,
    owner: Option<(i32,i32)>,
}
