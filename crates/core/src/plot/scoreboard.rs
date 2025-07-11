use crate::{player::{PacketSender, Player}, plot::Plot};
use fpga::scheduler::FPGAScheduler;
use mchprs_network::packets::clientbound::{
    CDisplayObjective, CResetScore, CUpdateObjectives, CUpdateScore, ClientBoundPacket,
    ObjectiveNumberFormat,
};
use mchprs_backend::{BackendStatus, BackendMsg};
use mchprs_redpiler::CompilerOptions;
use mchprs_text::{ColorCode, TextComponent, TextComponentBuilder};
use std::{collections::HashMap, sync::{Arc, Mutex}};

#[derive(Default)]
pub struct Scoreboard {
    backend_list: HashMap<String, (CompilerOptions, BackendStatus)>,
    lines: Vec<ScoreboardLine>,
}

impl Scoreboard {
    pub fn new() -> Scoreboard{
        let mut sb: Scoreboard = Default::default();
        sb
    }

    fn to_str_vec(&self, scheduler: &Arc<Mutex<FPGAScheduler>>) -> Vec<ScoreboardLine> {
        let mut sb: Vec<ScoreboardLine> = Vec::new();

        sb.push(ScoreboardLine::from_str("&fRTPS", Some("10")));
        sb.push(ScoreboardLine::from_str("&fWSR", Some("10")));
        sb.push(ScoreboardLine::from_str("", None));

        sb.push(ScoreboardLine::from_str(&format!("&f{:^23}", "Backends"), None));

        for (name, (options, status)) in &self.backend_list {
            sb.push(ScoreboardLine::from_str(&format!("&f{}", name), Some(&status.to_str())));
            for option in options.to_str_vec() {
                sb.push(ScoreboardLine::from_str(&option, None));
            }
        }
        sb.push(ScoreboardLine::from_str("", None));

        sb.push(ScoreboardLine::from_str(&format!("&f{:^23}", "FPGA"), None));

        let fpgas = scheduler.lock().unwrap();

        for fpga in &fpgas.fpgas {
            if let Some((x,z)) = fpga.get_owner() {
                sb.push(ScoreboardLine::from_str(&format!("&f{}", fpga.config.name), Some("&cLocked")));
                sb.push(ScoreboardLine::from_str("  &fOwner", Some(&format!("&f{x},{z}"))));
            }
            else {
                sb.push(ScoreboardLine::from_str(&format!("&f{}", fpga.config.name), Some("&aOpen")));
            }
            sb.push(ScoreboardLine::from_str("", None));
        }
        
        sb
    }

    pub fn add_backend(&mut self, name: String, options: CompilerOptions) {
        self.backend_list.insert(name, (options, BackendStatus::Redpiling));
    }

    pub fn update (&mut self, players: &[Player], scheduler: &Arc<Mutex<FPGAScheduler>>) {
        self.set_lines(players, self.to_str_vec(scheduler));
        
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
        let right_text = if let Some(right) = &self.lines[line].right_text {
            Some(ObjectiveNumberFormat::Fixed { content: right.clone() })
        }
        else {
            Some(ObjectiveNumberFormat::Blank)
        };

        CUpdateScore {
            entity_name: line.to_string(),
            objective_name: "MCHPRS".to_string(),
            value: (self.lines.len() - line) as i32,
            display_name: Some(self.lines[line].left_text.clone()),
            number_format: right_text,
        }
    }

    fn make_removal_packet(&self, line: usize) -> CResetScore {
        CResetScore {
            entity_name: line.to_string(),
            objective_name: Some("MCHPRS".to_string()),
        }
    }

    fn set_lines(&mut self, players: &[Player], lines: Vec<ScoreboardLine>) {
        for line in 0..self.lines.len() {
            let removal_packet = self.make_removal_packet(line).encode();
            players.iter().for_each(|p| p.send_packet(&removal_packet));
        }

        self.lines = lines;

        for line in 0..self.lines.len() {
            let update_packet = self.make_update_packet(line).encode();
            players.iter().for_each(|p| p.send_packet(&update_packet));
        }
    }

    pub fn add_player(&self, player: &Player) {
        player.send_packet(
            &CUpdateObjectives {
                objective_name: "MCHPRS".into(),
                mode: 0,
                objective_value: TextComponentBuilder::new("MCHPRS".into())
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
                score_name: "MCHPRS".into(),
            }
            .encode(),
        );
        for i in 0..self.lines.len() {
            player.send_packet(&self.make_update_packet(i).encode());
        }
    }

    pub fn remove_player(&mut self, player: &Player) {
        for i in 0..self.lines.len() {
            player.send_packet(&self.make_removal_packet(i).encode());
        }
    }
}

struct ScoreboardLine {
    left_text: TextComponent,
    right_text: Option<TextComponent>
}

impl ScoreboardLine {
    fn from_str(left: &str, right: Option<&str>) -> ScoreboardLine {
        let mut sb_line = ScoreboardLine{
            left_text: TextComponent::from_legacy_text(left),
            right_text: None 
        };
        if let Some(right_str) = right {
            sb_line.right_text = Some(TextComponent::from_legacy_text(right_str));
        }

        sb_line
    }
}