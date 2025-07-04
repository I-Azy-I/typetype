pub mod solo_race_game;
pub mod solo_infinite_game;
#[derive(Debug)]
pub enum GameMod {
    Race,
    Clock,
    Infinite,
}
