use ratatui::{
    text::Span,
    widgets::{StatefulWidget, Widget},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, stores::Store, ui::list::MenuListComponent};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::FirstMenu;

#[derive(Debug)]
pub struct DebugMenuComponent {}
impl DebugMenuComponent {
    pub fn new() -> Self {
        DebugMenuComponent {}
    }
}

impl Store for DebugMenuComponent {
    fn update(&mut self, _action: Action) {}
}

impl Widget for &DebugMenuComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect(70, 70, area);
        let text = Span::raw("debug");
        text.render(area, buf);
    }
}
