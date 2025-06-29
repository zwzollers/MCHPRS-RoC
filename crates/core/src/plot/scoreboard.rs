use crate::player::{PacketSender, Player};
use mchprs_network::packets::clientbound::{
    CDisplayObjective, CResetScore, CUpdateObjectives, CUpdateScore, ClientBoundPacket,
    ObjectiveNumberFormat,
};
use mchprs_backend::{BackendStatus, BackendMsg};
use mchprs_redpiler::CompilerOptions;
use mchprs_text::{ColorCode, TextComponent, TextComponentBuilder};
use std::collections::HashMap;

#[derive(Default)]
pub struct Scoreboard {
    backend_list: HashMap<String, (CompilerOptions, BackendStatus)>,
    current_state: Vec<String>,
}

impl Scoreboard {
    pub fn new() -> Scoreboard{
        let mut sb: Scoreboard = Default::default();
        sb.current_state = sb.to_str_vec();
        sb
    }

    fn to_str_vec(&self) -> Vec<String> {
        let mut sb: Vec<String> = Vec::new();

        for (name, (options, status)) in &self.backend_list {
            sb.push(format!("&f{:15} {}", name, status.to_str()));
            sb.extend(options.to_str_vec());
        }

        // state_str.push(format!("&fRoC: {}",self.fpga_compiler_state.to_str()));
        
        // if self.fpga_device_state != DeviceStatus::Inactive {
        //     state_str.push(format!("&7  device: &a{}",self.fpga_device_name.clone()));
        //     state_str.push(format!("&7  ping: &a{}us",self.fpga_ping));
        //     state_str.push(format!("&7  utilization: &a22%"));
        // }
        

        sb
    }

    pub fn add_backend(&mut self, name: String, options: CompilerOptions) {
        self.backend_list.insert(name, (options, BackendStatus::Redpiling));
    }

    pub fn update (&mut self, players: &[Player]) {
        self.set_lines(players, self.to_str_vec());
        
    }

    pub fn parse_scoreboard_msg (&mut self, msg: BackendMsg) {
        match msg {
            BackendMsg::New { backend, options} => {
                self.add_backend(backend.clone(), options);
                 self.backend_list.get_mut(&backend).unwrap().1 = BackendStatus::Redpiling;
            }
            BackendMsg::Delete { backend} => {
                self.backend_list.remove(&backend);
            }
            BackendMsg::BackendStatus { backend, status } => {
                self.backend_list.get_mut(&backend).unwrap().1 = status;
            }
        }
    }

     fn make_update_packet(&self, line: usize) -> CUpdateScore {
         let mut text = TextComponent::default();
         text.extra = TextComponent::from_legacy_text(&self.current_state[line]);

         CUpdateScore {
            entity_name: self.current_state[line].clone(),
            objective_name: "status".to_string(),
            value: (self.current_state.len() - line) as i32,
            display_name: Some(text),
            number_format: Some(ObjectiveNumberFormat::Blank),
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