use ratatui::widgets::{StatefulWidget, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    flux::SendAction,
    stores::Store,
    ui::{
        list::{MenuMultipleListComponent, MutipleEntry},
        screens::{IsScreen, menus::settings::SoloRaceSettingScreen},
    },
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloGamesMenu;

#[derive(Debug)]
pub struct SoloGamesMenuScreen {
    dispatcher_tx: UnboundedSender<Action>,
    list_store: MenuMultipleListComponent,
}
impl SoloGamesMenuScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        let entry_speed = MutipleEntry::new(
            ["Speed", "Settings"]
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
            vec![
                Action::AskChangeToScreen(Screen::SoloRaceGame),
                Action::AskChangeToScreen(Screen::SoloRaceSettingScreen),
            ],
        );
        let entry_clock = MutipleEntry::new(
            ["Clock", "Settings"]
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
            vec![Action::AskChangeToScreen(Screen::DebugMenu), Action::None],
        );
        let entry_inifinite = MutipleEntry::new(
            ["Infinite", "Settings"]
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
            vec![
                Action::AskChangeToScreen(Screen::SoloInfiniteGame),
                Action::None,
            ],
        );
        let list_store = MenuMultipleListComponent::new(
            "Select your game mod".to_string(),
            dispatcher_tx.clone(),
            vec![entry_speed, entry_clock, entry_inifinite],
            vec![Constraint::Percentage(100), Constraint::Length(10)],
            SCREEN,
        );
        SoloGamesMenuScreen {
            dispatcher_tx,
            list_store,
        }
    }
   
}

impl Store for SoloGamesMenuScreen {
    fn update(&mut self, action: Action) {
        self.list_store.update(action);
    }
}

impl Widget for &SoloGamesMenuScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect(70, 70, area);
        self.list_store.render(area, buf);
    }
}
impl SendAction for SoloGamesMenuScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for SoloGamesMenuScreen {}
