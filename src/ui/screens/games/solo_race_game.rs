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
    settings::{OffsetText, Settings, StartingPointSentence, TextOrigin},
    stores::Store,
    ui::{
        clock::ClockWidget,
        screens::IsScreen,
        text::{SettingsText, TextWidgetComponent},
    },
    win_data::{GameMod, RaceData, WinData},
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloRaceGame;

#[derive(Debug)]
pub struct SoloRaceGameScreen {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>,
    shr_settings: Rc<RefCell<Settings>>,
    shr_win_data: Rc<RefCell<WinData>>,
    done: bool,
}
impl SoloRaceGameScreen {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        shr_settings: Rc<RefCell<Settings>>,
        shr_win_data: Rc<RefCell<WinData>>,
    ) -> Self {
        SoloRaceGameScreen {
            screen: SCREEN,
            dispatcher_tx,
            shr_settings,
            text_component: None,
            start_time: None,
            shr_win_data,
            done: false,
        }
    }
    pub fn load_settings(&mut self) {
        self.start_time = None;

        let (text_origin, number_words, seed, keep_seed, path_source, start_end, starting_point) = {
            let settings = self.shr_settings.borrow();
            let race_settings = &settings.game_settings.race_game_settings;
            (
                race_settings.text_origin.clone(),
                race_settings.number_words,
                settings.game_settings.seed,
                settings.game_settings.keep_seed,
                race_settings.filename.clone(),
                race_settings.start_end_sentence,
                race_settings.starting_point,
            )
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
            TextOrigin::Text => match starting_point {
                StartingPointSentence::Beginning => Some(0.0),
                StartingPointSentence::Random => {
                    let mut r = StdRng::seed_from_u64(seed);
                    Some(r.random())
                }
            },
            _ => None,
        };

        self.text_component = Some(TextWidgetComponent::new(
            self.dispatcher_tx.clone(),
            self.screen,
            text_origin,
            path_source,
            Some(number_words),
            offset,
            Some(start_end),
            Some(seed),
        ));
    }

    fn process_end(&mut self) {
        // edit data for end screen
        let game = GameMod::Race(RaceData {
            skip: !self.done,
            time: Instant::now() - self.start_time.unwrap_or(Instant::now()),
            n_words: self
                .text_component
                .as_ref()
                .map(|text| text.get_n_words_correctly_typed())
                .unwrap_or(0),
        });

        self.shr_win_data.borrow_mut().game = game;

        self.start_time = None;
        self.text_component = None;
    }
}

impl Store for SoloRaceGameScreen {
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
            if text_component.is_done_correctly() && !self.done {
                self.done = true;
                self.send(Action::AskChangeToScreen(Screen::WinMenu))
                    .expect("to be able to change screen");
            }
        }
    }
}
impl SendAction for SoloRaceGameScreen {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut SoloRaceGameScreen {
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

impl IsScreen for SoloRaceGameScreen {
    fn open(&mut self) {
        self.done = false;
        self.load_settings();
    }
    fn close(&mut self) {
        self.process_end();
    }
}
