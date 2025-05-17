use crate::ui::{gauge, screens::Screen};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    StartingApp,

    AskChangeToScreen(Screen),
    OpeningScreen(Screen),
    ClosingScreen(Screen),  

    
    KeyPressed(char),
    BackspacePressed,
    EscPressed,
    EnterPressed,
    UpPressed,
    DownPressed,
    Exit,
    
    UpdateLineGauge(gauge::GaugeId, f32),
    InitializeSoloSpeedGame,

    None
}