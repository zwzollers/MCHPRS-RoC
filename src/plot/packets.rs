use super::Plot;
use crate::blocks::{Block, BlockFace, BlockPos};
use crate::items::{Item, ItemStack, UseOnBlockContext};
use crate::network::packets::clientbound::*;
use crate::network::packets::serverbound::*;
use crate::network::packets::{DecodeResult, PacketDecoder};
use crate::player::SkinParts;
use crate::server::Message;
use log::debug;
use serde_json::json;
use std::time::Instant;

impl Plot {
    pub(super) fn handle_packets_for_player(&mut self, player: usize) {
        let packets: Vec<PacketDecoder> = self.players[player].client.packets.drain(..).collect();
        for packet in packets {
            let id = packet.packet_id;
            if let Err(err) = self.handle_packet(player, packet) {
                self.players[player].kick(
                    json!({
                        "text":
                            format!("There was an error handling packet 0x{:02X}: {:?}", id, err)
                    })
                    .to_string(),
                );
            }
        }
    }

    fn handle_packet(&mut self, player: usize, packet: PacketDecoder) -> DecodeResult<()> {
        match packet.packet_id {
            0x03 => self.handle_chat_message(player, S03ChatMessage::decode(packet)?),
            0x05 => self.handle_client_settings(player, S05ClientSettings::decode(packet)?),
            0x0B => self.handle_plugin_message(player, S0BPluginMessage::decode(packet)?),
            0x0F => self.players[player].last_keep_alive_received = Instant::now(), // Keep Alive
            0x11 => self.handle_player_position(player, S11PlayerPosition::decode(packet)?),
            0x12 => self.handle_player_position_and_rotation(
                player,
                S12PlayerPositionAndRotation::decode(packet)?,
            ),
            0x13 => self.handle_player_rotation(player, S13PlayerRotation::decode(packet)?),
            0x14 => self.handle_player_movement(player, S14PlayerMovement::decode(packet)?),
            0x19 => self.handle_player_abilities(player, S19PlayerAbilities::decode(packet)?),
            0x1A => self.handle_player_digging(player, S1APlayerDigging::decode(packet)?),
            0x1B => self.handle_entity_action(player, S1BEntityAction::decode(packet)?),
            0x23 => self.handle_held_item_change(player, S23HeldItemChange::decode(packet)?),
            0x26 => self.handle_creative_inventory_action(
                player,
                S26CreativeInventoryAction::decode(packet)?,
            ),
            0x2A => self.handle_animation(player, S2AAnimation::decode(packet)?),
            0x2C => {
                self.handle_player_block_placement(player, S2CPlayerBlockPlacemnt::decode(packet)?)
            }
            id => {
                debug!("Unhandled packet: {:02X}", id);
            }
        }
        Ok(())
    }

    fn handle_creative_inventory_action(
        &mut self,
        player: usize,
        creative_inventory_action: S26CreativeInventoryAction,
    ) {
        if let Some(slot_data) = creative_inventory_action.clicked_item {
            let item = ItemStack {
                count: slot_data.item_count as u8,
                damage: 0,
                item_type: Item::from_id(slot_data.item_id as u32),
                nbt: slot_data.nbt,
            };
            self.players[player].inventory[creative_inventory_action.slot as usize] = Some(item);
        } else {
            self.players[player].inventory[creative_inventory_action.slot as usize] = None;
        }
    }

    fn handle_player_abilities(&mut self, player: usize, player_abilities: S19PlayerAbilities) {
        self.players[player].flying = player_abilities.flags.contains(S19PlayerAbilitiesFlags::IS_FLYING);
    }

    fn handle_animation(&mut self, player: usize, animation: S2AAnimation) {
        let animation_id = match animation.hand {
            0 => 0,
            1 => 3,
            _ => 0,
        };
        let entity_animation = C06EntityAnimation {
            entity_id: self.players[player].entity_id as i32,
            animation: animation_id,
        }
        .encode();
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player]
                .client
                .send_packet(&entity_animation);
        }
    }

    fn handle_player_block_placement(
        &mut self,
        player: usize,
        player_block_placement: S2CPlayerBlockPlacemnt,
    ) {
        let block_face = BlockFace::from_id(player_block_placement.face as u32);

        let selected_slot = self.players[player].selected_slot as usize;
        let item_in_hand = if player_block_placement.hand == 0 {
            self.players[player].inventory[selected_slot + 36].clone()
        } else {
            self.players[player].inventory[45].clone()
        };

        let block_pos = BlockPos::new(
            player_block_placement.x,
            player_block_placement.y as u32,
            player_block_placement.z,
        );

        if let Some(item) = item_in_hand {
            item.use_on_block(
                self,
                UseOnBlockContext {
                    block_face,
                    block_pos,
                    player_crouching: self.players[player].crouching,
                    player_direction: self.players[player].get_direction(),
                },
            );
        }
    }

    fn handle_chat_message(&mut self, player: usize, chat_message: S03ChatMessage) {
        let message = chat_message.message;
        if message.starts_with('/') {
            let mut args: Vec<&str> = message.split(' ').collect();
            let command = args.remove(0);
            self.handle_command(player, command, args);
        } else {
            let player = &self.players[player];
            let broadcast_message = Message::ChatInfo(player.username.to_owned(), message);
            self.message_sender.send(broadcast_message).unwrap();
        }
    }

    fn handle_client_settings(&mut self, player: usize, client_settings: S05ClientSettings) {
        let player = &mut self.players[player];
        player.skin_parts =
            SkinParts::from_bits_truncate(client_settings.displayed_skin_parts as u32);
        let metadata_entry = C44EntityMetadataEntry {
            index: 16,
            metadata_type: 0,
            value: vec![player.skin_parts.bits() as u8],
        };
        let entity_metadata = C44EntityMetadata {
            entity_id: player.entity_id as i32,
            metadata: vec![metadata_entry],
        }
        .encode();
        for player in &mut self.players {
            player.client.send_packet(&entity_metadata);
        }
    }

    fn handle_plugin_message(&mut self, _player: usize, plugin_message: S0BPluginMessage) {
        debug!(
            "Client initiated plugin channel: {:?}",
            plugin_message.channel
        );
    }

    fn handle_player_position(&mut self, player: usize, player_position: S11PlayerPosition) {
        let old_x = self.players[player].x;
        let old_y = self.players[player].y;
        let old_z = self.players[player].z;
        let new_x = player_position.x;
        let new_y = player_position.y;
        let new_z = player_position.z;
        self.players[player].x = player_position.x;
        self.players[player].y = player_position.y;
        self.players[player].z = player_position.z;
        self.players[player].on_ground = player_position.on_ground;
        let packet = if (new_x - old_x).abs() > 8.0
            || (new_y - old_y).abs() > 8.0
            || (new_z - old_z).abs() > 8.0
        {
            C57EntityTeleport {
                entity_id: self.players[player].entity_id as i32,
                x: new_x,
                y: new_y,
                z: new_z,
                yaw: self.players[player].yaw,
                pitch: self.players[player].pitch,
                on_ground: player_position.on_ground,
            }
            .encode()
        } else {
            let delta_x = ((player_position.x * 32.0 - old_x * 32.0) * 128.0) as i16;
            let delta_y = ((player_position.y * 32.0 - old_y * 32.0) * 128.0) as i16;
            let delta_z = ((player_position.z * 32.0 - old_z * 32.0) * 128.0) as i16;
            C29EntityPosition {
                delta_x,
                delta_y,
                delta_z,
                entity_id: self.players[player].entity_id as i32,
                on_ground: player_position.on_ground,
            }
            .encode()
        };
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player].client.send_packet(&packet);
        }
    }

    fn handle_player_position_and_rotation(
        &mut self,
        player: usize,
        player_position_and_rotation: S12PlayerPositionAndRotation,
    ) {
        let old_x = self.players[player].x;
        let old_y = self.players[player].y;
        let old_z = self.players[player].z;
        let new_x = player_position_and_rotation.x;
        let new_y = player_position_and_rotation.y;
        let new_z = player_position_and_rotation.z;
        self.players[player].x = player_position_and_rotation.x;
        self.players[player].y = player_position_and_rotation.y;
        self.players[player].z = player_position_and_rotation.z;
        self.players[player].yaw = player_position_and_rotation.yaw;
        self.players[player].pitch = player_position_and_rotation.pitch;
        self.players[player].on_ground = player_position_and_rotation.on_ground;
        let packet = if (new_x - old_x).abs() > 8.0
            || (new_y - old_y).abs() > 8.0
            || (new_z - old_z).abs() > 8.0
        {
            C57EntityTeleport {
                entity_id: self.players[player].entity_id as i32,
                x: new_x,
                y: new_y,
                z: new_z,
                yaw: self.players[player].yaw,
                pitch: self.players[player].pitch,
                on_ground: player_position_and_rotation.on_ground,
            }
            .encode()
        } else {
            let delta_x = ((player_position_and_rotation.x * 32.0 - old_x * 32.0) * 128.0) as i16;
            let delta_y = ((player_position_and_rotation.y * 32.0 - old_y * 32.0) * 128.0) as i16;
            let delta_z = ((player_position_and_rotation.z * 32.0 - old_z * 32.0) * 128.0) as i16;
            C2AEntityPositionAndRotation {
                delta_x,
                delta_y,
                delta_z,
                pitch: player_position_and_rotation.pitch,
                yaw: player_position_and_rotation.yaw,
                entity_id: self.players[player].entity_id as i32,
                on_ground: player_position_and_rotation.on_ground,
            }
            .encode()
        };
        let entity_head_look = C3CEntityHeadLook {
            entity_id: self.players[player].entity_id as i32,
            yaw: player_position_and_rotation.yaw,
        }
        .encode();
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player].client.send_packet(&packet);
            self.players[other_player]
                .client
                .send_packet(&entity_head_look);
        }
    }

    fn handle_player_rotation(&mut self, player: usize, player_rotation: S13PlayerRotation) {
        self.players[player].yaw = player_rotation.yaw;
        self.players[player].pitch = player_rotation.pitch;
        self.players[player].on_ground = player_rotation.on_ground;
        let rotation_packet = C2BEntityRotation {
            entity_id: self.players[player].entity_id as i32,
            yaw: player_rotation.yaw,
            pitch: player_rotation.pitch,
            on_ground: player_rotation.on_ground,
        }
        .encode();
        let entity_head_look = C3CEntityHeadLook {
            entity_id: self.players[player].entity_id as i32,
            yaw: player_rotation.yaw,
        }
        .encode();
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player]
                .client
                .send_packet(&rotation_packet);
            self.players[other_player]
                .client
                .send_packet(&entity_head_look);
        }
    }

    fn handle_player_movement(&mut self, player: usize, player_movement: S14PlayerMovement) {
        self.players[player].on_ground = player_movement.on_ground;
        let packet = C2CEntityMovement {
            entity_id: self.players[player].entity_id as i32,
        }
        .encode();
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player].client.send_packet(&packet);
        }
    }

    fn handle_player_digging(&mut self, player: usize, player_digging: S1APlayerDigging) {
        if player_digging.status == 0 {
            let block_pos =
                BlockPos::new(player_digging.x, player_digging.y as u32, player_digging.z);
            let other_block = self.get_block(&block_pos);
            self.set_block(&block_pos, Block::Air);

            let effect = C23Effect {
                effect_id: 2001,
                x: player_digging.x,
                y: player_digging.y,
                z: player_digging.z,
                data: other_block.get_id() as i32,
                disable_relative_volume: false,
            }
            .encode();
            for other_player in 0..self.players.len() {
                if player == other_player {
                    continue;
                };
                self.players[other_player].client.send_packet(&effect);
            }
        }
    }

    fn handle_entity_action(&mut self, player: usize, entity_action: S1BEntityAction) {
        match entity_action.action_id {
            0 => self.players[player].crouching = true,
            1 => self.players[player].crouching = false,
            3 => self.players[player].sprinting = true,
            4 => self.players[player].sprinting = false,
            _ => {}
        }
        let mut bitfield = 0;
        if self.players[player].crouching {
            bitfield |= 0x02
        };
        if self.players[player].sprinting {
            bitfield |= 0x08
        };
        let mut metadata_entries = Vec::new();
        metadata_entries.push(C44EntityMetadataEntry {
            index: 0,
            metadata_type: 0,
            value: vec![bitfield],
        });
        metadata_entries.push(C44EntityMetadataEntry {
            index: 6,
            metadata_type: 18,
            value: vec![if self.players[player].crouching { 5 } else { 0 }],
        });
        let entity_metadata = C44EntityMetadata {
            entity_id: self.players[player].entity_id as i32,
            metadata: metadata_entries,
        }
        .encode();
        for other_player in 0..self.players.len() {
            if player == other_player {
                continue;
            };
            self.players[other_player]
                .client
                .send_packet(&entity_metadata);
        }
    }

    fn handle_held_item_change(&mut self, player: usize, held_item_change: S23HeldItemChange) {
        self.players[player].selected_slot = held_item_change.slot as u32;
    }
}