use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use serde_json;
use serde;


#[derive(serde::Deserialize, Debug)]

pub struct DeviceConfig {
    pub name:           String,
    pub device:         String,
    pub family:         String,
    pub compiler:       String,
    pub version:        String,
    pub logic_elements: u32,
    pub system_clk:     u32,
    pub pin_assignments:PinAssignments
}

#[derive(serde::Deserialize, Debug)]
pub struct PinAssignments {
    i_clk:          String,
    i_rx:           String,
    i_rst:          Option<String>,
    o_tx:           String,
    o_debug:        Option<String>,
    o_tick:         Option<String>
}

impl DeviceConfig {
    pub fn read_config(name: &str) -> Option<DeviceConfig> {
        let config_str = fs::read_to_string("FPGA/config/devices.json").unwrap();
        let configs: Vec<DeviceConfig> = serde_json::from_str(&config_str).unwrap();

        for config in configs {
            if config.name == name {
                return Some(config);
            }
        }
        None
    }   
    pub fn create_project(&self, path: &Path, output_cnt: u32, input_cnt: u32) -> bool {

        let mut tcl = format!(
        "package require ::quartus::project
project_new -overwrite -revision RoC RoC 
set_global_assignment -name FAMILY \"{family}\"
set_global_assignment -name DEVICE {device}
set_global_assignment -name TOP_LEVEL_ENTITY top
set_global_assignment -name ORIGINAL_QUARTUS_VERSION {version}STD.1
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/top.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/interface/uart.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/interface/clk_div.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/interface/command_controller.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../redstone.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/redstone/components.sv
set_global_assignment -name SYSTEMVERILOG_FILE ../../../../src/redstone/tps_clk_div.sv
set_global_assignment -name SOURCE_FILE ../../../../ip/tick_clk.cmp
set_global_assignment -name QIP_FILE ../../../../ip/tick_clk.qip
set_global_assignment -name SIP_FILE ../../../../ip/tick_clk.sip
set_parameter -name ROC_OUTPUTS {output_cnt}
set_parameter -name ROC_INPUTS {input_cnt}
set_location_assignment PIN_{i_rx} -to i_RX
set_location_assignment PIN_{o_tx} -to o_TX
set_location_assignment PIN_{i_clk} -to i_clk\n",
        device = self.device,
        family = self.family,
        version = self.version,
        output_cnt = output_cnt,
        input_cnt = input_cnt,
        i_rx = self.pin_assignments.i_rx,
        o_tx = self.pin_assignments.o_tx,
        i_clk = self.pin_assignments.i_clk,
        );

        if let Some(i_rst) = &self.pin_assignments.i_rst {
            tcl.push_str(&format!("set_location_assignment PIN_{} -to i_rst\n", i_rst));
        }
        if let Some(o_tick) = &self.pin_assignments.o_tick {
            tcl.push_str(&format!("set_location_assignment PIN_{} -to o_tick\n", o_tick));
        }
        if let Some(o_debug) = &self.pin_assignments.o_debug {
            tcl.push_str(&format!("set_location_assignment PIN_{} -to o_debug\n", o_debug));
        }

        tcl.push_str("export_assignments\nproject_close\n");

        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let mut file = File::create(path).unwrap();
        match file.write_all(tcl.as_bytes()) {
            Err(..) => return false,
            _ => ()
        };

        env::set_var("quartus_sh", r"C:\intelFPGA_lite\23.1std\quartus\bin64");

        let out = Command::new("cmd")
            .current_dir(prefix)
            .args(&["/C", r"C:\intelFPGA_lite\23.1std\quartus\bin64\quartus_sh -t prj.tcl"])
            .output()
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(&out.stdout));

        return true;
    }

    pub fn compile (&self, path: &Path) -> CompilerResults {
        let results = CompilerResults{state: true};
        let out = Command::new("cmd")
            .current_dir(path)
            .args(&["/C", r"C:\intelFPGA_lite\23.1std\quartus\bin64\quartus_sh --flow compile RoC"])
            .output()
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(&out.stdout));

        results
    }

    pub fn program (&self) -> ProgramResults {
        let results = ProgramResults{};
        let out = Command::new("cmd")
            .current_dir("FPGA/prj")
            .arg("/C")
            .raw_arg(r#"C:\intelFPGA_lite\23.1std\quartus\bin64\quartus_pgm -c "DE-SoC [USB-1]" -m jtag -o "p;RoC.sof@2""#)
            .output()
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(&out.stdout));

        results
    }
}

pub struct CompilerResults {
    pub state: bool
}

pub struct ProgramResults {

}

#[derive(Default, Clone, Copy)]
pub enum CompilerStatus {
    #[default]
    Idle,
    Compiling,
    Programming,
    Failed
}

impl CompilerStatus {
    pub fn to_str(self) -> &'static str {
        match self {
            CompilerStatus::Idle => "§aIdle",
            CompilerStatus::Compiling => "§eCompiling",
            CompilerStatus::Programming => "§9Programming",
            CompilerStatus::Failed => "§cStopped",
        }
    }
}





