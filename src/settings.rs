use serde::{Deserialize, Serialize};

use crate::config::{DEFAULT_LANGUAGE, DEFAULT_TEXT};

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
    pub infinite_game_settings: InfiniteGameSettings,
    pub text_settings: TextSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct TextSettings {
    pub text_origin: TextOrigin,
    pub filename: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum TextOrigin {
    #[default]
    Language,
    Text {
        start_end_sentence: StartEndSentence,
        starting_point: StartingPointSentence,
    }, // should add StartEnd and StartingPoint in it
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct InfiniteGameSettings {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClockGameSettings {
    pub time: usize,
}
impl Default for ClockGameSettings {
    fn default() -> Self {
        Self { time: 30 }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaceGameSettings {
    pub number_words: usize,
}
impl Default for RaceGameSettings {
    fn default() -> Self {
        Self { number_words: 50 }
    }
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, Copy)]
pub enum StartEndSentence {
    #[default]
    Any,
    Start,
    StartEnd,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, Copy)]
pub enum StartingPointSentence {
    #[default]
    Beginning,
    Random,
}
#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub enum OffsetText {
    Beginning,
    #[default]
    Random,
}
