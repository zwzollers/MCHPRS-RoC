use crate::player::{PacketSender, Player};
use mchprs_network::packets::clientbound::{
    CDisplayObjective, CResetScore, CUpdateObjectives, CUpdateScore, ClientBoundPacket,
    ObjectiveNumberFormat,
};
use mchprs_redpiler::{BackendVariant, CompilerOptions};
use mchprs_text::{ColorCode, TextComponentBuilder};
use fpga::{compiler::CompilerStatus, interface::DeviceStatus};

#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum RedpilerState {
    #[default]
    Stopped,
    Compiling,
    Running,
}

impl RedpilerState {
    fn to_str(self) -> &'static str {
        match self {
            RedpilerState::Stopped => "§cStopped",
            RedpilerState::Compiling => "§eCompiling",
            RedpilerState::Running => "§aRunning",
        }
    }
}

#[derive(Default)]
pub struct Scoreboard {
    pub redpiler_state: RedpilerState,
    pub redpiler_options: CompilerOptions,
    pub fpga_compiler_state: CompilerStatus,
    pub fpga_device_name: String,
    pub fpga_ping: u32,
    pub fpga_device_state: DeviceStatus,

    current_state: Vec<String>,
    pub changed: bool
}

impl Scoreboard {
    pub fn new() -> Scoreboard{
        let mut sb: Scoreboard = Default::default();
        sb.current_state = sb.to_str_vec();
        sb
    }
    
    fn to_str_vec(&self) -> Vec<String> {
        let mut state_str: Vec<String> = Vec::new();

        state_str.push(format!("§fRedpiler: §a§l{}",self.redpiler_state.to_str()));

        let mut flags = Vec::new();
        if self.redpiler_options.optimize {
            flags.push("    §9- optimize");
        }
        if self.redpiler_options.export {
            flags.push("    §9- export");
        }
        if self.redpiler_options.io_only {
            flags.push("    §9- io only");
        }
        if self.redpiler_options.update {
            flags.push("    §9- update");
        }
        if self.redpiler_options.wire_dot_out {
            flags.push("    §9- wire dot out");
        }
        if self.redpiler_options.selection {
            flags.push("    §9- selection only");
        }
        if self.redpiler_options.backend_variant == BackendVariant::FPGA {
            flags.push("    §9- FPGA");
        }

        state_str.extend(flags.iter().map(|s| s.to_string()));
        state_str.push(format!("§fQuartus: {}",self.fpga_compiler_state.to_str()));
        state_str.push("----------".to_string());
        state_str.push(format!("§fFPGA: {}",self.fpga_device_state.to_str()));
        if self.fpga_device_state != DeviceStatus::Inactive {
            state_str.push(format!("§7  device: §a{}",self.fpga_device_name.clone()));
            state_str.push(format!("§7  ping: §a{}us",self.fpga_ping));
            state_str.push(format!("§7  utilization: §a22%"));
        }
        

        state_str
    }

    pub fn update (&mut self, players: &[Player]) {
        if self.changed {
            self.changed = false;
            self.set_lines(players, self.to_str_vec());
        }
        
    }

     fn make_update_packet(&self, line: usize) -> CUpdateScore {
        CUpdateScore {
            entity_name: self.current_state[line].clone(),
            objective_name: "status".to_string(),
            value: (self.current_state.len() - line) as i32,
            display_name: None,
            number_format: None,
        }
    }

    fn make_removal_packet(&self, line: usize) -> CResetScore {
        CResetScore {
            entity_name: self.current_state[line].clone(),
            objective_name: Some("status".to_string()),
        }
    }

    fn set_lines(&mut self, players: &[Player], lines: Vec<String>) {
        for line in 0..self.current_state.len() {
            let removal_packet = self.make_removal_packet(line).encode();
            players.iter().for_each(|p| p.send_packet(&removal_packet));
        }

        self.current_state = lines;

        for line in 0..self.current_state.len() {
            let update_packet = self.make_update_packet(line).encode();
            players.iter().for_each(|p| p.send_packet(&update_packet));
        }
    }

    pub fn add_player(&self, player: &Player) {
        player.send_packet(
            &CUpdateObjectives {
                objective_name: "status".into(),
                mode: 0,
                objective_value: TextComponentBuilder::new("Status".into())
                    .color_code(ColorCode::White)
                    .finish(),
                ty: 0,
                number_format: Some(ObjectiveNumberFormat::Blank),
            }
            .encode(),
        );
        player.send_packet(
            &CDisplayObjective {
                position: 1,
                score_name: "status".into(),
            }
            .encode(),
        );
        for i in 0..self.current_state.len() {
            player.send_packet(&self.make_update_packet(i).encode());
        }
    }

    pub fn remove_player(&mut self, player: &Player) {
        for i in 0..self.current_state.len() {
            player.send_packet(&self.make_removal_packet(i).encode());
        }
    }
}