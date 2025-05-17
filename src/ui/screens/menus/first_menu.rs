use ratatui::widgets::{StatefulWidget, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, stores::Store, ui::list::MenuListComponent};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::FirstMenu;

#[derive(Debug)]
pub struct FirstMenuComponent {
    list_store: MenuListComponent
}
impl FirstMenuComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        let options = ["Solo", "Multi (in progress)", "Settings", "About"].into_iter().map(|el|el.to_string()).collect();
        let actions = vec![Action::AskChangeToScreen(Screen::SoloGamesMenu), Action::None, Action::AskChangeToScreen(Screen::Settings), Action::None];
        let list_store = MenuListComponent::new("Menu".to_string(), dispatcher_tx, options, actions, SCREEN);

        FirstMenuComponent {list_store}
    }
}

impl Store for  FirstMenuComponent {
    fn update(&mut self, action: Action) {
        self.list_store.update_screen_member(action);
    }
}

impl Widget for &FirstMenuComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        let area = centered_rect(70, 70, area);
        self.list_store.render(area, buf);
    }
}