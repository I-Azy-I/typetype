use std::path::PathBuf;

use ratatui::text;
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextOrigin {
    Generated,
    Text(OffsetText),
}
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct InfiniteGameSettings {
    pub text_origin: TextOrigin,
    pub filename: String,
    pub start_end_sentence: StartEndSentence,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClockGameSettings {
    pub text_origin: TextOrigin,
    pub filename: String,
    pub time: usize,
    pub start_end_sentence: StartEndSentence,
}
impl Default for ClockGameSettings {
    fn default() -> Self {
        let text_origin = TextOrigin::default();
        let filename = match text_origin {
            TextOrigin::Generated => DEFAULT_LANGUAGE,
            TextOrigin::Text(_) => DEFAULT_TEXT,
        };

        Self {
            text_origin: Default::default(),
            filename: String::from(filename),
            time: 30,
            start_end_sentence: StartEndSentence::default(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaceGameSettings {
    pub text_origin: TextOrigin,
    pub filename: String,
    pub number_words: usize,
    pub start_end_sentence: StartEndSentence,
}
impl Default for RaceGameSettings {
    fn default() -> Self {
        let text_origin = TextOrigin::default();
        let filename = match text_origin {
            TextOrigin::Generated => DEFAULT_LANGUAGE,
            TextOrigin::Text(_) => DEFAULT_TEXT,
        };

        Self {
            text_origin: Default::default(),
            filename: String::from(filename),
            number_words: 50,
            start_end_sentence: StartEndSentence::default(),
        }
    }
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, Copy)]
pub enum StartEndSentence {
    #[default]
    Any,
    Start,
    StartEnd,
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
