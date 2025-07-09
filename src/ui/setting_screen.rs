// use std::{
//     cell::RefCell,
//     path::{self, PathBuf},
//     rc::Rc,
// };

// use async_deferred::Deferred;
// use ratatui::{
//     layout::{Constraint, Direction, Layout, Rect},
//     style::{Style, Stylize},
//     widgets::{Block, BorderType, Borders, List, ListState, StatefulWidget, Widget},
// };
// use tokio::{fs, sync::mpsc::UnboundedSender};

// use super::{
//     list::HorizontalList,
//     screens::{Screen, games::GameMod},
// };
// use crate::{
//     action::Action,
//     flux::SendAction,
//     settings::{OffsetText, Settings, TextOrigin},
//     stores::Store,
// };
// use crate::{
//     config::{DEFAULT_LANGUAGE, DEFAULT_TEXT, PATH_LANGUAGES, PATH_TEXTS},
//     ui::list::ScrollableList,
// };

// #[derive(Copy, Clone, Debug)]
// enum SelectedPart {
//     Generator,
//     Source,
//     None,
// }

// impl SelectedPart {
//     fn next(self) -> Self {
//         match self {
//             SelectedPart::Generator => SelectedPart::Source,
//             SelectedPart::Source => SelectedPart::Source,
//             SelectedPart::None => SelectedPart::None,
//         }
//     }

//     fn previous(self) -> Self {
//         match self {
//             SelectedPart::Generator => SelectedPart::Generator,
//             SelectedPart::Source => SelectedPart::Generator,
//             SelectedPart::None => SelectedPart::None,
//         }
//     }
// }

// #[derive(Copy, Clone, Debug, Default)]
// enum GeneratingOption {
//     #[default]
//     Text,
//     Language,
// }
// impl GeneratingOption {
//     fn to_setting_param(self) -> TextOrigin {
//         match self {
//             GeneratingOption::Text => TextOrigin::Text(OffsetText::Random),
//             GeneratingOption::Language => TextOrigin::Generated,
//         }
//     }
// }

// impl GeneratingOption {
//     fn next(self) -> Self {
//         match self {
//             GeneratingOption::Text => GeneratingOption::Language,
//             GeneratingOption::Language => GeneratingOption::Language,
//         }
//     }
//     fn previous(self) -> Self {
//         match self {
//             GeneratingOption::Text => GeneratingOption::Text,
//             GeneratingOption::Language => GeneratingOption::Text,
//         }
//     }

//     fn to_list_state(self) -> ListState {
//         let selected = match self {
//             GeneratingOption::Text => 0,
//             GeneratingOption::Language => 1,
//         };
//         ListState::default().with_selected(Some(selected))
//     }
//     fn get_list() -> [&'static str; 2] {
//         ["Text", "Language"]
//     }
// }
// async fn get_list_file_in_folder(path: &str, first_value: Option<String>) -> Vec<String> {
//     let res = async {
//         let mut entries = fs::read_dir(path).await.ok()?;
//         let mut files = Vec::new();

//         while let Some(entry) = entries.next_entry().await.ok()? {
//             if entry.file_type().await.ok()?.is_file() {
//                 files.push(entry.file_name().display().to_string());
//             }
//         }

//         if let Some(first_value) = first_value {
//             let index = files.iter().position(|entry| *entry == first_value);
//             if let Some(index) = index {
//                 files.remove(index);
//             }
//             files.sort();
//             if index.is_some() {
//                 files.insert(0, first_value);
//             }
//         } else {
//             files.sort();
//         }

//         Some(files)
//     }
//     .await;
//     res.unwrap_or_default()
// }

// #[derive(Debug)]
// pub struct SettingsSoloGameComponent {
//     dispatcher_tx: UnboundedSender<Action>,
//     settings: Rc<RefCell<Settings>>,
//     state_generator: GeneratingOption,
//     texts: Deferred<Vec<String>>,
//     selected_text: ListState,
//     languages: Deferred<Vec<String>>,
//     selected_language: ListState,
//     selected_part: SelectedPart,
//     editing_part: SelectedPart,
//     game_mod: GameMod,
// }
// impl SettingsSoloGameComponent {
//     pub fn new(
//         dispatcher_tx: UnboundedSender<Action>,
//         settings: Rc<RefCell<Settings>>,
//         game_mod: GameMod,
//     ) -> Self {
//         let texts = Deferred::start(async || {
//             get_list_file_in_folder(PATH_TEXTS, Some(String::from(DEFAULT_TEXT))).await
//         });
//         let languages = Deferred::start(async || {
//             get_list_file_in_folder(PATH_LANGUAGES, Some(String::from(DEFAULT_LANGUAGE))).await
//         });

//         SettingsSoloGameComponent {
//             dispatcher_tx,
//             settings,
//             state_generator: GeneratingOption::default(),
//             selected_part: SelectedPart::None,
//             editing_part: SelectedPart::None,
//             texts,
//             selected_text: ListState::default().with_selected(Some(0)),
//             languages,
//             selected_language: ListState::default().with_selected(Some(0)),
//             game_mod,
//         }
//     }

//     pub fn select_default(&mut self) {
//         self.selected_part = SelectedPart::Generator
//     }
//     pub fn select_generator(&mut self) {
//         self.selected_part = SelectedPart::Generator
//     }
//     pub fn select_source(&mut self) {
//         self.selected_part = SelectedPart::Source
//     }

//     pub fn select_next(&mut self) {
//         self.selected_part = self.selected_part.next();
//     }
//     pub fn select_previous(&mut self) {
//         self.selected_part = self.selected_part.previous();
//     }
//     pub fn unselect(&mut self) {
//         self.selected_part = SelectedPart::None;
//     }

//     pub fn next_generator(&mut self) {
//         self.state_generator = self.state_generator.next();
//     }

//     pub fn previous_generator(&mut self) {
//         self.state_generator = self.state_generator.previous();
//     }
//     // fn select_new_source(&mut self, filename: String) {
//     //     match self.game_mod {
//     //         GameMod::Race => {
//     //             self.settings
//     //                 .borrow_mut()
//     //                 .game_settings
//     //                 .race_game_settings
//     //                 .filename = filename
//     //         }
//     //         GameMod::Clock => todo!(),
//     //         GameMod::Infinite => todo!(),
//     //     }
//     // }
//     fn get_current_filename(&self) -> Option<String> {
//         match self.state_generator {
//             GeneratingOption::Text => self
//                 .texts
//                 .try_get()
//                 .map(|texts| texts[self.selected_text.selected().unwrap()].clone()),
//             GeneratingOption::Language => self
//                 .languages
//                 .try_get()
//                 .map(|languages| languages[self.selected_text.selected().unwrap()].clone()),
//         }
//     }
//     pub fn save_in_settings(&self) {
//         let mut settings = self.settings.borrow_mut();
//         match self.game_mod {
//             GameMod::Race => {
//                 let race_game_settings = &mut settings.game_settings.race_game_settings;
//                 race_game_settings.text_origin = self.state_generator.to_setting_param();
//                 if let Some(filename) = self.get_current_filename() {
//                     race_game_settings.filename = filename
//                 }
//             }
//             GameMod::Clock => todo!(),
//             GameMod::Infinite => todo!(),
//         }
//     }

//     pub fn is_editing(&self) -> bool {
//         !matches!(self.editing_part, SelectedPart::None)
//     }
//     pub fn selected(&self) -> bool {
//         !matches!(self.selected_part, SelectedPart::None)
//     }
// }

// impl Store for SettingsSoloGameComponent {
//     fn update(&mut self, action: Action) {
//         match action {
//             Action::RightPressed => match self.editing_part {
//                 SelectedPart::Generator => self.next_generator(),
//                 SelectedPart::Source => self.editing_part = SelectedPart::None,
//                 SelectedPart::None => self.unselect(),
//             },
//             Action::LeftPressed => match self.editing_part {
//                 SelectedPart::Generator => self.previous_generator(),
//                 SelectedPart::Source => self.editing_part = SelectedPart::None,
//                 SelectedPart::None => {}
//             },

//             Action::UpPressed => match self.editing_part {
//                 SelectedPart::Generator => self.editing_part = SelectedPart::None,
//                 SelectedPart::Source => match self.state_generator {
//                     GeneratingOption::Text => {
//                         if self.texts.is_complete() {
//                             self.selected_text.select_previous()
//                         }
//                     }
//                     GeneratingOption::Language => {
//                         if self.languages.is_complete() {
//                             self.selected_language.select_previous();
//                         }
//                     }
//                 },
//                 SelectedPart::None => self.selected_part = self.selected_part.previous(),
//             },
//             Action::DownPressed => match self.editing_part {
//                 SelectedPart::Generator => {
//                     self.editing_part = SelectedPart::None;
//                     self.selected_part = self.selected_part.next()
//                 }
//                 SelectedPart::Source => match self.state_generator {
//                     GeneratingOption::Text => {
//                         if let Some(sources) = self.texts.try_get() {
//                             if self.selected_text.selected().unwrap() < sources.len() - 1 {
//                                 self.selected_text.select_next();
//                             }
//                         }
//                     }
//                     GeneratingOption::Language => {
//                         if let Some(sources) = self.languages.try_get() {
//                             if self.selected_language.selected().unwrap() < sources.len() - 1 {
//                                 self.selected_language.select_next();
//                             }
//                         }
//                     }
//                 },
//                 SelectedPart::None => self.selected_part = self.selected_part.next(),
//             },
//             Action::EnterPressed if !matches!(self.editing_part, SelectedPart::None) => {
//                 self.editing_part = SelectedPart::None;
//             }
//             Action::EnterPressed if matches!(self.editing_part, SelectedPart::None) => {
//                 self.editing_part = self.selected_part;
//             }

//             _ => {}
//         }
//     }
// }

// impl SendAction for SettingsSoloGameComponent {
//     fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
//         self.dispatcher_tx.send(action)
//     }
// }

// impl Widget for &SettingsSoloGameComponent {
//     fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
//     where
//         Self: Sized,
//     {
//         //self.a_component.render(area, buf);
//         let layout = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints(vec![Constraint::Length(3), Constraint::Percentage(50)])
//             .split(area);
//         let block_generator = {
//             let block = Block::default()
//                 .border_type(BorderType::Rounded)
//                 .title("Text generation")
//                 .borders(Borders::ALL);
//             if matches!(self.editing_part, SelectedPart::Generator) {
//                 block.border_style(Style::new().blue())
//             } else if matches!(self.selected_part, SelectedPart::Generator) {
//                 block.border_style(Style::new().yellow())
//             } else {
//                 block
//             }
//         };

//         let list_genertator = HorizontalList::new(
//             GeneratingOption::get_list()
//                 .into_iter()
//                 .map(|el| el.to_string())
//                 .collect(),
//         )
//         .block(block_generator)
//         .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
//         ratatui::widgets::StatefulWidget::render(
//             &list_genertator,
//             layout[0],
//             buf,
//             &mut self.state_generator.to_list_state(),
//         );

//         let block_src = {
//             let block = Block::default()
//                 .border_type(BorderType::Rounded)
//                 .borders(Borders::ALL);

//             if matches!(self.editing_part, SelectedPart::Source) {
//                 block.border_style(Style::new().blue())
//             } else if matches!(self.selected_part, SelectedPart::Source) {
//                 block.border_style(Style::new().yellow())
//             } else {
//                 block
//             }
//         };

//         match self.state_generator {
//             GeneratingOption::Text => {
//                 if let Some(sources) = self.texts.try_get() {
//                     let list_src = List::new(sources.clone())
//                         .block(block_src)
//                         .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
//                     StatefulWidget::render(
//                         list_src,
//                         layout[1],
//                         buf,
//                         &mut self.selected_text.clone(),
//                     );
//                 }
//             }
//             GeneratingOption::Language => {
//                 if let Some(sources) = self.languages.try_get() {
//                     let list_src = List::new(sources.clone())
//                         .block(block_src)
//                         .highlight_style(Style::new().bg(ratatui::style::Color::Yellow));
//                     StatefulWidget::render(
//                         list_src,
//                         layout[1],
//                         buf,
//                         &mut self.selected_language.clone(),
//                     );
//                 }
//             }
//         }
//     }
// }
