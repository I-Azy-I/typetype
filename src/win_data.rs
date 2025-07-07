use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct WinData {
    pub game: GameMod,
}

#[derive(Default, Debug, Clone)]
pub enum GameMod {
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
    pub time: Duration,
    pub n_words: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ClockData {
    pub time: Duration,
    pub n_words: usize,
}
