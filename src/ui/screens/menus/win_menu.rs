use std::{cell::RefCell, fmt::format, rc::Rc};

use ratatui::{
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, ListState, StatefulWidget, Widget},
};
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::{
    action::Action,
    flux::SendAction,
    settings::Settings,
    stores::Store,
    ui::{list::HorizontalList, screens::IsScreen},
    win_data::{ClockData, RaceData, WinData},
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::WinMenu;

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
            let text = "Game Skipped";
            let line = Span::raw(text).into_centered_line();
            line.render(area, buf);
        } else {
            let time = race_data.time;
            let average_wpm = (race_data.n_words as f32) * 60.0 / race_data.time.as_secs_f32();

            let text_time =
                Span::raw(format!("time: {:.2} seconds", time.as_secs_f32())).into_centered_line();
            let text_average_wpm =
                Span::raw(format!("average wpm: {:.0}", average_wpm)).into_centered_line();

            let v_data_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
                .split(area);
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
            let text = "Game Skipped";
            let line = Span::raw(text).into_centered_line();
            line.render(area, buf);
        } else {
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
                .split(area);
            text_time.render(v_data_layout[0], buf);
            text_average_wpm.render(v_data_layout[1], buf);
        }
    }
    fn new_game(&self) {
        match &self.shr_win_data.borrow().game {
            crate::win_data::GameMod::None => self
                .send(Action::EscPressed)
                .expect("to be able to send action"),
            crate::win_data::GameMod::Infinite(_) => todo!(),
            crate::win_data::GameMod::Race(_) => self
                .send(Action::AskChangeToScreen(Screen::SoloRaceGame))
                .expect("to be able to send action"),
            crate::win_data::GameMod::Clock(_) => self
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
    fn default(&mut self) {
        self.choice_state = SelectedOption::default()
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
        let data_layout = centered_rect_with_length(inner_area_block.width, 2, inner_area_block);
        match &self.shr_win_data.borrow().game {
            crate::win_data::GameMod::None => unreachable!("WinData has neverbeen initialized"),
            crate::win_data::GameMod::Infinite(infinite_data) => todo!(),
            crate::win_data::GameMod::Race(race_data) => {
                self.win_screen_race(race_data, data_layout, buf)
            }
            crate::win_data::GameMod::Clock(clock_data) => {
                self.win_screen_clock(clock_data, data_layout, buf)
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
