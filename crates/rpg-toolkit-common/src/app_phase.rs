use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppPhase {
    #[default]
    TitleScreen,
    InGame,
    Battle,
    Shop,
    Status,
}

/// Marker resource inserted by the title screen to signal a fresh new game
/// (as opposed to loading a save). Systems that should only run on new game
/// start (e.g., intro narration) check for the presence of this resource.
#[derive(Resource)]
pub struct NewGameFlag;
