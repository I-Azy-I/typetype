use std::{clone, fmt::Debug};

use menus::first_menu::FirstMenuComponent;
use ratatui::widgets::Widget;
use settings::setting_screen::SettingsScreenComponent;
use menus::solo_games_menu::SoloGamesMenuComponent;
use games::solo_speed_game::ScreenSoloSpeedGameComponent;
use tokio::sync::mpsc::UnboundedSender;


mod games;
mod menus;
mod settings;

use crate::{action::Action, flux::SendAction, stores::Store};

const STARTING_SCREEN: Screen = Screen::FirstMenu;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    #[default]
    FirstMenu,
    SoloGamesMenu,
    SoloSpeedGame,
    Settings
}
impl Screen {
    fn previous(self) -> Option<Screen>  {
        match self {
            Screen::FirstMenu => None,
            Screen::SoloGamesMenu => Some(Screen::FirstMenu),
            Screen::SoloSpeedGame => Some(Screen::FirstMenu),
            Screen::Settings => Some(Screen::FirstMenu),
                    }
    }
}


#[derive(Debug)]
pub struct ScreenRouterComponent {
    dispatcher_tx: UnboundedSender<Action>,
    pub current_sceen: Screen,
    // screens
    first_menu: FirstMenuComponent,
    solo_game_menu: SoloGamesMenuComponent,
    solo_speed_game: ScreenSoloSpeedGameComponent,
    settings_screen: SettingsScreenComponent,

}
impl ScreenRouterComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        let first_menu = FirstMenuComponent::new(dispatcher_tx.clone());
        let solo_game_menu = SoloGamesMenuComponent::new(dispatcher_tx.clone());
        let solo_speed_game = ScreenSoloSpeedGameComponent::new(dispatcher_tx.clone());
        let settings_screen = SettingsScreenComponent{};
        ScreenRouterComponent { dispatcher_tx, current_sceen: Screen::default(), first_menu, solo_game_menu, solo_speed_game, settings_screen }
    }

    fn close_current(&self){
        self.send(Action::ClosingScreen(self.current_sceen)).unwrap()
    }

    fn change_screen(&mut self, action: Action){
        match action {
            Action::AskChangeToScreen(asked_screen) if self.current_sceen != asked_screen => 
            {
                self.close_current();
                self.current_sceen = asked_screen;
                self.send(Action::OpeningScreen(asked_screen)).unwrap()
            } 
            _ => {panic!("It's not a action to change a screen")}
        }
    }

    fn updates_menus(&mut self, action: Action){
        self.first_menu.update(action);
        self.solo_game_menu.update(action);
        self.solo_speed_game.update(action);
        self.settings_screen.update(action);
    }

}

impl Store for ScreenRouterComponent {
    fn update(&mut self, action: crate::action::Action) {
        self.updates_menus(action);
        match action {
            Action::EscPressed => 
                if let Some(new_screen) = self.current_sceen.previous(){
                    self.send(Action::AskChangeToScreen(new_screen)).unwrap()
                } else {
                    self.send(Action::Exit).unwrap()
                }
            Action::StartingApp => self.send(Action::OpeningScreen(STARTING_SCREEN)).unwrap(),
            Action::AskChangeToScreen(_) => self.change_screen(action),
            _ => {}
        }
    }
} 


impl SendAction for ScreenRouterComponent{
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}


pub trait ScreenMember{
    fn screen(&self) -> Screen;
    fn deactivate(&mut self);
    fn activate(&mut self);
    fn handle_screen_activation(&mut self, action: Action)
    where
        Self: SendAction {
        match action {
            Action::OpeningScreen(screen) if screen == self.screen() => self.activate(),
            Action::ClosingScreen(screen) if screen == self.screen() => self.deactivate(),
            _ => {}
        }
    }
    fn update_screen_member(&mut self, action: Action) where
        Self: SendAction + Store {
        self.handle_screen_activation(action);
        self.update(action);
    }
}

impl Widget for &mut ScreenRouterComponent{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        match self.current_sceen {
            Screen::FirstMenu => self.first_menu.render(area, buf),
            Screen::SoloGamesMenu => self.solo_game_menu.render(area, buf),
            Screen::SoloSpeedGame => self.solo_speed_game.render(area, buf),
            Screen::Settings => self.settings_screen.render(area, buf),
        }
    }
}