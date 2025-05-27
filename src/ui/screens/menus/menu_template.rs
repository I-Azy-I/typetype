use ratatui::{style::{Style, Stylize}, widgets::{Block, List, ListState, StatefulWidget, Widget}};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

use crate::{action::Action, flux::SendAction, stores::Store, ui::list::{MenuList, MenuListComponent}};

use super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen;

#[derive(Debug)]
pub struct ScreenComponent {
    a_component: Component
}
impl ScreenComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        SoloGamesMenuComponent {a_component}
    }
}

impl Store for  ScreenComponent {
    fn update(&mut self, action: Action) {
        self.a_component.update_screen_member(action);
    }
}

impl Widget for &ScreenComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        self.a_component.render(area, buf);
    }
}