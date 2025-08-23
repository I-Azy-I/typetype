use std::{cell::RefCell, fmt::Debug, rc::Rc};

use games::solo_game::SoloGameScreen;
use log::warn;
use menus::first_menu::FirstMenuScreen;
use menus::solo_games_menu::SoloGamesMenuScreen;
use ratatui::widgets::Widget;
use tokio::sync::mpsc::UnboundedSender;

pub mod games;
mod menus;

use crate::ui::screens::menus::debug_menu::DebugMenuScreen;
use crate::ui::screens::menus::settings::TextSettingScreen;
use crate::ui::screens::menus::win_menu::WinMenuScreen;
use crate::win_data::WinData;
use crate::{action::Action, flux::SendAction, settings::Settings, stores::Store};

const STARTING_SCREEN: Screen = Screen::FirstMenu;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    #[default]
    FirstMenu,
    SoloGamesMenu,
    SoloRaceGame,

    SoloInfiniteGame,

    SoloClockGame,
    SoloGameSettings,
    WinMenu,

    DebugMenu,
}
impl Screen {
    fn previous(self) -> Option<Screen> {
        match self {
            Screen::FirstMenu => None,
            Screen::SoloGamesMenu => Some(Screen::FirstMenu),
            Screen::SoloRaceGame => Some(Screen::WinMenu),
            Screen::SoloInfiniteGame => Some(Screen::WinMenu),
            Screen::SoloGameSettings => Some(Screen::SoloGamesMenu),
            Screen::DebugMenu => Some(Screen::FirstMenu),
            Screen::WinMenu => Some(Screen::SoloGamesMenu),
            Screen::SoloClockGame => Some(Screen::WinMenu),
        }
    }
}

#[derive(Debug)]
pub struct ScreenRouterComponent {
    dispatcher_tx: UnboundedSender<Action>,
    pub current_sceen_kind: Screen,
    // screens
    first_menu: FirstMenuScreen,
    solo_game_menu: SoloGamesMenuScreen,

    solo_speed_game: SoloGameScreen,

    solo_infinte_game: SoloGameScreen,

    solo_clock_game: SoloGameScreen,

    solo_text_settings: TextSettingScreen,
    win_menu: WinMenuScreen,
    debug_menu: DebugMenuScreen,
}

impl ScreenRouterComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let data_end_game = Rc::new(RefCell::new(WinData::default()));

        let debug_menu = DebugMenuScreen::new(dispatcher_tx.clone());
        let first_menu = FirstMenuScreen::new(dispatcher_tx.clone());
        let solo_game_menu = SoloGamesMenuScreen::new(dispatcher_tx.clone());
        // solo speed
        let solo_speed_game = SoloGameScreen::new(
            dispatcher_tx.clone(),
            settings.clone(),
            data_end_game.clone(),
            games::GameMod::Race,
        );
        // solo inifinite
        let solo_infinte_game = SoloGameScreen::new(
            dispatcher_tx.clone(),
            settings.clone(),
            data_end_game.clone(),
            games::GameMod::Infinite,
        );

        let solo_clock_game = SoloGameScreen::new(
            dispatcher_tx.clone(),
            settings.clone(),
            data_end_game.clone(),
            games::GameMod::Clock,
        );

        let solo_text_settings = TextSettingScreen::new(dispatcher_tx.clone(), settings.clone());
        let win_menu = WinMenuScreen::new(
            dispatcher_tx.clone(),
            data_end_game.clone(),
            settings.clone(),
        );
        ScreenRouterComponent {
            dispatcher_tx,
            current_sceen_kind: STARTING_SCREEN,
            first_menu,
            solo_game_menu,
            solo_speed_game,
            solo_infinte_game,
            solo_clock_game,
            solo_text_settings,
            win_menu,
            debug_menu,
        }
    }

    fn change_screen_protocol(&mut self, action: Action) {
        match action {
            Action::AskChangeToScreen(asked_screen) if self.current_sceen_kind != asked_screen => {
                self.send(Action::AskCloseScreen(asked_screen)).unwrap();
            }
            Action::ScreenClosed(asked_screen) => {
                self.current_sceen_kind = asked_screen;
                self.send(Action::AskOpenScreen).unwrap()
            }

            Action::AskChangeToScreen(asked_screen) => {
                warn!(
                    "Try to change screen {:?} with {:?} which are the same, action is ignored",
                    self.current_sceen_kind, asked_screen
                );
            }
            _ => {}
        }
    }

    fn updates_screens(&mut self, action: Action) {
        match self.current_sceen_kind {
            Screen::FirstMenu => self.first_menu.update_screen(action),
            Screen::SoloGamesMenu => self.solo_game_menu.update_screen(action),
            Screen::SoloRaceGame => self.solo_speed_game.update_screen(action),

            Screen::SoloInfiniteGame => self.solo_infinte_game.update_screen(action),

            Screen::SoloClockGame => self.solo_clock_game.update_screen(action),

            Screen::DebugMenu => self.debug_menu.update_screen(action),
            Screen::WinMenu => self.win_menu.update_screen(action),
            Screen::SoloGameSettings => self.solo_text_settings.update_screen(action),
        }
    }
}

impl Store for ScreenRouterComponent {
    fn update(&mut self, action: crate::action::Action) {
        self.updates_screens(action);
        match action {
            Action::EscPressed => {
                if let Some(new_screen) = self.current_sceen_kind.previous() {
                    self.send(Action::AskChangeToScreen(new_screen)).unwrap()
                } else {
                    self.send(Action::Exit).unwrap()
                }
            }
            Action::StartingApp => self.send(Action::AskOpenScreen).unwrap(),
            _ => {}
        }
        self.change_screen_protocol(action);
    }
}

impl SendAction for ScreenRouterComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl Widget for &mut ScreenRouterComponent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.current_sceen_kind {
            Screen::FirstMenu => self.first_menu.render(area, buf),
            Screen::SoloGamesMenu => self.solo_game_menu.render(area, buf),
            Screen::SoloRaceGame => self.solo_speed_game.render(area, buf),
            Screen::SoloInfiniteGame => self.solo_infinte_game.render(area, buf),
            Screen::SoloClockGame => self.solo_clock_game.render(area, buf),
            Screen::DebugMenu => self.debug_menu.render(area, buf),
            Screen::WinMenu => self.win_menu.render(area, buf),
            Screen::SoloGameSettings => self.solo_text_settings.render(area, buf),
        }
    }
}

pub trait IsScreen: Store + SendAction {
    fn open(&mut self) {}
    fn close(&mut self) {}
    fn update_screen(&mut self, action: Action) {
        self.update(action);
        match action {
            Action::AskCloseScreen(asked_screen) => {
                self.close();
                self.send(Action::ScreenClosed(asked_screen))
                    .expect("to notify that the screen is closed")
            }
            Action::AskOpenScreen => self.open(),
            _ => {}
        }
    }
}
