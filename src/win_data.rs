use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct WinData {
    pub game: GameModEndResult,
}

#[derive(Default, Debug, Clone)]
pub enum GameModEndResult {
    #[default]
    None,
    Infinite(InfiniteData),
    Race(RaceData),
    Clock(ClockData),
}
#[derive(Default, Debug, Clone)]
pub struct InfiniteData {
    pub time_spend: Duration,
}
#[derive(Default, Debug, Clone)]
pub struct RaceData {
    pub skip: bool,
    pub time: Duration,
    pub n_words: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ClockData {
    pub skip: bool,
    pub time: usize,
    pub n_words: usize,
}
