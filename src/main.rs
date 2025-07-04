use std::{cell::RefCell, rc::Rc};

use action::Action;
use dispatcher::Dispatcher;
mod app;
use app::App;
mod action;
mod dispatcher;
mod flux;
mod settings;
mod stores;
mod text_generator;
mod ui;
mod user_input;

use settings::Settings;
use tokio::sync::mpsc::UnboundedSender;
use user_input::UserInput;

#[tokio::main]
async fn main() {
    [cfg!(debug_assertions)];
    {
        let _ = simple_logging::log_to_file("test.log", log::LevelFilter::Debug);
    }

    let settings: Rc<RefCell<Settings>> = Rc::new(RefCell::new(Settings::default()));
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

async fn stress_test(dispatcher_tx: UnboundedSender<Action>) {
    loop {
        dispatcher_tx.send(Action::KeyPressed(' ')).unwrap();
        tokio::task::yield_now().await;
    }
}
