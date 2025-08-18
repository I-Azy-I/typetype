use rand::prelude::*;
use ratatui::{
    text::Span,
    widgets::{Block, BorderType, Borders, StatefulWidget, Widget},
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
    settings::{OffsetText, Settings, TextOrigin},
    stores::Store,
    ui::{
        clock::ClockWidget,
        screens::IsScreen,
        text::{SettingsText, TextWidgetComponent},
    },
    win_data::{ClockData, GameMod, WinData},
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloClockGame;

#[derive(Debug)]
pub struct SoloClockGameScreen {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>,
    start_time: Option<Instant>,
    shr_settings: Rc<RefCell<Settings>>,
    shr_win_data: Rc<RefCell<WinData>>,
    time: Option<usize>,
    done: bool,
}
impl SoloClockGameScreen {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        shr_settings: Rc<RefCell<Settings>>,
        shr_win_data: Rc<RefCell<WinData>>,
    ) -> Self {
        SoloClockGameScreen {
            screen: SCREEN,
            dispatcher_tx,
            shr_settings,
            text_component: None,
            start_time: None,
            shr_win_data,
            time: None,
            done: false,
        }
    }
    pub fn load_settings(&mut self) {
        self.start_time = None;

        let (text_origin, time, seed, keep_seed, path_source, start_end) = {
            let seed = self.shr_settings.borrow().game_settings.seed;
            let keep_seed = self.shr_settings.borrow().game_settings.keep_seed;
            let clock_game_settings = &self.shr_settings.borrow().game_settings.clock_game_settings;
            let text_origin = clock_game_settings.text_origin.clone();
            let time = clock_game_settings.time;
            let path_source = clock_game_settings.filename.clone();
            let start_end = clock_game_settings.start_end_sentence;
            (text_origin, time, seed, keep_seed, path_source, start_end)
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
            TextOrigin::Text => { // TODO add beginning option
                let mut r = StdRng::seed_from_u64(seed);
                Some(r.random())
            }
            _ => None,
        };

        self.time = Some(time);

        self.text_component = Some(TextWidgetComponent::new(
            self.dispatcher_tx.clone(),
            self.screen,
            text_origin,
            path_source,
            None,
            offset,
            Some(start_end),
            Some(seed),
        ));
    }

    fn process_end(&mut self) {
        // edit data for end screen

        let game = GameMod::Clock(ClockData {
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
        });

        {
            let mut shr_win_data = self.shr_win_data.borrow_mut();
            shr_win_data.game = game;
        }

        self.start_time = None;
        self.text_component = None;
    }
}

impl Store for SoloClockGameScreen {
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
    }
}
impl SendAction for SoloClockGameScreen {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut SoloClockGameScreen {
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

        let text_title = Span::raw("COCK MOD").into_centered_line();
        text_title.render(info_layout[1], buf);

        let clock = if let Some(start_time) = self.start_time {
            if let Some(time_left) =
                Duration::from_secs(self.time.unwrap() as u64).checked_sub(start_time.elapsed())
            {
                ClockWidget::new(time_left)
            } else {
                self.done = true;
                self.send(Action::AskChangeToScreen(Screen::WinMenu))
                    .expect("to be able to change screen"); // a bit wierd to be in the render part put there is no real game loop elsewhere
                ClockWidget::default()
            }
        } else {
            ClockWidget::new(Duration::from_secs(self.time.unwrap() as u64))
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

impl IsScreen for SoloClockGameScreen {
    fn open(&mut self) {
        self.done = false;
        self.load_settings();
    }
    fn close(&mut self) {
        self.process_end();
    }
}
