use crate::ui::screens::Screen;

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

    InitializeSoloSpeedGame,

    None
}