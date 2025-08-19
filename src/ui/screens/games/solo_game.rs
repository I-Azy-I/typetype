use log::debug;
use rand::prelude::*;
use ratatui::{
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, LineGauge, StatefulWidget, Widget},
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::{
    action::Action,
    flux::SendAction,
    settings::{OffsetText, Settings, StartingPointSentence, TextOrigin},
    stores::Store,
    ui::{
        clock::ClockWidget,
        screens::{IsScreen, games::GameMod},
        text::{SettingsText, TextWidgetComponent},
    },
    win_data::{ClockData, GameModEndResult, RaceData, WinData},
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloRaceGame;

#[derive(Debug, Clone, Copy, Default)]
enum GameModParam {
    Race {
        n_words: usize,
    },
    Clock {
        time: usize,
    },
    #[default]
    Infinite,
}

#[derive(Debug)]
pub struct SoloGameScreen {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    game_mod: GameMod,
    game_mod_params: GameModParam,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>,
    shr_settings: Rc<RefCell<Settings>>,
    shr_win_data: Rc<RefCell<WinData>>,
    done: bool,
}
impl SoloGameScreen {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        shr_settings: Rc<RefCell<Settings>>,
        shr_win_data: Rc<RefCell<WinData>>,
        game_mod: GameMod,
    ) -> Self {
        SoloGameScreen {
            screen: SCREEN,
            dispatcher_tx,
            game_mod,
            game_mod_params: GameModParam::default(),
            shr_settings,
            text_component: None,
            start_time: None,
            shr_win_data,
            done: false,
        }
    }
    pub fn load_settings(&mut self) {
        self.start_time = None;

        let (text_origin, game_mod_params, seed, keep_seed, path_source) = {
            let settings = self.shr_settings.borrow();
            let text_settings = &settings.game_settings.text_settings;
            let game_mod_params = match self.game_mod {
                GameMod::Race => GameModParam::Race {
                    n_words: settings.game_settings.race_game_settings.number_words,
                },
                GameMod::Clock => GameModParam::Clock {
                    time: settings.game_settings.clock_game_settings.time,
                },
                GameMod::Infinite => GameModParam::Infinite,
            };

            (
                text_settings.text_origin.clone(),
                game_mod_params,
                settings.game_settings.seed,
                settings.game_settings.keep_seed,
                text_settings.filename.clone(),
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
            TextOrigin::Text {
                start_end_sentence,
                starting_point,
            } => match starting_point {
                StartingPointSentence::Beginning => Some(0.0),
                StartingPointSentence::Random => {
                    let mut r = StdRng::seed_from_u64(seed);
                    Some(r.random())
                }
            },
            _ => None,
        };

        if let Some(path_source) = path_source {
            let n_words = if let GameModParam::Race { n_words } = game_mod_params {
                Some(n_words)
            } else {
                None
            };
            self.text_component = Some(TextWidgetComponent::new(
                self.dispatcher_tx.clone(),
                self.screen,
                text_origin,
                path_source,
                n_words,
                offset,
                Some(seed),
            ));
        }

        self.game_mod_params = game_mod_params;

        match self.game_mod_params {
            GameModParam::Race { n_words: _ } => assert!(matches!(self.game_mod, GameMod::Race)),
            GameModParam::Clock { time: _ } => assert!(matches!(self.game_mod, GameMod::Clock)),
            GameModParam::Infinite => assert!(matches!(self.game_mod, GameMod::Infinite)),
        }
    }

    fn process_end(&mut self) {
        let game_result = match self.game_mod_params {
            GameModParam::Race { n_words } => GameModEndResult::Race(RaceData {
                skip: !self.done,
                time: Instant::now() - self.start_time.unwrap_or(Instant::now()),
                n_words: self
                    .text_component
                    .as_ref()
                    .map(|text| text.get_n_words_correctly_typed())
                    .unwrap_or(0),
            }),
            GameModParam::Clock { time } => GameModEndResult::Clock(ClockData {
                skip: !self.done,
                time: self
                    .start_time
                    .unwrap_or(Instant::now())
                    .elapsed()
                    .as_secs() as usize,
                n_words: self
                    .text_component
                    .as_ref()
                    .unwrap()
                    .get_n_words_correctly_typed(),
            }),
            GameModParam::Infinite => todo!(),
        };
        // edit data for end screen

        self.shr_win_data.borrow_mut().game = game_result;

        self.start_time = None;
        self.text_component = None;
    }
    fn render_race(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        assert!(matches!(self.game_mod_params, GameModParam::Race { .. }));

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

    fn render_clock(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        assert!(matches!(self.game_mod_params, GameModParam::Clock { .. }));

        let time = match self.game_mod_params {
            GameModParam::Clock { time } => time,
            _ => unreachable!(),
        };
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

        let text_title = Span::raw("COCK MOD").into_centered_line();
        text_title.render(info_layout[1], buf);

        let clock = if let Some(start_time) = self.start_time {
            if let Some(time_left) =
                Duration::from_secs(time as u64).checked_sub(start_time.elapsed())
            {
                ClockWidget::new(time_left)
            } else {
                self.done = true;
                self.send(Action::AskChangeToScreen(Screen::WinMenu))
                    .expect("to be able to change screen"); // a bit wierd to be in the render part put there is no real game loop elsewhere
                ClockWidget::default()
            }
        } else {
            ClockWidget::new(Duration::from_secs(time as u64))
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
        } else {
            todo!("Add text if impossible to load file")
        };
    }

    fn render_infinite(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
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

        let text = Span::raw("INFINITE MOD").into_centered_line();
        text.render(info_layout[1], buf);
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

impl Store for SoloGameScreen {
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
        if matches!(self.game_mod, GameMod::Race) {
            if let Some(text_component) = self.text_component.as_ref() {
                if text_component.is_done_correctly() && !self.done {
                    self.done = true;
                    self.send(Action::AskChangeToScreen(Screen::WinMenu))
                        .expect("to be able to change screen");
                }
            }
        }
    }
}
impl SendAction for SoloGameScreen {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut SoloGameScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.game_mod {
            GameMod::Race => self.render_race(area, buf),
            GameMod::Clock => self.render_clock(area, buf),
            GameMod::Infinite => self.render_infinite(area, buf),
        }
    }
}

impl IsScreen for SoloGameScreen {
    fn open(&mut self) {
        self.done = false;
        self.load_settings();
    }
    fn close(&mut self) {
        self.process_end();
    }
}
