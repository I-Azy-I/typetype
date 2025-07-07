use std::mem;

use ratatui::widgets::{LineGauge, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, flux::SendAction, stores::Store};

use super::screens::Screen;

pub type GaugeId = u16;

pub struct GaugeComponent<'a> {
    id: GaugeId,
    dispatcher_tx: UnboundedSender<Action>,
    screen: Screen,
    is_active: bool,
    widget: LineGauge<'a>,
}

impl<'a> GaugeComponent<'a> {
    pub fn new(
        line_gauge: LineGauge<'a>,
        id: GaugeId,
        screen: Screen,
        dispatcher_tx: UnboundedSender<Action>,
    ) -> Self {
        GaugeComponent {
            id,
            dispatcher_tx,
            screen,
            widget: line_gauge,
            is_active: false,
        }
    }
    pub fn get_widget(&self) -> &'a LineGauge {
        &self.widget
    }
}

impl<'a> SendAction for GaugeComponent<'a> {
    fn send(
        &self,
        action: crate::action::Action,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<crate::action::Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl<'a> Store for GaugeComponent<'a> {
    fn update(&mut self, action: Action) {
        match action {
            _ => {}
        }
    }
}

impl<'a> Widget for &GaugeComponent<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        (&self.widget).render(area, buf);
    }
}
