use log::error;
use ratatui::widgets::{Block, List, ListState, StatefulWidget, Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    flux::SendAction,
    stores::Store,
    ui::screens::IsScreen,
};

use super::super::{super::*, Screen};

const SCREEN: Screen = Screen::SoloGamesMenu;

#[derive(Debug)]
pub struct SoloGamesMenuScreen {
    dispatcher_tx: UnboundedSender<Action>,
    selected_entry: usize,
}
impl SoloGamesMenuScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        // let entry_speed = MutipleEntry::new(
        //     ["Speed", "Settings"]
        //         .into_iter()
        //         .map(|el| el.to_string())
        //         .collect(),
        //     vec![
        //         Action::AskChangeToScreen(Screen::SoloRaceGame),
        //         Action::AskChangeToScreen(Screen::SoloRaceSetting),
        //     ],
        // );
        // let entry_clock = MutipleEntry::new(
        //     ["Clock", "Settings"]
        //         .into_iter()
        //         .map(|el| el.to_string())
        //         .collect(),
        //     vec![
        //         Action::AskChangeToScreen(Screen::SoloClockGame),
        //         Action::AskChangeToScreen(Screen::SoloClockSetting),
        //     ],
        // );
        // let entry_inifinite = MutipleEntry::new(
        //     ["Infinite", "Settings"]
        //         .into_iter()
        //         .map(|el| el.to_string())
        //         .collect(),
        //     vec![
        //         Action::AskChangeToScreen(Screen::SoloInfiniteGame),
        //         Action::AskChangeToScreen(Screen::SoloInfiniteSetting),
        //     ],
        // );

        // let text_settting_entry = MutipleEntry::new(
        //     ["Settings", "test"]
        //         .into_iter()
        //         .map(|el| el.to_string())
        //         .collect(),
        //     vec![
        //         Action::AskChangeToScreen(Screen::SoloTextSettings),
        //         Action::AskChangeToScreen(Screen::SoloTextSettings),
        //     ],
        // );
        // let list_store = MenuMultipleListComponent::new(
        //     "Select your game mod".to_string(),
        //     dispatcher_tx.clone(),
        //     vec![
        //         entry_speed,
        //         entry_clock,
        //         entry_inifinite,
        //         text_settting_entry,
        //     ],
        //     vec![Constraint::Percentage(100), Constraint::Length(10)],
        //     SCREEN,
        // );
        SoloGamesMenuScreen {
            dispatcher_tx,
            selected_entry: 0,
        }
    }
    fn select_next(&mut self) {
        self.selected_entry = std::cmp::min(self.selected_entry + 1, 3);
    }

    fn select_previous(&mut self) {
        self.selected_entry = self.selected_entry.saturating_sub(1);
    }
}

impl Store for SoloGamesMenuScreen {
    fn update(&mut self, action: Action) {
        match action {
            Action::DownPressed => {
                self.select_next();
            }
            Action::UpPressed => {
                self.select_previous();
            }
            Action::EnterPressed => {
                let result = match self.selected_entry {
                    0 => self.send(Action::AskChangeToScreen(Screen::SoloRaceGame)),
                    1 => self.send(Action::AskChangeToScreen(Screen::SoloClockGame)),
                    2 => self.send(Action::AskChangeToScreen(Screen::SoloInfiniteGame)),
                    3 => self.send(Action::AskChangeToScreen(Screen::SoloGameSettings)),
                    _ => unreachable!("Invalid selected entry: {}", self.selected_entry),
                };
                if let Err(e) = result {
                    error!("Failed to send action: {}", e);
                }
            }
            _ => {}
        }
    }
}

impl Widget for &SoloGamesMenuScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect_with_length(20, 9, area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Length(3)])
            .split(area);
        let block_games = apply_block_style(Block::default().title("Games"));

        let block_settings = apply_block_style(Block::default());

        let game_list =
            apply_list_style(List::new(vec!["Race", "Clock", "Infinite"]).block(block_games));
        // .highlight_style(get_style1());

        let state = if self.selected_entry < 3 {
            &mut ListState::default().with_selected(Some(self.selected_entry))
        } else {
            &mut ListState::default()
        };
        StatefulWidget::render(&game_list, layout[0], buf, state);

        let setting_list = apply_list_style(List::new(vec!["Settings"]).block(block_settings));
        //.highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        let state = if self.selected_entry == 3 {
            &mut ListState::default().with_selected(Some(self.selected_entry))
        } else {
            &mut ListState::default()
        };
        StatefulWidget::render(&setting_list, layout[1], buf, state);
    }
}
impl SendAction for SoloGamesMenuScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for SoloGamesMenuScreen {}
