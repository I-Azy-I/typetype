use std::{cell::RefCell, rc::Rc};

use async_deferred::Deferred;
use log::{debug, warn};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, List, ListState, StatefulWidget, Widget},
};
use tokio::{fs, sync::mpsc::UnboundedSender};

use crate::{
    action::Action,
    config::{DEFAULT_LANGUAGE, DEFAULT_TEXT, PATH_LANGUAGES, PATH_TEXTS},
    flux::SendAction,
    settings::{Settings, StartEndSentence, StartingPointSentence, TextOrigin},
    stores::Store,
    ui::{
        list::HorizontalList,
        screens::{IsScreen, games::GameMod},
    },
};

#[derive(Copy, Clone, Debug)]
enum SelectedPart {
    Generator,
    Source,
    None,
}

impl SelectedPart {
    fn next(self) -> Self {
        match self {
            SelectedPart::Generator => SelectedPart::Source,
            SelectedPart::Source => SelectedPart::Source,
            SelectedPart::None => SelectedPart::None,
        }
    }

    fn previous(self) -> Self {
        match self {
            SelectedPart::Generator => SelectedPart::Generator,
            SelectedPart::Source => SelectedPart::Generator,
            SelectedPart::None => SelectedPart::None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
enum GeneratingOption {
    #[default]
    Text,
    Language,
}

impl GeneratingOption {
    fn next(self) -> Self {
        match self {
            GeneratingOption::Text => GeneratingOption::Language,
            GeneratingOption::Language => GeneratingOption::Language,
        }
    }
    fn previous(self) -> Self {
        match self {
            GeneratingOption::Text => GeneratingOption::Text,
            GeneratingOption::Language => GeneratingOption::Text,
        }
    }

    fn to_list_state(self) -> ListState {
        let selected = match self {
            GeneratingOption::Text => 0,
            GeneratingOption::Language => 1,
        };
        ListState::default().with_selected(Some(selected))
    }
    fn get_list() -> [&'static str; 2] {
        ["Text", "Language"]
    }
}
async fn get_list_file_in_folder(path: &str, first_value: Option<String>) -> Vec<String> {
    let res = async {
        let mut entries = fs::read_dir(path).await.ok()?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await.ok()? {
            if entry.file_type().await.ok()?.is_file() {
                files.push(entry.file_name().display().to_string());
            }
        }

        if let Some(first_value) = first_value {
            let index = files.iter().position(|entry| *entry == first_value);
            if let Some(index) = index {
                files.remove(index);
            }
            files.sort();
            if index.is_some() {
                files.insert(0, first_value);
            }
        } else {
            files.sort();
        }

        Some(files)
    }
    .await;
    res.unwrap_or_default()
}

#[derive(Debug)]
pub struct SourceSettingsSoloGameComponent {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    state_generator: GeneratingOption,
    texts: Deferred<Vec<String>>,
    selected_text: ListState,
    languages: Deferred<Vec<String>>,
    selected_language: ListState,
    selected_part: SelectedPart,
    editing_part: SelectedPart,
    game_mod: GameMod,
}
impl SourceSettingsSoloGameComponent {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        settings: Rc<RefCell<Settings>>,
        game_mod: GameMod,
    ) -> Self {
        let texts = Deferred::start(async || {
            get_list_file_in_folder(PATH_TEXTS, Some(String::from(DEFAULT_TEXT))).await
        });
        let languages = Deferred::start(async || {
            get_list_file_in_folder(PATH_LANGUAGES, Some(String::from(DEFAULT_LANGUAGE))).await
        });

        SourceSettingsSoloGameComponent {
            dispatcher_tx,
            settings,
            state_generator: GeneratingOption::default(),
            selected_part: SelectedPart::None,
            editing_part: SelectedPart::None,
            texts,
            selected_text: ListState::default().with_selected(Some(0)),
            languages,
            selected_language: ListState::default().with_selected(Some(0)),
            game_mod,
        }
    }

    pub fn selected_part(&self) -> SelectedPart {
        self.selected_part
    }
    pub fn editing_part(&self) -> SelectedPart {
        self.editing_part
    }
    pub fn select_default(&mut self) {
        self.selected_part = SelectedPart::Generator
    }
    pub fn select_generator(&mut self) {
        self.selected_part = SelectedPart::Generator
    }
    pub fn select_source(&mut self) {
        self.selected_part = SelectedPart::Source
    }

    pub fn select_next(&mut self) {
        self.selected_part = self.selected_part.next();
    }
    pub fn select_previous(&mut self) {
        self.selected_part = self.selected_part.previous();
    }
    pub fn unselect(&mut self) {
        self.editing_part = SelectedPart::None;
        self.selected_part = SelectedPart::None;
    }

    pub fn next_generator(&mut self) {
        self.state_generator = self.state_generator.next();
    }

    pub fn previous_generator(&mut self) {
        self.state_generator = self.state_generator.previous();
    }
    // fn select_new_source(&mut self, filename: String) {
    //     match self.game_mod {
    //         GameMod::Race => {
    //             self.settings
    //                 .borrow_mut()
    //                 .game_settings
    //                 .race_game_settings
    //                 .filename = filename
    //         }
    //         GameMod::Clock => todo!(),
    //         GameMod::Infinite => todo!(),
    //     }
    // }
    fn get_current_filename(&self) -> Option<String> {
        match self.state_generator {
            GeneratingOption::Text => self
                .texts
                .try_get()
                .map(|texts| texts[self.selected_text.selected().unwrap()].clone()),
            GeneratingOption::Language => self
                .languages
                .try_get()
                .map(|languages| languages[self.selected_text.selected().unwrap()].clone()),
        }
    }
    pub fn save_in_settings(&self) {
        // let mut settings = self.settings.borrow_mut().game_settings.text_settings;

        // match self.game_mod {
        //     GameMod::Race => {
        //         let race_game_settings = &mut settings.game_settings.race_game_settings;
        //         race_game_settings.text_origin = self.state_generator.to_setting_param();
        //         if let Some(filename) = self.get_current_filename() {
        //             race_game_settings.filename = filename
        //         }
        //     }
        //     GameMod::Clock => todo!(),
        //     GameMod::Infinite => todo!(),
        // }
    }

    pub fn is_editing(&self) -> bool {
        !matches!(self.editing_part, SelectedPart::None)
    }
    pub fn is_selected(&self) -> bool {
        !matches!(self.selected_part, SelectedPart::None)
    }
}

impl Store for SourceSettingsSoloGameComponent {
    fn update(&mut self, action: Action) {
        match action {
            Action::RightPressed => match self.editing_part {
                SelectedPart::Generator => self.next_generator(),
                SelectedPart::Source => {}
                SelectedPart::None => {}
            },
            Action::LeftPressed => match self.editing_part {
                SelectedPart::Generator => self.previous_generator(),
                SelectedPart::Source => {}
                SelectedPart::None => {}
            },

            Action::UpPressed => match self.editing_part {
                SelectedPart::Generator => self.editing_part = SelectedPart::None,
                SelectedPart::Source => match self.state_generator {
                    GeneratingOption::Text => {
                        if self.texts.is_complete() {
                            self.selected_text.select_previous()
                        }
                    }
                    GeneratingOption::Language => {
                        if self.languages.is_complete() {
                            self.selected_language.select_previous();
                        }
                    }
                },
                SelectedPart::None => self.selected_part = self.selected_part.previous(),
            },
            Action::DownPressed => match self.editing_part {
                SelectedPart::Generator => {
                    self.editing_part = SelectedPart::None;
                    self.selected_part = self.selected_part.next()
                }
                SelectedPart::Source => match self.state_generator {
                    GeneratingOption::Text => {
                        if let Some(sources) = self.texts.try_get() {
                            if self.selected_text.selected().unwrap() < sources.len() - 1 {
                                self.selected_text.select_next();
                            }
                        }
                    }
                    GeneratingOption::Language => {
                        if let Some(sources) = self.languages.try_get() {
                            if self.selected_language.selected().unwrap() < sources.len() - 1 {
                                self.selected_language.select_next();
                            }
                        }
                    }
                },
                SelectedPart::None => self.selected_part = self.selected_part.next(),
            },
            Action::EnterPressed if !matches!(self.editing_part, SelectedPart::None) => {
                self.editing_part = SelectedPart::None;
            }
            Action::EnterPressed if matches!(self.editing_part, SelectedPart::None) => {
                self.editing_part = self.selected_part;
            }

            _ => {}
        }
    }
}

impl SendAction for SourceSettingsSoloGameComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl Widget for &SourceSettingsSoloGameComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        //self.a_component.render(area, buf);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(50)])
            .split(area);
        let block_generator = {
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .title("Text generation")
                .borders(Borders::ALL);
            if matches!(self.editing_part, SelectedPart::Generator) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, SelectedPart::Generator) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_genertator = HorizontalList::new(
            GeneratingOption::get_list()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_generator)
        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        ratatui::widgets::StatefulWidget::render(
            &list_genertator,
            layout[0],
            buf,
            &mut self.state_generator.to_list_state(),
        );

        let block_src = {
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);

            if matches!(self.editing_part, SelectedPart::Source) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, SelectedPart::Source) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        match self.state_generator {
            GeneratingOption::Text => {
                if let Some(sources) = self.texts.try_get() {
                    let list_src = List::new(sources.clone())
                        .block(block_src)
                        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
                    StatefulWidget::render(
                        list_src,
                        layout[1],
                        buf,
                        &mut self.selected_text.clone(),
                    );
                }
            }
            GeneratingOption::Language => {
                if let Some(sources) = self.languages.try_get() {
                    let list_src = List::new(sources.clone())
                        .block(block_src)
                        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
                    StatefulWidget::render(
                        list_src,
                        layout[1],
                        buf,
                        &mut self.selected_language.clone(),
                    );
                }
            }
        }
    }
}

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
enum StartEndSentenceState {
    #[default]
    None,
    Start,
    StartEnd,
}
impl StartEndSentenceState {
    fn next(self) -> Self {
        match self {
            StartEndSentenceState::None => StartEndSentenceState::Start,
            StartEndSentenceState::Start => StartEndSentenceState::StartEnd,
            StartEndSentenceState::StartEnd => StartEndSentenceState::StartEnd,
        }
    }
    fn previous(self) -> Self {
        match self {
            StartEndSentenceState::None => StartEndSentenceState::None,
            StartEndSentenceState::Start => StartEndSentenceState::None,
            StartEndSentenceState::StartEnd => StartEndSentenceState::Start,
        }
    }
    fn state(self) -> ListState {
        let value = match self {
            StartEndSentenceState::None => 0,
            StartEndSentenceState::Start => 1,
            StartEndSentenceState::StartEnd => 2,
        };
        ListState::default().with_selected(Some(value))
    }

    fn setting_value(self) -> StartEndSentence {
        match self {
            StartEndSentenceState::None => StartEndSentence::Any,
            StartEndSentenceState::Start => StartEndSentence::Start,
            StartEndSentenceState::StartEnd => StartEndSentence::StartEnd,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum StartingPointState {
    #[default]
    Beginning,
    Random,
}
impl StartingPointState {
    fn next(self) -> Self {
        match self {
            Self::Beginning => Self::Random,
            Self::Random => Self::Random,
        }
    }
    fn previous(self) -> Self {
        match self {
            Self::Beginning => Self::Beginning,
            Self::Random => Self::Beginning,
        }
    }
    fn state(self) -> ListState {
        let value = match self {
            StartingPointState::Beginning => 0,
            StartingPointState::Random => 1,
        };
        ListState::default().with_selected(Some(value))
    }

    fn setting_value(self) -> StartingPointSentence {
        match self {
            Self::Beginning => StartingPointSentence::Beginning,
            Self::Random => StartingPointSentence::Random,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum RacePart {
    NumberWord,
    #[default]
    None,
    StartEnd,
    StartingPoint,
}

#[derive(Debug)]
pub struct SoloRaceSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    source_settings: SourceSettingsSoloGameComponent,
    chosen_length: NumberWord,
    start_end_option: StartEndSentenceState,
    starting_point_option: StartingPointState,
    selected_part: RacePart,
    editing_part: RacePart,
}

impl SoloRaceSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut basic_settings = SourceSettingsSoloGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            GameMod::Race,
        );
        basic_settings.select_default();
        SoloRaceSettingScreen {
            dispatcher_tx,
            settings,
            source_settings: basic_settings,
            start_end_option: StartEndSentenceState::default(),
            starting_point_option: StartingPointState::default(),
            chosen_length: NumberWord::default(),
            selected_part: RacePart::default(),
            editing_part: RacePart::default(),
        }
    }

    fn next_choosed_number_words(&mut self) {
        self.chosen_length = self.chosen_length.next();
    }
    fn previous_choosed_number_words(&mut self) {
        self.chosen_length = self.chosen_length.previous();
    }

    fn next_start_end(&mut self) {
        self.start_end_option = self.start_end_option.next();
    }
    fn previous_start_end(&mut self) {
        self.start_end_option = self.start_end_option.previous();
    }

    fn next_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.next();
    }
    fn previous_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.previous();
    }

    fn save_in_settings(&self) {
        self.source_settings.save_in_settings();
        let settings_race = &mut self.settings.borrow_mut().game_settings.race_game_settings;
        settings_race.number_words = self.chosen_length.value();
    }
}

impl Store for SoloRaceSettingScreen {
    fn update(&mut self, action: Action) {
        self.source_settings.update(action);
        if !self.source_settings.is_selected() {
            match action {
                Action::DownPressed => match self.selected_part {
                    RacePart::NumberWord => {
                        self.editing_part = RacePart::None;
                        self.selected_part = RacePart::StartEnd
                    }
                    RacePart::StartEnd => {
                        if matches!(self.editing_part, RacePart::StartEnd) {
                            self.next_start_end()
                        } else {
                            self.editing_part = RacePart::None;
                            self.selected_part = RacePart::StartingPoint
                        }
                    }
                    RacePart::StartingPoint
                        if matches!(self.editing_part, RacePart::StartingPoint) =>
                    {
                        self.next_starting_point()
                    }
                    RacePart::StartingPoint | RacePart::None => {}
                },
                Action::UpPressed => match self.selected_part {
                    RacePart::NumberWord => {}
                    RacePart::None => {}
                    RacePart::StartEnd => {
                        if matches!(self.editing_part, RacePart::StartEnd) {
                            self.previous_start_end()
                        } else {
                            self.selected_part = RacePart::NumberWord
                        }
                    }
                    RacePart::StartingPoint => {
                        if matches!(self.editing_part, RacePart::StartingPoint) {
                            self.previous_starting_point()
                        } else {
                            self.selected_part = RacePart::StartEnd
                        }
                    }
                },
                Action::RightPressed => match self.editing_part {
                    RacePart::NumberWord => self.next_choosed_number_words(),
                    RacePart::None | RacePart::StartEnd | RacePart::StartingPoint => {}
                },
                Action::LeftPressed => match self.editing_part {
                    RacePart::NumberWord => self.previous_choosed_number_words(),
                    RacePart::None | RacePart::StartEnd | RacePart::StartingPoint => {
                        match self.selected_part {
                            RacePart::NumberWord => self.source_settings.select_generator(),
                            RacePart::None => unreachable!(),
                            RacePart::StartEnd => self.source_settings.select_source(),
                            RacePart::StartingPoint => self.source_settings.select_source(),
                        }
                        self.selected_part = RacePart::None;
                        self.editing_part = RacePart::None;
                    }
                },
                Action::EnterPressed => match self.editing_part {
                    RacePart::NumberWord => self.editing_part = RacePart::None,
                    RacePart::None => self.editing_part = self.selected_part,
                    RacePart::StartEnd => self.editing_part = RacePart::None,
                    RacePart::StartingPoint => self.editing_part = RacePart::None,
                },
                _ => {}
            }
        } else if let Action::RightPressed = action {
            match self.source_settings.selected_part() {
                SelectedPart::Generator
                    if !matches!(self.source_settings.editing_part(), SelectedPart::Generator) =>
                {
                    self.source_settings.unselect();
                    self.selected_part = RacePart::NumberWord
                }
                SelectedPart::Source => {
                    self.source_settings.unselect();
                    self.selected_part = RacePart::StartEnd;
                }
                SelectedPart::Generator | SelectedPart::None => {}
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
        self.source_settings.render(basic_config_area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(4),
            ])
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

        // fit sentences
        let block_start = {
            let block = Block::default()
                .title("Fit sentences")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, RacePart::StartEnd) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, RacePart::StartEnd) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Any", "Start", "Start and End"])
            .block(block_start)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(list_src, layout[1], buf, &mut self.start_end_option.state());

        //starting point
        let block_starting_point = {
            let block = Block::default()
                .title("Starting point")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, RacePart::StartingPoint) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, RacePart::StartingPoint) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Beginning", "Random"])
            .block(block_starting_point)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(
            list_src,
            layout[2],
            buf,
            &mut self.starting_point_option.state(),
        );
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

#[derive(Debug, Default, Clone, Copy)]
enum TimeCock {
    S10,
    S20,
    #[default]
    S30,
    S60,
    Custom(usize), // TODO
}

impl TimeCock {
    fn next(self) -> Self {
        use TimeCock::*;
        match self {
            S10 => S20,
            S20 => S30,
            S30 => S60,
            S60 => S60,
            Custom(_) => todo!(),
        }
    }
    fn previous(self) -> Self {
        use TimeCock::*;
        match self {
            S10 => S10,
            S20 => S10,
            S30 => S20,
            S60 => S30,
            Custom(_) => todo!(),
        }
    }
    fn state(self) -> ListState {
        use TimeCock::*;
        let value = match self {
            S10 => 0,
            S20 => 1,
            S30 => 2,
            S60 => 3,
            Custom(_) => todo!(),
        };
        ListState::default().with_selected(Some(value))
    }
    fn value(self) -> usize {
        use TimeCock::*;
        match self {
            S10 => 10,
            S20 => 20,
            S30 => 30,
            S60 => 60,
            Custom(_) => todo!(),
        }
    }
    fn get_list_option() -> [&'static str; 4] {
        ["10s", "20s", "30s", "60s"]
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum ClockPart {
    NumberWord,
    #[default]
    None,
    StartEnd,
    StartingPoint,
}

#[derive(Debug)]
pub struct SoloClockSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    source_settings: SourceSettingsSoloGameComponent,
    chosen_time: TimeCock,
    start_end_option: StartEndSentenceState,
    starting_point_option: StartingPointState,
    selected_part: ClockPart,
    editing_part: ClockPart,
}

impl SoloClockSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut basic_settings = SourceSettingsSoloGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            GameMod::Race,
        );
        basic_settings.select_default();
        SoloClockSettingScreen {
            dispatcher_tx,
            settings,
            source_settings: basic_settings,
            start_end_option: StartEndSentenceState::default(),
            starting_point_option: StartingPointState::default(),
            chosen_time: TimeCock::default(),
            selected_part: ClockPart::default(),
            editing_part: ClockPart::default(),
        }
    }

    fn next_choosed_time(&mut self) {
        self.chosen_time = self.chosen_time.next();
    }
    fn previous_choosed_time(&mut self) {
        self.chosen_time = self.chosen_time.previous();
    }

    fn next_start_end(&mut self) {
        self.start_end_option = self.start_end_option.next();
    }
    fn previous_start_end(&mut self) {
        self.start_end_option = self.start_end_option.previous();
    }
    fn next_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.next();
    }
    fn previous_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.previous();
    }
    fn save_in_settings(&self) {
        self.source_settings.save_in_settings();
        let settings_clock = &mut self.settings.borrow_mut().game_settings.clock_game_settings;
        settings_clock.time = self.chosen_time.value();
    }
}

impl Store for SoloClockSettingScreen {
    fn update(&mut self, action: Action) {
        self.source_settings.update(action);
        if !self.source_settings.is_selected() {
            match action {
                Action::DownPressed => match self.selected_part {
                    ClockPart::NumberWord => {
                        self.editing_part = ClockPart::None;
                        self.selected_part = ClockPart::StartEnd
                    }
                    ClockPart::StartEnd => {
                        if matches!(self.editing_part, ClockPart::StartEnd) {
                            self.next_start_end()
                        } else {
                            self.editing_part = ClockPart::None;
                            self.selected_part = ClockPart::StartingPoint
                        }
                    }
                    ClockPart::StartingPoint
                        if matches!(self.editing_part, ClockPart::StartingPoint) =>
                    {
                        self.next_starting_point()
                    }
                    ClockPart::StartingPoint | ClockPart::None => {}
                },
                Action::UpPressed => match self.selected_part {
                    ClockPart::NumberWord => {}
                    ClockPart::None => {}
                    ClockPart::StartEnd => {
                        if matches!(self.editing_part, ClockPart::StartEnd) {
                            self.previous_start_end()
                        } else {
                            self.selected_part = ClockPart::NumberWord
                        }
                    }
                    ClockPart::StartingPoint => {
                        if matches!(self.editing_part, ClockPart::StartingPoint) {
                            self.previous_starting_point()
                        } else {
                            self.selected_part = ClockPart::StartEnd
                        }
                    }
                },
                Action::RightPressed => match self.editing_part {
                    ClockPart::NumberWord => self.next_choosed_time(),
                    ClockPart::None | ClockPart::StartEnd | ClockPart::StartingPoint => {}
                },
                Action::LeftPressed => match self.editing_part {
                    ClockPart::NumberWord => self.previous_choosed_time(),
                    ClockPart::None | ClockPart::StartEnd | ClockPart::StartingPoint => {
                        match self.selected_part {
                            ClockPart::NumberWord => self.source_settings.select_generator(),
                            ClockPart::None => unreachable!(),
                            ClockPart::StartEnd => self.source_settings.select_source(),
                            ClockPart::StartingPoint => self.source_settings.select_source(),
                        }
                        self.selected_part = ClockPart::None;
                        self.editing_part = ClockPart::None;
                    }
                },
                Action::EnterPressed => match self.editing_part {
                    ClockPart::NumberWord => self.editing_part = ClockPart::None,
                    ClockPart::None => self.editing_part = self.selected_part,
                    ClockPart::StartEnd => self.editing_part = ClockPart::None,
                    ClockPart::StartingPoint => self.editing_part = ClockPart::None,
                },
                _ => {}
            }
        } else if let Action::RightPressed = action {
            match self.source_settings.selected_part() {
                SelectedPart::Generator
                    if !matches!(self.source_settings.editing_part(), SelectedPart::Generator) =>
                {
                    self.source_settings.unselect();
                    self.selected_part = ClockPart::NumberWord
                }
                SelectedPart::Source => {
                    self.source_settings.unselect();
                    self.selected_part = ClockPart::StartEnd;
                }
                SelectedPart::Generator | SelectedPart::None => {}
            }
        }
    }
}

impl Widget for &SoloClockSettingScreen {
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
        self.source_settings.render(basic_config_area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(4),
            ])
            .split(layout[1]);

        let block_n_words_selection = {
            let block = Block::default()
                .title("Time")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, ClockPart::NumberWord) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, ClockPart::NumberWord) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };
        let list = HorizontalList::new(
            TimeCock::get_list_option()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_n_words_selection)
        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        list.render(layout[0], buf, &mut self.chosen_time.state());

        let block_start = {
            let block = Block::default()
                .title("Fit sentences")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, ClockPart::StartEnd) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, ClockPart::StartEnd) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Any", "Start", "Start and End"])
            .block(block_start)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(list_src, layout[1], buf, &mut self.start_end_option.state());

        //starting point
        let block_starting_point = {
            let block = Block::default()
                .title("Starting point")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, ClockPart::StartingPoint) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, ClockPart::StartingPoint) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Beginning", "Random"])
            .block(block_starting_point)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(
            list_src,
            layout[2],
            buf,
            &mut self.starting_point_option.state(),
        );
    }
}
impl SendAction for SoloClockSettingScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for SoloClockSettingScreen {
    fn close(&mut self) {
        debug!("closed");
        self.save_in_settings();
        debug!("settings: {:#?}", self.settings.borrow())
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum InfinitePart {
    #[default]
    None,
    StartEnd,
}

#[derive(Debug)]
pub struct SoloInfiniteSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    source_settings: SourceSettingsSoloGameComponent,
    start_end_option: StartEndSentenceState,
    selected_part: InfinitePart,
    editing_part: InfinitePart,
}

impl SoloInfiniteSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut basic_settings = SourceSettingsSoloGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            GameMod::Race,
        );
        basic_settings.select_default();
        SoloInfiniteSettingScreen {
            dispatcher_tx,
            settings,
            source_settings: basic_settings,
            start_end_option: StartEndSentenceState::default(),
            selected_part: InfinitePart::default(),
            editing_part: InfinitePart::default(),
        }
    }

    fn next_start_end(&mut self) {
        self.start_end_option = self.start_end_option.next();
    }
    fn previous_start_end(&mut self) {
        self.start_end_option = self.start_end_option.previous();
    }

    fn save_in_settings(&self) {
        self.source_settings.save_in_settings();
        let settings_infinite = &mut self
            .settings
            .borrow_mut()
            .game_settings
            .infinite_game_settings;
    }
}

impl Store for SoloInfiniteSettingScreen {
    fn update(&mut self, action: Action) {
        self.source_settings.update(action);
        if !self.source_settings.is_selected() {
            match action {
                Action::DownPressed => match self.selected_part {
                    InfinitePart::StartEnd
                        if matches!(self.editing_part, InfinitePart::StartEnd) =>
                    {
                        self.next_start_end()
                    }
                    InfinitePart::StartEnd | InfinitePart::None => {}
                },
                Action::UpPressed => match self.editing_part {
                    InfinitePart::None => {}
                    InfinitePart::StartEnd => self.previous_start_end(),
                },
                Action::RightPressed => match self.editing_part {
                    InfinitePart::None | InfinitePart::StartEnd => {}
                },
                Action::LeftPressed => match self.editing_part {
                    InfinitePart::None | InfinitePart::StartEnd => {
                        match self.selected_part {
                            InfinitePart::None => unreachable!(),
                            InfinitePart::StartEnd => self.source_settings.select_generator(),
                        }
                        self.selected_part = InfinitePart::None;
                        self.editing_part = InfinitePart::None;
                    }
                },
                Action::EnterPressed => match self.editing_part {
                    InfinitePart::None => self.editing_part = self.selected_part,
                    InfinitePart::StartEnd => self.editing_part = InfinitePart::None,
                },
                _ => {}
            }
        } else if let Action::RightPressed = action {
            match self.source_settings.selected_part() {
                SelectedPart::Generator
                    if !matches!(self.source_settings.editing_part(), SelectedPart::Generator) =>
                {
                    self.source_settings.unselect();
                    self.selected_part = InfinitePart::StartEnd
                }
                SelectedPart::Source => {
                    self.source_settings.unselect();
                    self.selected_part = InfinitePart::StartEnd;
                }
                SelectedPart::Generator | SelectedPart::None => {}
            }
        }
    }
}

impl Widget for &SoloInfiniteSettingScreen {
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
        self.source_settings.render(basic_config_area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(5)])
            .split(layout[1]);

        let block_start = {
            let block = Block::default()
                .title("Fit sentences")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, InfinitePart::StartEnd) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, InfinitePart::StartEnd) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Any", "Start", "Start and End"])
            .block(block_start)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(list_src, layout[0], buf, &mut self.start_end_option.state());
    }
}
impl SendAction for SoloInfiniteSettingScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for SoloInfiniteSettingScreen {
    fn close(&mut self) {
        debug!("closed");
        self.save_in_settings();
        debug!("settings: {:#?}", self.settings.borrow())
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum TextSettingPart {
    #[default]
    Generator,
    Source,
    StartEnd,
    StartingPoint,
}

#[derive(Debug)]
pub struct TextSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    chosen_length: NumberWord,
    start_end_option: StartEndSentenceState,
    starting_point_option: StartingPointState,
    selected_part: TextSettingPart,
    editing_part: Option<TextSettingPart>,

    state_generator: GeneratingOption,
    texts: Deferred<Vec<String>>,
    selected_text: ListState,
    languages: Deferred<Vec<String>>,
    selected_language: ListState,
}

impl TextSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut basic_settings = SourceSettingsSoloGameComponent::new(
            dispatcher_tx.clone(),
            settings.clone(),
            GameMod::Race,
        );
        let texts = Deferred::start(async || {
            get_list_file_in_folder(PATH_TEXTS, Some(String::from(DEFAULT_TEXT))).await
        });
        let languages = Deferred::start(async || {
            get_list_file_in_folder(PATH_LANGUAGES, Some(String::from(DEFAULT_LANGUAGE))).await
        });
        basic_settings.select_default();
        TextSettingScreen {
            dispatcher_tx,
            settings,
            start_end_option: StartEndSentenceState::default(),
            starting_point_option: StartingPointState::default(),
            chosen_length: NumberWord::default(),
            selected_part: TextSettingPart::default(),
            editing_part: None,

            state_generator: GeneratingOption::default(),
            texts,
            selected_text: ListState::default().with_selected(Some(0)),
            languages,
            selected_language: ListState::default().with_selected(Some(0)),
        }
    }

    fn next_start_end(&mut self) {
        self.start_end_option = self.start_end_option.next();
    }
    fn previous_start_end(&mut self) {
        self.start_end_option = self.start_end_option.previous();
    }

    fn next_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.next();
    }
    fn previous_starting_point(&mut self) {
        self.starting_point_option = self.starting_point_option.previous();
    }
    fn get_current_filename(&self) -> Option<String> {
        match self.state_generator {
            GeneratingOption::Text => self
                .texts
                .try_get()
                .map(|texts| texts[self.selected_text.selected().unwrap()].clone()),
            GeneratingOption::Language => self
                .languages
                .try_get()
                .map(|languages| languages[self.selected_text.selected().unwrap()].clone()),
        }
    }
    fn save_in_settings(&self) {
        let text_origin = match self.state_generator {
            GeneratingOption::Text => TextOrigin::Text {
                start_end_sentence: self.start_end_option.setting_value(),
                starting_point: self.starting_point_option.setting_value(),
            },
            GeneratingOption::Language => TextOrigin::Language,
        };

        let filename = self.get_current_filename();

        let settings = &mut self.settings.borrow_mut().game_settings.text_settings;
        settings.filename = filename;
        settings.text_origin = text_origin;
    }
}

impl Store for TextSettingScreen {
    fn update(&mut self, action: Action) {
        match action {
            Action::DownPressed => match (self.selected_part, self.editing_part) {
                (TextSettingPart::Generator, None) => self.selected_part = TextSettingPart::Source,
                (TextSettingPart::Generator, Some(TextSettingPart::Generator)) => {
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::Source;
                }
                (TextSettingPart::Source, None) => {}
                (TextSettingPart::Source, Some(TextSettingPart::Source)) => {
                    match self.state_generator {
                        GeneratingOption::Text => {
                            if let Some(sources) = self.texts.try_get() {
                                if self.selected_text.selected().unwrap() < sources.len() - 1 {
                                    self.selected_text.select_next();
                                }
                            }
                        }
                        GeneratingOption::Language => {
                            if let Some(sources) = self.languages.try_get() {
                                if self.selected_language.selected().unwrap() < sources.len() - 1 {
                                    self.selected_language.select_next();
                                }
                            }
                        }
                    }
                }

                (TextSettingPart::StartEnd, None) => {
                    self.selected_part = TextSettingPart::StartingPoint
                }
                (TextSettingPart::StartEnd, Some(TextSettingPart::StartEnd)) => {
                    self.next_start_end();
                }
                (TextSettingPart::StartingPoint, None) => {}
                (TextSettingPart::StartingPoint, Some(TextSettingPart::StartingPoint)) => {
                    self.next_starting_point()
                }
                _ => warn!(
                    "When moving up in text settings an illegal move happened: selected part: {:?}, edditing part: {:?}",
                    self.selected_part, self.editing_part
                ),
            },
            Action::UpPressed => match (self.selected_part, self.editing_part) {
                (TextSettingPart::Generator, None) => {}
                (TextSettingPart::Generator, Some(TextSettingPart::Generator)) => {
                    self.editing_part = None
                }
                (TextSettingPart::Source, None) => self.selected_part = TextSettingPart::Generator,
                (TextSettingPart::Source, Some(TextSettingPart::Source)) => {
                    match self.state_generator {
                        GeneratingOption::Text => {
                            if self.texts.is_complete() {
                                self.selected_text.select_previous()
                            }
                        }
                        GeneratingOption::Language => {
                            if self.languages.is_complete() {
                                self.selected_language.select_previous();
                            }
                        }
                    }
                }
                (TextSettingPart::StartEnd, None) => {}
                (TextSettingPart::StartEnd, Some(TextSettingPart::StartEnd)) => {
                    self.previous_start_end();
                }
                (TextSettingPart::StartingPoint, None) => {
                    self.selected_part = TextSettingPart::StartEnd
                }
                (TextSettingPart::StartingPoint, Some(TextSettingPart::StartingPoint)) => {
                    self.previous_starting_point();
                }
                _ => warn!(
                    "When moving down in text settings an illegal move happened: selected part: {:?}, edditing part: {:?}",
                    self.selected_part, self.editing_part
                ),
            },
            Action::RightPressed => match (self.selected_part, self.editing_part) {
                (TextSettingPart::Generator, None) => {
                    self.selected_part = TextSettingPart::StartEnd
                }
                (TextSettingPart::Generator, Some(TextSettingPart::Generator)) => {
                    self.state_generator = self.state_generator.next()
                }
                (TextSettingPart::Source, None) => {
                    self.selected_part = TextSettingPart::StartingPoint
                }
                (TextSettingPart::Source, Some(TextSettingPart::Source)) => {
                    match self.state_generator {
                        GeneratingOption::Text => {
                            if self.texts.is_complete() {
                                self.selected_text.select_previous()
                            }
                        }
                        GeneratingOption::Language => {
                            if self.languages.is_complete() {
                                self.selected_language.select_previous();
                            }
                        }
                    }
                }
                (TextSettingPart::StartEnd, None) => {}
                (TextSettingPart::StartEnd, Some(TextSettingPart::StartEnd)) => {
                    self.editing_part = None
                }
                (TextSettingPart::StartingPoint, None) => {}
                (TextSettingPart::StartingPoint, Some(TextSettingPart::StartingPoint)) => {
                    self.editing_part = None;
                }
                _ => warn!(
                    "When moving right in text settings an illegal move happened: selected part: {:?}, edditing part: {:?}",
                    self.selected_part, self.editing_part
                ),
            },
            Action::LeftPressed => match (self.selected_part, self.editing_part) {
                (TextSettingPart::Generator, None) => {}
                (TextSettingPart::Generator, Some(TextSettingPart::Generator)) => {
                    self.state_generator = self.state_generator.previous()
                }
                (TextSettingPart::Source, None) => {}
                (TextSettingPart::Source, Some(TextSettingPart::Source)) => {
                    self.editing_part = None;
                }
                (TextSettingPart::StartEnd, None) => {
                    self.selected_part = TextSettingPart::Generator
                }
                (TextSettingPart::StartEnd, Some(TextSettingPart::StartEnd)) => {
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::Generator
                }
                (TextSettingPart::StartingPoint, None) => {
                    self.selected_part = TextSettingPart::Source
                }
                (TextSettingPart::StartingPoint, Some(TextSettingPart::StartingPoint)) => {
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::Generator
                }
                _ => warn!(
                    "When moving left in text settings an illegal move happened: selected part: {:?}, edditing part: {:?}",
                    self.selected_part, self.editing_part
                ),
            },
            Action::EnterPressed => match self.editing_part {
                None => self.editing_part = Some(self.selected_part),
                Some(_) => self.editing_part = None,
            },
            _ => {}
        }
    }
}

impl Widget for &TextSettingScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(30)])
            .split(area);
        // render generator and source settings
        let basic_config_area = layout[0];

        let layout_gen_src = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(50)])
            .split(basic_config_area);

        let block_generator = {
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .title("Text generation")
                .borders(Borders::ALL);
            if matches!(self.editing_part, Some(TextSettingPart::Generator)) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, TextSettingPart::Generator) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_generator = HorizontalList::new(
            GeneratingOption::get_list()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_generator)
        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        ratatui::widgets::StatefulWidget::render(
            &list_generator,
            layout_gen_src[0],
            buf,
            &mut self.state_generator.to_list_state(),
        );

        let block_src = {
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);

            if matches!(self.editing_part, Some(TextSettingPart::Source)) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, TextSettingPart::Source) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        match self.state_generator {
            GeneratingOption::Text => {
                if let Some(sources) = self.texts.try_get() {
                    let list_src = List::new(sources.clone())
                        .block(block_src)
                        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
                    StatefulWidget::render(
                        list_src,
                        layout_gen_src[1],
                        buf,
                        &mut self.selected_text.clone(),
                    );
                }
            }
            GeneratingOption::Language => {
                if let Some(sources) = self.languages.try_get() {
                    let list_src = List::new(sources.clone())
                        .block(block_src)
                        .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
                    StatefulWidget::render(
                        list_src,
                        layout_gen_src[1],
                        buf,
                        &mut self.selected_language.clone(),
                    );
                }
            }
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(5), Constraint::Length(4)])
            .split(layout[1]);

        // fit sentences
        let block_start = {
            let block = Block::default()
                .title("Fit sentences")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, Some(TextSettingPart::StartEnd)) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, TextSettingPart::StartEnd) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Any", "Start", "Start and End"])
            .block(block_start)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(list_src, layout[0], buf, &mut self.start_end_option.state());

        //starting point
        let block_starting_point = {
            let block = Block::default()
                .title("Starting point")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);
            if matches!(self.editing_part, Some(TextSettingPart::StartingPoint)) {
                block.border_style(Style::new().blue())
            } else if matches!(self.selected_part, TextSettingPart::StartingPoint) {
                block.border_style(Style::new().yellow())
            } else {
                block
            }
        };

        let list_src = List::new(vec!["Beginning", "Random"])
            .block(block_starting_point)
            .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
        StatefulWidget::render(
            list_src,
            layout[1],
            buf,
            &mut self.starting_point_option.state(),
        );
    }
}
impl SendAction for TextSettingScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl IsScreen for TextSettingScreen {
    fn close(&mut self) {
        self.save_in_settings();
    }
}
