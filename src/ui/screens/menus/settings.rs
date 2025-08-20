use std::{cell::RefCell, rc::Rc};

use async_deferred::Deferred;
use log::{debug, warn};
use ratatui::{
    layout::{self, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, List, ListState, StatefulWidget, Widget, block::title},
};
use tokio::{fs, sync::mpsc::UnboundedSender};

use crate::{
    action::Action,
    config::{DEFAULT_LANGUAGE, DEFAULT_TEXT, PATH_LANGUAGES, PATH_TEXTS},
    flux::SendAction,
    settings::{Settings, StartEndSentence, StartingPointSentence, TextOrigin},
    stores::Store,
    ui::{
        apply_block_style, apply_list_style,
        list::HorizontalList,
        list_hightlight_style, over_block_style,
        screens::{IsScreen, games::GameMod},
        select_block_style,
    },
};

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
enum TimeClock {
    S10,
    S20,
    #[default]
    S30,
    S60,
    Custom(usize), // TODO
}

impl TimeClock {
    fn next(self) -> Self {
        use TimeClock::*;
        match self {
            S10 => S20,
            S20 => S30,
            S30 => S60,
            S60 => S60,
            Custom(_) => todo!(),
        }
    }
    fn previous(self) -> Self {
        use TimeClock::*;
        match self {
            S10 => S10,
            S20 => S10,
            S30 => S20,
            S60 => S30,
            Custom(_) => todo!(),
        }
    }
    fn state(self) -> ListState {
        use TimeClock::*;
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
        use TimeClock::*;
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum TextSettingPart {
    #[default]
    Generator,
    Source,
    StartEnd,
    StartingPoint,
    TimeClock,
    NumberWordsRace,
}

#[derive(Debug)]
pub struct TextSettingScreen {
    dispatcher_tx: UnboundedSender<Action>,
    settings: Rc<RefCell<Settings>>,
    start_end_option: StartEndSentenceState,
    starting_point_option: StartingPointState,
    selected_part: TextSettingPart,
    editing_part: Option<TextSettingPart>,

    state_generator: GeneratingOption,
    texts: Deferred<Vec<String>>,
    selected_text: ListState,
    languages: Deferred<Vec<String>>,
    selected_language: ListState,

    state_number_words_race: NumberWord,
    state_time_clock: TimeClock,
}

impl TextSettingScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, settings: Rc<RefCell<Settings>>) -> Self {
        let texts = Deferred::start(async || {
            get_list_file_in_folder(PATH_TEXTS, Some(String::from(DEFAULT_TEXT))).await
        });
        let languages = Deferred::start(async || {
            get_list_file_in_folder(PATH_LANGUAGES, Some(String::from(DEFAULT_LANGUAGE))).await
        });

        TextSettingScreen {
            dispatcher_tx,
            settings,
            start_end_option: StartEndSentenceState::default(),
            starting_point_option: StartingPointState::default(),
            selected_part: TextSettingPart::default(),
            editing_part: None,

            state_generator: GeneratingOption::default(),
            texts,
            selected_text: ListState::default().with_selected(Some(0)),
            languages,
            selected_language: ListState::default().with_selected(Some(0)),

            state_time_clock: TimeClock::default(),
            state_number_words_race: NumberWord::default(),
        }
    }
    fn next_chosen_time(&mut self) {
        self.state_time_clock = self.state_time_clock.next();
    }
    fn previous_chosen_time(&mut self) {
        self.state_time_clock = self.state_time_clock.previous();
    }
    fn next_chosen_number_words(&mut self) {
        self.state_number_words_race = self.state_number_words_race.next();
    }
    fn previous_chosen_number_words(&mut self) {
        self.state_number_words_race = self.state_number_words_race.previous();
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

        let time_clock = self.state_time_clock.value();
        let number_words = self.state_number_words_race.value();

        let settings = &mut self.settings.borrow_mut().game_settings;
        settings.text_settings.filename = filename;
        settings.text_settings.text_origin = text_origin;
        settings.race_game_settings.number_words = number_words;
        settings.clock_game_settings.time = time_clock;
    }

    fn create_block(&self, title: &'static str, part: TextSettingPart) -> Block {
        let block = apply_block_style(Block::default().title(title));

        if self.editing_part == Some(part) {
            select_block_style(block)
        } else if self.selected_part == part {
            over_block_style(block)
        } else {
            block
        }
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
                (TextSettingPart::NumberWordsRace, None) => {
                    self.selected_part = TextSettingPart::TimeClock
                }
                (TextSettingPart::NumberWordsRace, Some(TextSettingPart::NumberWordsRace)) => {
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::TimeClock;
                }
                (TextSettingPart::TimeClock, None) => {}
                (TextSettingPart::TimeClock, Some(TextSettingPart::TimeClock)) => {
                    self.editing_part = None;
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
                (TextSettingPart::NumberWordsRace, None) => {}
                (TextSettingPart::NumberWordsRace, Some(TextSettingPart::NumberWordsRace)) => {
                    self.editing_part = None;
                }
                (TextSettingPart::TimeClock, None) => {
                    self.selected_part = TextSettingPart::NumberWordsRace
                }
                (TextSettingPart::TimeClock, Some(TextSettingPart::TimeClock)) => {
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::NumberWordsRace;
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
                    self.editing_part = None;
                    self.selected_part = TextSettingPart::StartingPoint;
                }
                (TextSettingPart::StartEnd, None) => {
                    self.selected_part = TextSettingPart::NumberWordsRace
                }
                (TextSettingPart::StartEnd, Some(TextSettingPart::StartEnd)) => {
                    self.selected_part = TextSettingPart::NumberWordsRace;
                    self.editing_part = None
                }
                (TextSettingPart::StartingPoint, None) => {
                    self.selected_part = TextSettingPart::TimeClock;
                }
                (TextSettingPart::StartingPoint, Some(TextSettingPart::StartingPoint)) => {
                    self.selected_part = TextSettingPart::TimeClock;
                    self.editing_part = None;
                }
                (TextSettingPart::NumberWordsRace, None) => {}
                (TextSettingPart::NumberWordsRace, Some(TextSettingPart::NumberWordsRace)) => {
                    self.next_chosen_number_words();
                }
                (TextSettingPart::TimeClock, None) => {}
                (TextSettingPart::TimeClock, Some(TextSettingPart::TimeClock)) => {
                    self.next_chosen_time();
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
                (TextSettingPart::NumberWordsRace, None) => {
                    self.selected_part = TextSettingPart::StartEnd
                }
                (TextSettingPart::NumberWordsRace, Some(TextSettingPart::NumberWordsRace)) => {
                    self.previous_chosen_number_words();
                }
                (TextSettingPart::TimeClock, None) => {
                    self.selected_part = TextSettingPart::StartEnd
                }
                (TextSettingPart::TimeClock, Some(TextSettingPart::TimeClock)) => {
                    self.previous_chosen_time();
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
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);
        // render generator and source settings
        let basic_config_area = layout[0];

        let layout_gen_src = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(50)])
            .split(basic_config_area);

        let block_generator = self.create_block("Text generation", TextSettingPart::Generator);

        let list_generator = HorizontalList::new(
            GeneratingOption::get_list()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_generator)
        .highlight_style(list_hightlight_style());
        ratatui::widgets::StatefulWidget::render(
            &list_generator,
            layout_gen_src[0],
            buf,
            &mut self.state_generator.to_list_state(),
        );

        let block_src = self.create_block("", TextSettingPart::Source);

        match self.state_generator {
            GeneratingOption::Text => {
                if let Some(sources) = self.texts.try_get() {
                    let list_src = apply_list_style(List::new(sources.clone()).block(block_src));
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
                    let list_src = apply_list_style(List::new(sources.clone()).block(block_src));
                    StatefulWidget::render(
                        list_src,
                        layout_gen_src[1],
                        buf,
                        &mut self.selected_language.clone(),
                    );
                }
            }
        }

        let layout_second_column = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(5), Constraint::Length(4)])
            .split(layout[1]);

        // fit sentences
        let block_start = self.create_block("Fit sentences", TextSettingPart::StartEnd);

        let list_src =
            apply_list_style(List::new(vec!["Any", "Start", "Start and End"]).block(block_start));
        StatefulWidget::render(
            list_src,
            layout_second_column[0],
            buf,
            &mut self.start_end_option.state(),
        );

        //starting point
        let block_starting_point =
            self.create_block("Starting point", TextSettingPart::StartingPoint);

        let list_src =
            apply_list_style(List::new(vec!["Beginning", "Random"]).block(block_starting_point));
        StatefulWidget::render(
            list_src,
            layout_second_column[1],
            buf,
            &mut self.starting_point_option.state(),
        );

        let layout_third_column = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
            .split(layout[2]);

        let block_n_words_race =
            self.create_block("Number words Race game", TextSettingPart::NumberWordsRace);

        let list_number_words_race = HorizontalList::new(
            NumberWord::get_list_option()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_n_words_race)
        .highlight_style(list_hightlight_style());

        let block_time_clock = self.create_block("Time Clock game", TextSettingPart::TimeClock);

        let list_time_clock = HorizontalList::new(
            TimeClock::get_list_option()
                .into_iter()
                .map(|el| el.to_string())
                .collect(),
        )
        .block(block_time_clock)
        .highlight_style(list_hightlight_style());

        list_number_words_race.render(
            layout_third_column[0],
            buf,
            &mut self.state_number_words_race.state(),
        );
        list_time_clock.render(
            layout_third_column[1],
            buf,
            &mut self.state_time_clock.state(),
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
