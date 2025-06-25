pub mod fpga;

use fpga::FPGA;

pub struct FPGAServer {
    hardware: Vec<FPGA>
}