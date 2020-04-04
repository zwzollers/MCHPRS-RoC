use crate::network::packets::clientbound::{
    C00DisconnectLogin, C02LoginSuccess, C03SetCompression, C19PluginMessageBrand, C26JoinGame,
    C36PlayerPositionAndLook, ClientBoundPacket,
};
use crate::network::packets::serverbound::{S00Handshake, S00LoginStart, ServerBoundPacket};
use crate::network::packets::PacketDecoder;
use crate::network::{NetworkServer, NetworkState};
use crate::permissions::Permissions;
use crate::player::Player;
use crate::plot::Plot;
use bus::{Bus, BusReader};
use serde_json::json;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Messages get passed between plot threads, the server thread, and the networking thread.
/// These messages are used to communicated when a player joins, leaves, or moves into another plot,
/// as well as to communicate chat messages.
#[derive(Debug, Clone)]
pub enum Message {
    Chat(String),
    PlayerJoinedInfo(PlayerJoinInfo),
    PlayerJoined(Arc<Player>),
    PlayerLeft(u128),
    PlayerEnterPlot(Arc<Player>, i32, i32),
    PlayerTeleportOther(Arc<Player>, String),
    PlotUnload(i32, i32),
}

#[derive(Debug, Clone)]
struct PlayerJoinInfo {
    username: String,
    uuid: u128,
    skin: Option<String>,
}

#[derive(Debug, Clone)]
struct PlayerListEntry {
    plot_x: i32,
    plot_z: i32,
    username: String,
    uuid: u128,
    skin: Option<String>,
}

struct PlotListEntry {
    plot_x: i32,
    plot_z: i32,
}

/// This represents a minecraft server
pub struct MinecraftServer {
    network: NetworkServer,
    config: config::Config,
    broadcaster: Bus<Message>,
    debug_plot_receiver: BusReader<Message>,
    receiver: Receiver<Message>,
    plot_sender: Sender<Message>,
    permissions: Arc<Mutex<Permissions>>,
    online_players: Vec<PlayerListEntry>,
    running_plots: Vec<PlotListEntry>,
}

impl MinecraftServer {
    pub fn run() {
        println!("Starting server...");
        let mut config = config::Config::default();
        config
            .merge(config::File::with_name("Config"))
            .expect("Error reading config file!");
        let bind_addr = config
            .get_str("bind_address")
            .expect("Bind address not found in config file!");
        let permissions = Arc::new(Mutex::new(Permissions::new(&config)));
        let (plot_tx, server_rx) = mpsc::channel();
        let mut bus = Bus::new(100);
        let debug_plot_receiver = bus.add_rx();
        let mut server = MinecraftServer {
            network: NetworkServer::new(bind_addr),
            config,
            broadcaster: bus,
            receiver: server_rx,
            plot_sender: plot_tx,
            debug_plot_receiver,
            permissions,
            online_players: Vec::new(),
            running_plots: Vec::new(),
        };
        // Load the spawn area plot on server start
        // This plot should be always active
        Plot::load_and_run(
            0,
            0,
            server.broadcaster.add_rx(),
            server.plot_sender.clone(),
            true,
        );
        server.running_plots.push(PlotListEntry {
            plot_x: 0,
            plot_z: 0,
        });
        loop {
            server.update();
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn update(&mut self) {
        while let Ok(message) = self.debug_plot_receiver.try_recv() {
            println!("Main thread broadcasted message: {:#?}", message);
        }
        while let Ok(message) = self.receiver.try_recv() {
            println!("Main thread received message: {:#?}", message);
            match message {
                Message::PlayerJoined(player) => {
                    let plot_x = (player.x / 128f64).floor() as i32;
                    let plot_z = (player.y / 128f64).floor() as i32;
                    let plot_loaded = self
                        .running_plots
                        .iter()
                        .any(|p| p.plot_x == plot_x && p.plot_z == plot_z);
                    if !plot_loaded {
                        Plot::load_and_run(
                            plot_x,
                            plot_z,
                            self.broadcaster.add_rx(),
                            self.plot_sender.clone(),
                            false,
                        );
                        self.running_plots.push(PlotListEntry { plot_x, plot_z });
                    }
                    let player_list_entry = PlayerListEntry {
                        plot_x,
                        plot_z,
                        username: player.username.clone(),
                        uuid: player.uuid,
                        skin: None,
                    };
                    let player_join_info = PlayerJoinInfo {
                        username: player.username.clone(),
                        uuid: player.uuid,
                        skin: None,
                    };
                    self.broadcaster
                        .broadcast(Message::PlayerJoinedInfo(player_join_info));
                    println!(
                        "Arc count in main thread: {:#?}",
                        Arc::strong_count(&player)
                    );
                    self.broadcaster
                        .broadcast(Message::PlayerEnterPlot(player, plot_x, plot_z));
                    self.online_players.push(player_list_entry);
                }
                Message::PlayerLeft(uuid) => {
                    let index = self.online_players.iter().position(|p| p.uuid == uuid);
                    if let Some(index) = index {
                        self.online_players.remove(index);
                    }
                }
                Message::PlotUnload(plot_x, plot_z) => {
                    let index = self
                        .running_plots
                        .iter()
                        .position(|p| p.plot_x == plot_x && p.plot_z == plot_z);
                    if let Some(index) = index {
                        self.running_plots.remove(index);
                    }
                }
                Message::Chat(chat) => {
                    self.broadcaster.broadcast(Message::Chat(chat));
                }
                _ => {}
            }
        }
        self.network.update();
        let clients = &mut self.network.handshaking_clients;
        let mut client = 0;
        while client < clients.len() {
            let packets: Vec<PacketDecoder> = clients[client].packets.drain(..).collect();
            for packet in packets {
                match clients[client].state {
                    NetworkState::Handshake => {
                        if packet.packet_id == 0x00 {
                            let handshake = S00Handshake::decode(packet);
                            let client = &mut clients[client];
                            match handshake.next_state {
                                1 => client.state = NetworkState::Status,
                                2 => client.state = NetworkState::Login,
                                _ => {}
                            }
                            if client.state == NetworkState::Login
                                && handshake.protocol_version != 578
                            {
                                let disconnect = C00DisconnectLogin {
                                    reason: json!({
                                        "text": "Version mismatch, pleast use version 1.15.2"
                                    })
                                    .to_string(),
                                }
                                .encode();
                                client.send_packet(disconnect);
                                client.close_connection();
                            }
                        }
                    }
                    NetworkState::Status => {}
                    NetworkState::Login => {
                        if packet.packet_id == 0x00 {
                            let login_start = S00LoginStart::decode(packet);
                            clients[client].username = Some(login_start.name);
                            let set_compression = C03SetCompression { threshold: 500 }.encode();
                            clients[client].send_packet(set_compression);
                            clients[client].compressed = true;
                            let username = if let Some(name) = &clients[client].username {
                                name.clone()
                            } else {
                                Default::default()
                            };
                            let uuid = Player::generate_offline_uuid(&username);

                            let login_success = C02LoginSuccess {
                                uuid,
                                username: username.clone(),
                            }
                            .encode();
                            clients[client].send_packet(login_success);

                            clients[client].state = NetworkState::Play;
                            client -= 0;
                            let mut client = clients.remove(client + 1);

                            let join_game = C26JoinGame {
                                entity_id: client.id as i32,
                                gamemode: 1,
                                dimention: 0,
                                hash_seed: 0,
                                max_players: u8::MAX,
                                level_type: "default".to_string(),
                                view_distance: 8,
                                reduced_debug_info: false,
                                enable_respawn_screen: false,
                            }
                            .encode();
                            client.send_packet(join_game);

                            let brand = C19PluginMessageBrand {
                                brand: "Minecraft High Performace Redstone Server".to_string(),
                            }
                            .encode();
                            client.send_packet(brand);

                            let mut player = Player::load_player(uuid, username.clone(), client);

                            let player_pos_and_look = C36PlayerPositionAndLook {
                                x: player.x,
                                y: player.y,
                                z: player.z,
                                yaw: player.yaw,
                                pitch: player.pitch,
                                flags: 0,
                                teleport_id: 0,
                            }
                            .encode();
                            player.client.send_packet(player_pos_and_look);

                            self.plot_sender
                                .send(Message::PlayerJoined(Arc::new(player)))
                                .unwrap();
                        }
                    }
                    NetworkState::Play => {}
                }
                client += 1;
                println!("{} {}", client, clients.len());
            }
        }
    }
}