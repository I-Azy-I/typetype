use std::{
    fs,
    path::{Path, PathBuf},
};

use ratatui::text;
use serde::{Deserialize, Serialize};
use tokio;

use crate::config::{default_language, default_text, path_settings, path_languages, path_texts};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextSettings {
    pub text_origin: TextOrigin,
    pub filename: Option<String>,
}
impl TextSettings {
    fn try_get_filename(path_files: PathBuf, default_filename: String) -> Option<String> {
        if path_files.join(&default_filename).exists() {
            Some(default_filename.to_string())
        } else {
            if let Ok(mut entries) = fs::read_dir(&default_filename) {
                if let Some(entry_result) = entries.next() {
                    let entry = entry_result.unwrap();
                    Some(entry.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
impl Default for TextSettings {
    fn default() -> Self {
        let text_origin = TextOrigin::default();
        let filename = match text_origin {
            TextOrigin::Language => Self::try_get_filename(path_languages(), default_language()),
            TextOrigin::Text { .. } => Self::try_get_filename(path_texts(), default_text()),
        };
        Self {
            text_origin: TextOrigin::default(),
            filename,
        }
    }
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

pub fn save_settings(settings: Settings) {
    tokio::spawn(async move {
        let settings_toml_string =
            toml::to_string(&settings).expect("To serialize settings to TOML");
        tokio::fs::write( path_settings(), settings_toml_string).await.expect("To write settings to file");
    });
}

pub fn load_settings() -> Settings {
    if path_settings().exists() {
        let settings_toml_string =
            fs::read_to_string(path_settings()).expect("To read settings from file");
        toml::from_str(&settings_toml_string).expect("To deserialize settings from TOML")
    } else {
        Settings::default()
    }
}
