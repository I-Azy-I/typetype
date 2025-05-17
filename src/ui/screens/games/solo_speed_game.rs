use std::time::{Duration, Instant};

use ratatui::{layout, style::{Color, Style, Stylize}, widgets::{Block, BorderType, Borders, LineGauge, List, ListState, StatefulWidget, Widget}};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

use crate::{action::Action, flux::SendAction, stores::Store, ui::{clock::ClockWidget, gauge::{self, GaugeComponent}, list::{MenuList, MenuListComponent}, text::{SettingsText, TextWidgetComponent}}};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::SoloSpeedGame;
const PROGRESS_BAR: gauge::GaugeId = 0;

#[derive(Debug)]
pub struct ScreenSoloSpeedGameComponent {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>

}
impl ScreenSoloSpeedGameComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        ScreenSoloSpeedGameComponent {screen: SCREEN, dispatcher_tx, text_component: None, start_time: None}
    }
}

impl Store for  ScreenSoloSpeedGameComponent {
    fn update(&mut self, action: Action) {
        match action {
            Action::OpeningScreen(screen) if screen == self.screen => {
                self.send(Action::InitializeSoloSpeedGame).unwrap();
                self.text_component = Some(TextWidgetComponent::new(self.dispatcher_tx.clone(), self.screen))
            },
            Action::KeyPressed(_) if self.start_time.is_none() => self.start_time = Some(Instant::now()),
            _ => {}
        }
        if let Some(text) = self.text_component.as_mut() {
            text.update_screen_member(action);
        }
    }
}
impl SendAction for ScreenSoloSpeedGameComponent {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut ScreenSoloSpeedGameComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
            
        let v_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Percentage(100),
            ])
            .split(area);
        let block_info = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(6),
                Constraint::Percentage(100),
            ]).split(block_info.inner(v_layout[0]));

        block_info.render(v_layout[0], buf);

        let ratio = self.text_component.as_ref().map(|w| w.current_ratio() ).unwrap_or(0.0);
        let line_gauge = LineGauge::default()
            .ratio(ratio as f64)
            .filled_style(Style::default().fg(Color::Blue))
            .unfilled_style(Style::default().fg(Color::Red));
        line_gauge.render(info_layout[1], buf);
        
        let clock = if let Some(start_time) = self.start_time {
            ClockWidget::new(start_time.elapsed())
        } else {
            ClockWidget::default()
        };
        clock.render(info_layout[0], buf);
        let block_text = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        (&block_text).render(v_layout[1], buf);
        let text_area = block_text.inner(v_layout[1]);
        if let Some(text) = self.text_component.as_mut() {

            text.widget.render(text_area, buf, &mut SettingsText::ThreeLine);
        };
        
    }
}