use std::{cell::OnceCell, fs, path::{Path, PathBuf}};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[cfg(not(windows))] const PATH_GENERAL_CONFIG: &str = "config/general_settings.toml";
#[cfg(    windows) ] const PATH_GENERAL_CONFIG: &str = "config\\general_settings.toml";

const _PATH_LANGUAGES: &str = "languages";
const _PATH_TEXTS: &str = "texts";
const _DEFAULT_LANGUAGE: &str = "english_1k.json";
const _DEFAULT_TEXT: &str = "lotr.txt";

const _PATH_DIR_SETTING: &str = "config";
const _SETTING_SOLO_GAMES_FILENAME: &str = "solo_games_settings.toml";

const _MILISECONDS_PER_FRAME: u64 = 20;

static GENERAL_CONFIG: Lazy<GeneralConfig> = Lazy::new(|| {
    // Example: read config file
    let path = Path::new(PATH_GENERAL_CONFIG);
    if path.exists() {
        let general_config =
            fs::read_to_string(path).expect("To read general config from file");
        toml::from_str(&general_config).expect("To deserialize config from TOML")
    } else {
        GeneralConfig::default()
    }
    
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    path_languages: String,
    path_texts: String,
    default_language: String,
    default_text: String,
    path_dir_setting: String,
    setting_solo_games_filename: String,
    miliseconds_per_frame: u64,
}
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            path_languages: _PATH_LANGUAGES.to_string(),
            path_texts: _PATH_TEXTS.to_string(),
            default_language: _DEFAULT_LANGUAGE.to_string(),
            default_text: _DEFAULT_TEXT.to_string(),
            path_dir_setting: _PATH_DIR_SETTING.to_string(),
            setting_solo_games_filename: _SETTING_SOLO_GAMES_FILENAME.to_string(),
            miliseconds_per_frame: _MILISECONDS_PER_FRAME,
        }
    }
}




pub fn path_settings() -> PathBuf {
    Path::new(&GENERAL_CONFIG.path_dir_setting)
        .join(&GENERAL_CONFIG.setting_solo_games_filename)
}


pub fn default_language() -> String {
    GENERAL_CONFIG.default_language.clone()
}

pub fn default_text() -> String {
    GENERAL_CONFIG.default_text.clone()
}

pub fn path_languages() -> PathBuf {
    Path::new(&GENERAL_CONFIG.path_languages).into()
}

pub fn path_texts() -> PathBuf {
    Path::new(&GENERAL_CONFIG.path_texts).into()
}

pub fn miliseconds_per_frame() -> u64 {
    GENERAL_CONFIG.miliseconds_per_frame
}

pub fn path_general_config() -> &'static str {
    PATH_GENERAL_CONFIG
}

