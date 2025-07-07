use std::{cell::RefCell, rc::Rc};

use log::debug;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, ListState, StatefulWidget, Widget},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    flux::SendAction,
    settings::Settings,
    stores::Store,
    ui::{
        list::HorizontalList,
        screens::{IsScreen, Screen, games::GameMod},
        setting_screen::SettingsSoloGameComponent,
    },
};

#[derive(Debug, Default, Clone, Copy)]
enum NumberWord {
    W10,
    W25,
    #[default]
    W50,
    W100,
    Custom(usize), // TODO
}

impl NumberWord {
    fn next(self) -> Self {
        use NumberWord::*;
        match self {
            W10 => W25,
            W25 => W50,
            W50 => W100,
            W100 => W100,
            Custom(_) => todo!(),
        }
    }
    fn previous(self) -> Self {
        use NumberWord::*;
        match self {
            W10 => W10,
            W25 => W10,
            W50 => W25,
            W100 => W50,
            Custom(_) => todo!(),
        }
    }
    fn state(self) -> ListState {
        use NumberWord::*;
        let value = match self {
            W10 => 0,
            W25 => 1,
            W50 => 2,
            W100 => 3,
            Custom(_) => todo!(),
        };
        ListState::default().with_selected(Some(value))
    }
    fn value(self) -> usize {
        use NumberWord::*;
        match self {
            W10 => 10,
            W25 => 25,
            W50 => 50,
            W100 => 100,
            Custom(_) => todo!(),
        }
    }
    fn get_list_option() -> [&'static str; 4] {
        ["10", "25", "50", "100"]
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum RacePart {
    NumberWord,
    #[default]
    None,
}
#[derive(Debug)]
pub struct SoloRaceSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    basic_settings: SettingsSoloGameComponent,
    chosen_length: NumberWord,
    selected_part: RacePart,
    editing_part: RacePart,
}

impl SoloRaceSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut basic_settings = SettingsSoloGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            GameMod::Race,
        );
        basic_settings.select_default();
        SoloRaceSettingScreen {
            dispatcher_tx,
            settings,
            basic_settings,
            chosen_length: NumberWord::default(),
            selected_part: RacePart::default(),
            editing_part: RacePart::default(),
        }
    }

    fn next_choosed_number_words(&mut self) {
        self.chosen_length = self.chosen_length.next();
        let mut settings = self.settings.borrow_mut();
        settings.game_settings.race_game_settings.number_words = self.chosen_length.value() as usize
    }
    fn previous_choosed_number_words(&mut self) {
        self.chosen_length = self.chosen_length.previous();
        let mut settings = self.settings.borrow_mut();
        settings.game_settings.race_game_settings.number_words = self.chosen_length.value() as usize
    }

     fn save_in_settings(&self) {
        self.basic_settings.save_in_settings();
        let settings_race = &mut self.settings.borrow_mut().game_settings.race_game_settings;
        settings_race.number_words = self.chosen_length.value();
    }
}

impl Store for SoloRaceSettingScreen {
    fn update(&mut self, action: Action) {
        self.basic_settings.update(action);
        if !self.basic_settings.selected() {
            if matches!(self.selected_part, RacePart::None) {
                self.selected_part = RacePart::NumberWord
            };
            match action {
                Action::DownPressed => {}
                Action::UpPressed => {}
                Action::RightPressed => match self.editing_part {
                    RacePart::NumberWord => self.next_choosed_number_words(),
                    RacePart::None => {}
                },
                Action::LeftPressed => match self.editing_part {
                    RacePart::NumberWord => self.previous_choosed_number_words(),
                    RacePart::None => {
                        self.selected_part = RacePart::None;
                        self.basic_settings.select_default();
                    }
                },
                Action::EnterPressed => match self.editing_part {
                    RacePart::NumberWord => self.editing_part = RacePart::None,
                    RacePart::None => self.editing_part = self.selected_part,
                },
                _ => {}
            }
        }
    }
}

impl Widget for &SoloRaceSettingScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);
        let basic_config_area = layout[0];
        self.basic_settings.render(basic_config_area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(100)])
            .split(layout[1]);

        let block_n_words_selection = {
            let block = Block::default()
                .title("Number of words")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, RacePart::NumberWord) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, RacePart::NumberWord) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };
        let list = HorizontalList::new(
            NumberWord::get_list_option()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_n_words_selection)
        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        list.render(layout[0], buf, &mut self.chosen_length.state());
    }
}
impl SendAction for SoloRaceSettingScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for SoloRaceSettingScreen {
    fn close(&mut self) {
        debug!("closed");
        self.save_in_settings();
        debug!("settings: {:#?}", self.settings.borrow())
    }
}
