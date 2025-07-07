use crate::ui::{gauge, screens::Screen};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    StartingApp,

    AskChangeToScreen(Screen),
    AskOpenScreen,
    AskCloseScreen(Screen),
    ScreenClosed(Screen),

    KeyPressed(char),
    BackspacePressed,
    EscPressed,
    EnterPressed,
    UpPressed,
    DownPressed,
    RightPressed,
    LeftPressed,
    Exit,

    AsyncCachedRecievedData(Option<u32>),
    None,
}
