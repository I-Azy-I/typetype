use std::{io, time::Duration};

use ratatui::crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc::{UnboundedSender};

use crate::action::Action;
pub struct UserInput {
    dispatcher_tx: UnboundedSender<Action>,
}
impl UserInput {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        UserInput {dispatcher_tx}
    }
    pub async fn main_loop(self){
        loop {
            tokio::task::yield_now().await;
            self.handle_events().unwrap();
        }
    }
    fn is_event_available() -> io::Result<bool> {
        // Zero duration says that the `poll` function must return immediately
        // with an `Event` availability information
        poll(Duration::from_secs(0))
    }
    fn handle_events(&self) -> io::Result<()> {
        if !Self::is_event_available()? {
            return Ok(());
        }
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    
    fn handle_key_event(&self, key_event: KeyEvent){
        
        let opt_action = match key_event.code {
            KeyCode::Char(key) => Some(Action::KeyPressed(key)),
            KeyCode::Backspace =>  Some(Action::BackspacePressed),
            KeyCode::Esc => Some(Action::EscPressed),
            KeyCode::Up => Some(Action::UpPressed),
            KeyCode::Down => Some(Action::DownPressed),
            KeyCode::Enter => Some(Action::EnterPressed),
            _ => {None}
        };
        if let Some(action) = opt_action { self.dispatcher_tx.send(action).unwrap();}
    }
}


