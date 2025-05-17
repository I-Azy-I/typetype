use std::{default, io, iter, time::Duration, u16::MIN};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use tokio::{sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender}, time::sleep};

use crate::{action::Action, ui::{screens::{Screen, ScreenRouterComponent}, text::TextWidgetComponent}};
use crate::stores::Store;





enum CurrentScreen {
    Menu,
    BasicType,
    Settings,
}

#[derive(Debug)]
pub struct AppStore{
    action_rx: UnboundedReceiver<Action>,
    dispatcher_tx: UnboundedSender<Action>,
    exit: bool,
    screen_router: ScreenRouterComponent,
}
impl  AppStore {
    fn new(dispatcher_tx: UnboundedSender<Action>) -> (Self, UnboundedSender<Action>) {
        let (action_tx, action_rx) = unbounded_channel::<Action>();
        let screen_router = ScreenRouterComponent::new(dispatcher_tx.clone());
        (AppStore {action_rx,dispatcher_tx, screen_router, exit: false}, action_tx)
    }
    async fn update(&mut self) {
        // Set up a timeout of 10ms
        let timeout = tokio::time::sleep(Duration::from_millis(10));
        tokio::pin!(timeout);
        
        // Keep processing actions until timeout
        loop {
            // println!("waiting for acrtion");
            // let action = self.action_rx.recv().await.unwrap();
            // println!("action recieved");
            // self.text_store.update(action);
            
            tokio::select! {
                // Try to receive more actions (will not block if channel is empty)
                biased;
                
                maybe_action = self.action_rx.recv() => {
                    match maybe_action {
                        Some(action) => {
                            if matches!(action, Action::Exit) {self.exit = true}
                            self.screen_router.update(action);
                        },
                        None => {
                            // Channel is closed, exit loop
                            break;
                        }
                    }
                }
                // If timeout completes, exit the loop
                _ = &mut timeout => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct App {
    app_store: AppStore,
    
}

impl App {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) ->  (Self, UnboundedSender<Action>) {
        let (app_store, action_tx) = AppStore::new(dispatcher_tx);
        (App {app_store}, action_tx)
    }
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();
        self.app_store.dispatcher_tx.send(Action::StartingApp).unwrap();
        while !self.app_store.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.app_store.update().await;
            
        }
        Ok(())
    }
  
    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.app_store.screen_router.render(area, buf);
        
    }
}




