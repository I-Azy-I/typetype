use ratatui::{
    text::Span,
    widgets::{StatefulWidget, Widget},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, dispatcher, flux::SendAction, stores::Store, ui::screens::IsScreen};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::DebugMenu;

#[derive(Debug)]
pub struct DebugMenuScreen {
    dispatcher_tx: UnboundedSender<Action>,
}
impl DebugMenuScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        DebugMenuScreen { dispatcher_tx }
    }
}

impl Store for DebugMenuScreen {
    fn update(&mut self, _action: Action) {}
}

impl Widget for &DebugMenuScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect(70, 70, area);
        let text = Span::raw("debug");
        text.render(area, buf);
    }
}

impl SendAction for DebugMenuScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl IsScreen for DebugMenuScreen {}
