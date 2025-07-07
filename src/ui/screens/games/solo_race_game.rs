use log::debug;
use rand::prelude::*;
use ratatui::{
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, LineGauge, StatefulWidget, Widget},
};
use std::{cell::RefCell, rc::Rc, time::Instant};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::{
    action::Action,
    flux::SendAction,
    settings::{OffsetText, Settings, TextOrigin},
    stores::Store,
    ui::{
        clock::ClockWidget,
        screens::IsScreen,
        text::{SettingsText, TextStartEnd, TextWidgetComponent},
    },
    win_data::{GameMod, RaceData, WinData},
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloRaceGame;

#[derive(Debug)]
pub struct ScreenSoloRaceGameScreen {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>,
    shr_settings: Rc<RefCell<Settings>>,
    shr_win_data: Rc<RefCell<WinData>>,
}
impl ScreenSoloRaceGameScreen {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        shr_settings: Rc<RefCell<Settings>>,
        shr_win_data: Rc<RefCell<WinData>>,
    ) -> Self {
        ScreenSoloRaceGameScreen {
            screen: SCREEN,
            dispatcher_tx,
            shr_settings,
            text_component: None,
            start_time: None,
            shr_win_data,
        }
    }
    pub fn load_settings(&mut self) {
        self.start_time = None;

        let (text_origin, number_words, seed, keep_seed, path_source) = {
            let seed = self.shr_settings.borrow().game_settings.seed;
            let keep_seed = self.shr_settings.borrow().game_settings.keep_seed;
            let race_game_settings = &self.shr_settings.borrow().game_settings.race_game_settings;
            let text_origin = race_game_settings.text_origin.clone();
            let number_words = race_game_settings.number_words;
            let path_source = race_game_settings.file_name.clone();
            (text_origin, number_words, seed, keep_seed, path_source)
        };

        let seed = if !keep_seed || seed.is_none() {
            let mut rng = rand::rng();
            let new_seed: u64 = rng.random();
            self.shr_settings.borrow_mut().game_settings.seed = Some(new_seed);
            new_seed
        } else {
            seed.unwrap()
        };

        let offset = match text_origin {
            TextOrigin::Text(OffsetText::Random) => {
                let mut r = StdRng::seed_from_u64(seed);
                Some(r.random())
            }
            _ => None,
        };

        self.text_component = Some(TextWidgetComponent::new(
            self.dispatcher_tx.clone(),
            self.screen,
            text_origin,
            path_source,
            Some(number_words),
            offset,
            Some(TextStartEnd::Start),
            Some(seed),
        ));
    }

    fn process_end(&mut self) {
        // edit data for end screen

        let game = GameMod::Race(RaceData {
            time: Instant::now() - self.start_time.expect("time to have stated"),
            n_words: self
                .shr_settings
                .borrow()
                .game_settings
                .race_game_settings
                .number_words,
        });

        {
            let mut shr_win_data = self.shr_win_data.borrow_mut();
            shr_win_data.game = game;
        }
        self.send(Action::AskChangeToScreen(Screen::WinMenu))
            .expect("to be able to change screen");
        self.start_time = None;
        self.text_component = None;
    }
}

impl Store for ScreenSoloRaceGameScreen {
    fn update(&mut self, action: Action) {
        match action {
            Action::KeyPressed(_) if self.start_time.is_none() => {
                self.start_time = Some(Instant::now())
            }
            _ => {}
        }
        if let Some(text) = self.text_component.as_mut() {
            text.update(action);
        }
        if let Some(text_component) = self.text_component.as_ref() {
            debug!("{}", text_component.is_done_correctly());
            if text_component.is_done_correctly() {
                self.process_end();
            }
        }
    }
}
impl SendAction for ScreenSoloRaceGameScreen {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut ScreenSoloRaceGameScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let v_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(100)])
            .split(area);
        let block_info = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(6),
                Constraint::Percentage(100),
                Constraint::Length(8),
            ])
            .split(block_info.inner(v_layout[0]));

        block_info.render(v_layout[0], buf);

        let ratio = self
            .text_component
            .as_ref()
            .map(|w| w.current_ratio())
            .unwrap_or(0.0);
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

        let wpm = self.text_component.as_ref().map(|w| w.wpm()).unwrap_or(0.0);
        let txt_wpm = format!(" {:.0} w/s", wpm);
        let span_wpm = Span::raw(txt_wpm);
        span_wpm.render(info_layout[2], buf);

        let block_text = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        (&block_text).render(v_layout[1], buf);
        let text_area = block_text.inner(v_layout[1]);
        if let Some(text) = self.text_component.as_mut() {
            text.widget
                .render(text_area, buf, &mut SettingsText::ThreeLine);
        };
    }
}

impl IsScreen for ScreenSoloRaceGameScreen {
    fn open(&mut self) {
        self.load_settings();
    }
}
