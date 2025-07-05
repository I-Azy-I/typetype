use ratatui::widgets::{StatefulWidget, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    stores::Store,
    ui::list::{MenuMultipleListComponent, MutipleEntry},
};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::SoloGamesMenu;

#[derive(Debug)]
pub struct SoloGamesMenuComponent {
    list_store: MenuMultipleListComponent,
}
impl SoloGamesMenuComponent {
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
            dispatcher_tx,
            vec![entry_speed, entry_clock, entry_inifinite],
            vec![Constraint::Percentage(100), Constraint::Length(10)],
            SCREEN,
        );
        SoloGamesMenuComponent { list_store }
    }
}

impl Store for SoloGamesMenuComponent {
    fn update(&mut self, action: Action) {
        self.list_store.update_screen_member(action);
    }
}

impl Widget for &SoloGamesMenuComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect(70, 70, area);
        self.list_store.render(area, buf);
    }
}
