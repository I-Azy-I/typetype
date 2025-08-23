use std::{cell::RefCell, fs, path::Path, rc::Rc};

use action::Action;
use dispatcher::Dispatcher;
mod app;
use app::App;
mod action;
mod config;
mod dispatcher;
mod flux;
mod settings;
mod stores;
mod text_generator;
mod ui;
mod user_input;
mod win_data;

use log::debug;
use settings::Settings;
use tokio::sync::mpsc::UnboundedSender;
use user_input::UserInput;

use crate::settings::load_settings;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        let _ = simple_logging::log_to_file("test.log", log::LevelFilter::Debug);
    }
    #[cfg(debug_assertions)]
    {
        use crate::config::{path_general_config, path_settings};

        if !path_settings().exists() {
            let settings_toml_string =
                toml::to_string(&Settings::default()).expect("To serialize settings to TOML");
            fs::write(path_settings(), settings_toml_string)
                .expect("To write default settings to file");
        }

        if !Path::new(path_general_config()).exists() {
            use crate::config::GeneralConfig;

            let config_toml_string =
                toml::to_string(&GeneralConfig::default()).expect("To serialize config to TOML");
            fs::write(path_general_config(), config_toml_string)
                .expect("To write default settings to file");
        }
    }

    let settings: Rc<RefCell<Settings>> = Rc::new(RefCell::new(load_settings()));
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
