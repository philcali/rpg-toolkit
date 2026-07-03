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
