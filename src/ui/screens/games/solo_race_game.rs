use std::{cell::RefCell, rc::Rc, time::Instant};

use ratatui::{style::{Color, Style, Stylize}, widgets::{Block, BorderType, Borders, LineGauge, StatefulWidget, Widget}};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

use crate::{action::Action, flux::SendAction, settings::{self, RaceGameSettings, Settings}, stores::Store, ui::{clock::ClockWidget, gauge::{self}, text::{SettingsText, TextWidgetComponent}}};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::SoloRaceGame;

#[derive(Debug)]
pub struct ScreenSoloRaceGameComponent {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>,
    settings: Rc<RefCell<Settings>>
}
impl ScreenSoloRaceGameComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self{
        ScreenSoloRaceGameComponent {screen: SCREEN, dispatcher_tx, settings, text_component: None, start_time: None}
    }
    pub fn reset(&mut self){
        self.start_time = None;
       
        self.text_component = Some(TextWidgetComponent::new(self.dispatcher_tx.clone(), self.screen, self.settings.borrow().game_settings.race_game_settings.text_origin.clone()));
    }
}

impl Store for  ScreenSoloRaceGameComponent {
    fn update(&mut self, action: Action) {
        match action {
            Action::OpeningScreen(screen) if screen == self.screen => {
                self.send(Action::InitializeSoloSpeedGame).unwrap();
                self.reset();
                
            },
            Action::KeyPressed(_) if self.start_time.is_none() => self.start_time = Some(Instant::now()),
            _ => {}
        }
        if let Some(text) = self.text_component.as_mut() {
            text.update_screen_member(action);
        }
    }
}
impl SendAction for ScreenSoloRaceGameComponent {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut ScreenSoloRaceGameComponent {
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