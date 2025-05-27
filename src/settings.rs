use std::default;

use serde::{Deserialize, Serialize};

pub const PATH_SETTINGS: &str = "settings/";



#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub game_settings: GameSettings
}


#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GameSettings {
    pub clock_game_settings: ClockGameSettings,
    pub race_game_settings: RaceGameSettings,
}
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClockGameSettings {
    pub text_origin: TextOrigin
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct RaceGameSettings {
    pub text_origin: TextOrigin,
    pub number_words: u32,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextOrigin {
    Generated(String),
    Text(String)
}
impl Default for TextOrigin {
    fn default() -> Self {
        TextOrigin::Text("lotr.txt".to_string())
    }
}


