use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const PATH_SETTINGS: &str = "settings/";

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub game_settings: GameSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GameSettings {
    pub keep_seed: bool,
    pub seed: Option<u64>,
    pub clock_game_settings: ClockGameSettings,
    pub race_game_settings: RaceGameSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextOrigin {
    Generated,
    Text(OffsetText),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClockGameSettings {
    pub text_origin: TextOrigin,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct RaceGameSettings {
    pub text_origin: TextOrigin,
    pub file_name: PathBuf,
    pub number_words: usize,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub enum OffsetText {
    Beginning,
    #[default]
    Random,
}

impl Default for TextOrigin {
    fn default() -> Self {
        TextOrigin::Text(OffsetText::default())
    }
}
