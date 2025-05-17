use ratatui::widgets::{StatefulWidget, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, stores::Store, ui::list::MenuListComponent};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::SoloGamesMenu;

#[derive(Debug)]
pub struct SoloGamesMenuComponent {
    list_store: MenuListComponent
}
impl SoloGamesMenuComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        let options = ["Clock", "Speed", "Zen", "Infinite"].into_iter().map(|el|el.to_string()).collect();
        let actions = vec![Action::None, Action::AskChangeToScreen(Screen::SoloSpeedGame), Action::None, Action::None];
        let list_store = MenuListComponent::new("Select your game mod".to_string(), dispatcher_tx, options, actions, SCREEN);

        SoloGamesMenuComponent {list_store}
    }
}

impl Store for  SoloGamesMenuComponent {
    fn update(&mut self, action: Action) {
        self.list_store.update_screen_member(action);
    }
}

impl Widget for &SoloGamesMenuComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        let area = centered_rect(70, 70, area);
        self.list_store.render(area, buf);
    }
}