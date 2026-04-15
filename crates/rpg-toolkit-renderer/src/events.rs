use bevy::prelude::*;
use rpg_toolkit_common::MapId;

/// Fired when the active map changes (via JumpTo or initial load).
#[derive(Message)]
pub struct MapChanged {
    pub previous_map_id: Option<MapId>,
    pub new_map_id: MapId,
}

/// Fired when the player completes a move to a new tile.
#[derive(Message)]
pub struct PlayerMoved {
    pub from: (u32, u32),
    pub to: (u32, u32),
}
