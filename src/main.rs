
use std::{cell::{RefCell, RefMut}, rc::Rc};

use action::Action;
use dispatcher::Dispatcher;
mod app;
use app::App;
mod ui;
mod action;
mod stores;
mod flux;
mod dispatcher;
mod user_input;
mod text_generator;
mod settings;
mod async_utils;

use ratatui::symbols::bar::Set;
use settings::Settings;
use tokio::sync::mpsc::UnboundedSender;
use user_input::UserInput;

#[tokio::main]
async fn main() {
    let settings: Rc<RefCell<Settings>> =  Rc::new(RefCell::new(Settings::default()));
    let (mut dispatcher, dispatcher_tx) = Dispatcher::new();
    let (mut app, app_tx) = App::new(dispatcher_tx.clone(), settings);

    
    dispatcher.add_store(app_tx);
    let user_input = UserInput::new(dispatcher_tx.clone());
    tokio::select! {
        //_ = stress_test(dispatcher_tx.clone()) => {},
        _ = dispatcher.dispatch() => {}
        _ = app.run() => {}
        _ = user_input.main_loop() => {}
    }
    ratatui::restore();
}

async fn stress_test(dispatcher_tx: UnboundedSender<Action>){
    loop {
        dispatcher_tx.send(Action::KeyPressed(' ')).unwrap();
        tokio::task::yield_now().await;
    }
}