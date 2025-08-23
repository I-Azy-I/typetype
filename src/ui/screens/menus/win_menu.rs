use std::{cell::RefCell, rc::Rc};

use log::debug;
use ratatui::{
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, ListState, Widget},
};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::{
    action::Action,
    flux::SendAction,
    settings::Settings,
    stores::Store,
    ui::{list::HorizontalList, screens::IsScreen},
    win_data::{ClockData, InfiniteData, RaceData, WinData},
};

use super::super::{super::*, Screen};

#[derive(Debug, Default, Clone, Copy)]
enum SelectedOption {
    Leave,
    #[default]
    Same,
    Next,
}
impl SelectedOption {
    fn next(&self) -> SelectedOption {
        match self {
            Self::Leave => SelectedOption::Same,
            Self::Same => SelectedOption::Next,
            Self::Next => SelectedOption::Next,
        }
    }
    fn previous(&self) -> SelectedOption {
        match self {
            Self::Leave => SelectedOption::Leave,
            Self::Same => SelectedOption::Leave,
            Self::Next => SelectedOption::Same,
        }
    }

    fn get_pos(&self) -> usize {
        match self {
            Self::Leave => 0,
            Self::Same => 1,
            Self::Next => 2,
        }
    }
}

#[derive(Debug)]
pub struct WinMenuScreen {
    shr_win_data: Rc<RefCell<WinData>>,
    shr_settings: Rc<RefCell<Settings>>,
    dispatcher_tx: UnboundedSender<Action>,
    choice_state: SelectedOption,
}
impl WinMenuScreen {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        shr_win_data: Rc<RefCell<WinData>>,
        shr_settings: Rc<RefCell<Settings>>,
    ) -> Self {
        WinMenuScreen {
            shr_win_data,
            shr_settings,
            dispatcher_tx,
            choice_state: SelectedOption::default(),
        }
    }

    fn win_screen_race(
        &self,
        race_data: &RaceData,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        if race_data.skip {
            let centered_area = centered_rect_with_length(area.width, 1, area);
            let text = "Game Skipped";
            let line = Span::raw(text).into_centered_line();
            line.render(centered_area, buf);
        } else {
            let centered_area = centered_rect_with_length(area.width, 2, area);

            let time = race_data.time;
            let average_wpm = (race_data.n_words as f32) * 60.0 / race_data.time.as_secs_f32();

            let text_time =
                Span::raw(format!("time: {:.2} seconds", time.as_secs_f32())).into_centered_line();
            let text_average_wpm =
                Span::raw(format!("average wpm: {:.0}", average_wpm)).into_centered_line();

            let v_data_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
                .split(centered_area);
            text_time.render(v_data_layout[0], buf);
            text_average_wpm.render(v_data_layout[1], buf);
        }
    }
    fn win_screen_clock(
        &self,
        clock_data: &ClockData,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        if clock_data.skip {
            let centered_area = centered_rect_with_length(area.width, 1, area);
            let text = "Game Skipped";
            let line = Span::raw(text).into_centered_line();
            line.render(centered_area, buf);
        } else {
            let centered_area = centered_rect_with_length(area.width, 2, area);

            let time = clock_data.time;
            let n_words = clock_data.n_words;
            let average_wpm = (n_words as f32) * 60.0 / time as f32;

            let text_time = Span::raw(format!("number of correctly typed words: {:}", n_words))
                .into_centered_line();
            let text_average_wpm =
                Span::raw(format!("average wpm: {:.0}", average_wpm)).into_centered_line();

            let v_data_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
                .split(centered_area);
            text_time.render(v_data_layout[0], buf);
            text_average_wpm.render(v_data_layout[1], buf);
        }
    }

    fn win_screen_infinite(
        &self,
        infinite_data: &InfiniteData,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        debug!(
            "Rendering infinite win screen with data: {:?}",
            infinite_data
        );
        debug!("Height of area: {}", area.height);
        let centered_area = centered_rect_with_length(area.width, 3, area);

        let time = infinite_data.time;
        let n_words_typed = infinite_data.n_words;
        let average_wpm = (n_words_typed as f32) * 60.0 / time.as_secs_f32();

        let text_time =
            Span::raw(format!("time: {:.2} seconds", time.as_secs_f32())).into_centered_line();
        let text_n_words = Span::raw(format!(
            "number of correctly typed words: {:}",
            n_words_typed
        ))
        .into_centered_line();
        let text_average_wpm =
            Span::raw(format!("average wpm: {:.0}", average_wpm)).into_centered_line();

        let v_data_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(centered_area);
        text_time.render(v_data_layout[0], buf);
        text_n_words.render(v_data_layout[1], buf);
        text_average_wpm.render(v_data_layout[2], buf);
    }

    fn new_game(&self) {
        match &self.shr_win_data.borrow().game {
            crate::win_data::GameModEndResult::None => self
                .send(Action::EscPressed)
                .expect("to be able to send action"),
            crate::win_data::GameModEndResult::Infinite(_) => todo!(),
            crate::win_data::GameModEndResult::Race(_) => self
                .send(Action::AskChangeToScreen(Screen::SoloRaceGame))
                .expect("to be able to send action"),
            crate::win_data::GameModEndResult::Clock(_) => self
                .send(Action::AskChangeToScreen(Screen::SoloClockGame))
                .expect("to be able to send action"),
        }
    }
    fn option_selected(&self) {
        match self.choice_state {
            SelectedOption::Leave => {
                self.shr_settings.borrow_mut().game_settings.keep_seed = false;
                self.send(Action::EscPressed)
                    .expect("to be able to send action")
            }
            SelectedOption::Same => {
                self.shr_settings.borrow_mut().game_settings.keep_seed = true;
                self.new_game();
            }
            SelectedOption::Next => {
                self.shr_settings.borrow_mut().game_settings.keep_seed = false;
                self.new_game();
            }
        }
    }
}

impl Store for WinMenuScreen {
    fn update(&mut self, action: Action) {
        match action {
            Action::RightPressed => self.choice_state = self.choice_state.next(),
            Action::LeftPressed => self.choice_state = self.choice_state.previous(),
            Action::EnterPressed => self.option_selected(),
            _ => {}
        }
    }
}

impl SendAction for WinMenuScreen {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &WinMenuScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .border_type(BorderType::Rounded)
            .title(" WELL DONE ")
            .borders(Borders::ALL);
        let v_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(3)])
            .split(area);
        let inner_area_block = block.inner(v_layout[0]);
        block.render(v_layout[0], buf);

        match &self.shr_win_data.borrow().game {
            crate::win_data::GameModEndResult::None => {
                unreachable!("WinData has neverbeen initialized")
            }
            crate::win_data::GameModEndResult::Infinite(infinite_data) => {
                self.win_screen_infinite(infinite_data, inner_area_block, buf)
            }
            crate::win_data::GameModEndResult::Race(race_data) => {
                self.win_screen_race(race_data, inner_area_block, buf)
            }
            crate::win_data::GameModEndResult::Clock(clock_data) => {
                self.win_screen_clock(clock_data, inner_area_block, buf)
            }
        }

        let block_choices = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let list_genertator = HorizontalList::new(vec![
            String::from(" Quit "),
            String::from(" Redo ⟳ "),
            String::from(" Next >> "),
        ])
        .block(block_choices)
        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        ratatui::widgets::StatefulWidget::render(
            &list_genertator,
            v_layout[1],
            buf,
            &mut ListState::default().with_selected(Some(self.choice_state.get_pos())),
        );
    }
}

impl IsScreen for WinMenuScreen {}
