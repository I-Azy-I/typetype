use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use dispatcher::Dispatcher;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
mod app;
use app::App;
mod ui;
mod action;
mod stores;
mod flux;
mod dispatcher;
mod user_input;

use user_input::UserInput;

#[tokio::main]
async fn main() {
    
    let (mut dispatcher, dispatcher_tx) = Dispatcher::new();
    let (mut app, app_tx) = App::new(dispatcher_tx.clone());
    dispatcher.add_store(app_tx);
    let user_input = UserInput::new(dispatcher_tx);
    tokio::select! {
        _ = dispatcher.dispatch() => {}
        _ = app.run() => {}
        _ = user_input.main_loop() => {}
    }
    ratatui::restore();
}