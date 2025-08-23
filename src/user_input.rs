use std::io;

use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use log::debug;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
pub struct UserInput {
    dispatcher_tx: UnboundedSender<Action>,
}
impl UserInput {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        UserInput { dispatcher_tx }
    }
    pub async fn main_loop(self) -> io::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        use futures::pin_mut;
        loop {
            debug!("intput revieved");
            tokio::task::yield_now().await;
            let crossterm_event = reader.next().fuse();
            pin_mut!(crossterm_event);
            if let Some(Ok(CrosstermEvent::Key(key_event))) = crossterm_event.await {
                
                    self.handle_key_event(key_event)
                
            }
        }
    }

    fn handle_key_event(&self, key_event: KeyEvent) {
        let opt_action = match key_event.code {
            crossterm::event::KeyCode::Char(key) => Some(Action::KeyPressed(key)),
            crossterm::event::KeyCode::Backspace => Some(Action::BackspacePressed),
            crossterm::event::KeyCode::Esc => Some(Action::EscPressed),
            crossterm::event::KeyCode::Up => Some(Action::UpPressed),
            crossterm::event::KeyCode::Down => Some(Action::DownPressed),
            crossterm::event::KeyCode::Right => Some(Action::RightPressed),
            crossterm::event::KeyCode::Left => Some(Action::LeftPressed),
            crossterm::event::KeyCode::Enter => Some(Action::EnterPressed),
            _ => None,
        };
        if let Some(action) = opt_action {
            self.dispatcher_tx.send(action).unwrap();
        }
    }
}
