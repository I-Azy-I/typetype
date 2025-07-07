use std::{cell::RefCell, fmt::Debug, rc::Rc};

use games::solo_race_game::ScreenSoloRaceGameComponent;
use log::{error, warn};
use menus::solo_games_menu::SoloGamesMenuComponent;
use menus::{first_menu::FirstMenuComponent, settings::SoloRaceSettingScreen};
use ratatui::widgets::Widget;
use tokio::sync::mpsc::UnboundedSender;

pub mod games;
mod menus;

use crate::ui::screens::games::solo_infinite_game::ScreenSoloInfiniteGameComponent;
use crate::ui::screens::menus::debug_menu::DebugMenuComponent;
use crate::ui::screens::menus::win_menu::{self, WinMenuComponent};
use crate::win_data::WinData;
use crate::{action::Action, flux::SendAction, settings::Settings, stores::Store};

const STARTING_SCREEN: Screen = Screen::FirstMenu;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    #[default]
    FirstMenu,
    SoloGamesMenu,
    SoloRaceGame, // true ask to change the seed
    SoloRaceSettingScreen,
    SoloInfiniteGame,
    WinMenu,
    DebugMenu,
}
impl Screen {
    fn previous(self) -> Option<Screen> {
        match self {
            Screen::FirstMenu => None,
            Screen::SoloGamesMenu => Some(Screen::FirstMenu),
            Screen::SoloRaceGame => Some(Screen::FirstMenu),
            Screen::SoloRaceSettingScreen => Some(Screen::SoloGamesMenu),
            Screen::SoloInfiniteGame => Some(Screen::SoloGamesMenu),
            Screen::DebugMenu => Some(Screen::FirstMenu),
            Screen::WinMenu => Some(Screen::FirstMenu),
        }
    }
}

#[derive(Debug)]
pub struct ScreenRouterComponent {
    dispatcher_tx: UnboundedSender<Action>,
    pub current_sceen_kind: Screen,
    settings: Rc<RefCell<Settings>>,
    // screens
    first_menu: FirstMenuComponent,
    solo_game_menu: SoloGamesMenuComponent,
    solo_speed_game: ScreenSoloRaceGameComponent,
    setting_solo_race: SoloRaceSettingScreen,
    solo_infinte_game: ScreenSoloInfiniteGameComponent,

    win_menu: WinMenuComponent,
    debug_menu: DebugMenuComponent,
}
impl ScreenRouterComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let data_end_game = Rc::new(RefCell::new(WinData::default()));

        let debug_menu = DebugMenuComponent::new();
        let first_menu = FirstMenuComponent::new(dispatcher_tx.clone());
        let solo_game_menu = SoloGamesMenuComponent::new(dispatcher_tx.clone());
        // solo speed
        let solo_speed_game = ScreenSoloRaceGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            data_end_game.clone(),
        );
        let setting_solo_race = SoloRaceSettingScreen::new(dispatcher_tx.clone(), settings.clone());
        // solo inifinite
        let solo_infinte_game = ScreenSoloInfiniteGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            data_end_game.clone(),
        );

        let win_menu = WinMenuComponent::new(
            dispatcher_tx.clone(),
            data_end_game.clone(),
            settings.clone(),
        );
        ScreenRouterComponent {
            dispatcher_tx,
            settings,
            current_sceen_kind: Screen::default(),
            first_menu,
            solo_game_menu,
            solo_speed_game,
            setting_solo_race,
            solo_infinte_game,

            win_menu,
            debug_menu,
        }
    }

    fn close_current(&self) {
        self.send(Action::ClosingScreen(self.current_sceen_kind))
            .unwrap()
    }

    fn change_screen(&mut self, action: Action) {
        match action {
            Action::AskChangeToScreen(asked_screen) if self.current_sceen_kind != asked_screen => {
                self.close_current();
                self.current_sceen_kind = asked_screen;
                self.send(Action::OpeningScreen(asked_screen)).unwrap()
            }
            Action::AskChangeToScreen(asked_screen) => {
                warn!(
                    "Try to change screen {:?} with {:?} which are the same, action is ignored",
                    self.current_sceen_kind, asked_screen
                );
            }
            _ => panic!("It's not a action to change a screen"),
        }
    }

    fn updates_screens(&mut self, action: Action) {
        match self.current_sceen_kind {
            Screen::FirstMenu => self.first_menu.update(action),
            Screen::SoloGamesMenu => self.solo_game_menu.update(action),
            Screen::SoloRaceGame => self.solo_speed_game.update(action),
            Screen::SoloRaceSettingScreen => self.setting_solo_race.update(action),
            Screen::SoloInfiniteGame => self.solo_infinte_game.update(action),
            Screen::DebugMenu => self.debug_menu.update(action),
            Screen::WinMenu => self.win_menu.update(action),
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
            Action::StartingApp => self.send(Action::OpeningScreen(STARTING_SCREEN)).unwrap(),
            Action::AskChangeToScreen(_) => self.change_screen(action),
            _ => {}
        }
    }
}

impl SendAction for ScreenRouterComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

pub trait ScreenMember {
    fn screen(&self) -> Screen;
    fn deactivate(&mut self);
    fn activate(&mut self);
    fn handle_screen_activation(&mut self, action: Action)
    where
        Self: SendAction,
    {
        match action {
            Action::OpeningScreen(screen) if screen == self.screen() => self.activate(),
            Action::ClosingScreen(screen) if screen == self.screen() => self.activate(),
            _ => {}
        }
    }
    fn update_screen_member(&mut self, action: Action)
    where
        Self: SendAction + Store,
    {
        self.handle_screen_activation(action);
        self.update(action);
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
            Screen::SoloRaceSettingScreen => self.setting_solo_race.render(area, buf),
            Screen::SoloInfiniteGame => self.solo_infinte_game.render(area, buf),
            Screen::DebugMenu => self.debug_menu.render(area, buf),
            Screen::WinMenu => self.win_menu.render(area, buf),
        }
    }
}
