use std::{
    iter::{self},
    path::Path,
    time::Instant,
};

use async_deferred::Deferred;
use log::debug;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    config::{path_languages, path_texts},
    flux::SendAction,
    settings::{StartEndSentence, TextOrigin},
    stores::Store,
    text_generator::{TextGenerator, get_text},
};

use super::centered_rect_with_length;
const N_WORD_FOR_WPS: usize = 5;

#[derive(Copy, Clone, Debug, PartialEq)]
enum CharacterState {
    NotTyped,
    Typed,
    Incorrect,
    Selected,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct TypeChar {
    char: char,
    state: CharacterState,
}
impl TypeChar {
    fn incorrect(&mut self) -> Self {
        self.state = CharacterState::Incorrect;
        *self
    }
    fn typed(&mut self) -> Self {
        self.state = CharacterState::Typed;
        *self
    }
    fn not_typed(&mut self) -> Self {
        self.state = CharacterState::NotTyped;
        *self
    }
    fn selected(&mut self) -> Self {
        self.state = CharacterState::Selected;
        *self
    }
    fn guard() -> TypeChar {
        TypeChar::space()
    }
    fn space() -> TypeChar {
        TypeChar {
            char: ' ',
            state: CharacterState::NotTyped,
        }
    }

    fn is_space(&self) -> bool {
        self.char == ' '
    }
    #[allow(dead_code)]
    fn is_typed(&self) -> bool {
        matches!(self.state, CharacterState::Typed)
    }
    #[allow(dead_code)]
    fn is_incorrect(&self) -> bool {
        matches!(self.state, CharacterState::Incorrect)
    }
    #[allow(dead_code)]
    fn is_not_typed(&self) -> bool {
        matches!(self.state, CharacterState::NotTyped)
            || matches!(self.state, CharacterState::Selected)
    }
    #[allow(dead_code)]
    fn is_cursor(&self) -> bool {
        matches!(self.state, CharacterState::Selected)
    }
}

impl<'a> From<TypeChar> for Span<'a> {
    fn from(tc: TypeChar) -> Span<'a> {
        match tc.state {
            CharacterState::NotTyped => Span::raw(tc.char.to_string()).dark_gray(),
            CharacterState::Typed => Span::raw(tc.char.to_string()).white(),
            CharacterState::Incorrect => Span::raw(tc.char.to_string()).bg(Color::Red),
            CharacterState::Selected => Span::raw(tc.char.to_string()).yellow().underlined(),
        }
    }
}

#[derive(Debug, Clone)]
struct TypeLine {
    line: Vec<TypeChar>,
}
impl TypeLine {
    fn new(line: Vec<TypeChar>) -> TypeLine {
        TypeLine { line }
    }
}

impl Widget for TypeLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(
            self.line
                .into_iter()
                .map(|el| el.into())
                .collect::<Vec<Span>>(),
        );
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

#[derive(Debug)]
enum KindLength {
    Unlimited,
    Finished(usize),
}

#[derive(Debug)]
enum TextWidgetState {
    Loading,
    ErrorLoading,
    InProgress,
    DoneWithMistakes,
    Done,
}

#[derive(Debug)]
enum AsyncTextSource {
    StaticText(Deferred<Option<String>>),
    Generator(Deferred<Option<TextGenerator>>),
}
impl AsyncTextSource {
    pub fn from_language(
        path_source: impl AsRef<Path>,
        dispatcher_tx: UnboundedSender<Action>,
        seed: Option<u64>,
    ) -> Self {
        let path = path_source
            .as_ref()
            .to_str()
            .expect("to be a path with valid characters")
            .to_string()
            .clone();
        Self::Generator(Deferred::start_with_callback(
            move || TextGenerator::from_language(path, seed),
            move || {
                dispatcher_tx
                    .send(Action::AsyncCachedRecievedData(None))
                    .unwrap()
            },
        ))
    }

    pub fn from_text(
        path_source: impl AsRef<Path>,
        dispatcher_tx: UnboundedSender<Action>,
        number_words: Option<usize>,
        offset: Option<f64>,
        start: StartEndSentence,
    ) -> Self {
        fn extend_text_if_needed(
            text: String,
            n_words: usize,
            total_n_words: usize,
        ) -> (String, usize) {
            if total_n_words < n_words {
                let iteration = n_words.div_ceil(total_n_words);
                let repeated = std::iter::repeat_n(text, iteration);
                let text = repeated.collect::<Vec<String>>().join(" ");
                (text, total_n_words * iteration + iteration - 1)
            } else {
                (text, total_n_words)
            }
        }

        let path_source = path_source
            .as_ref()
            .to_str()
            .expect("to be a path with valid characters")
            .to_string()
            .clone();
        Self::StaticText(Deferred::start_with_callback(
            async move || {
                let text = get_text(path_source).await;
                let total_n_words = text
                    .as_ref()
                    .map(|text| text.split_whitespace().count())
                    .unwrap_or(0);

                let real_offset = (total_n_words as f64 * offset.unwrap_or(0.0)) as usize;
                text.map(|text| match (number_words, start) {
                    (None, StartEndSentence::Any) => {
                        let words: Vec<&str> = text.split_whitespace().collect();
                        words
                            .iter()
                            .skip(real_offset)
                            .chain(words.iter().take(real_offset))
                            .copied()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }
                    (None, _) => {
                        let words: Vec<&str> = text.split_whitespace().collect();
                        let mut skip_counter = 0;
                        let iter = words.iter().skip(real_offset);
                        if real_offset > 0 {
                            for c in iter {
                                if !c.ends_with('.') {
                                    skip_counter += 1;
                                } else {
                                    break;
                                }
                            }
                            skip_counter += 1;
                        }

                        words
                            .iter()
                            .skip(real_offset + skip_counter)
                            .chain(words.iter().take(real_offset + skip_counter))
                            .copied()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }
                    (Some(n_words), StartEndSentence::Any) => {
                        let (text, total_n_words) =
                            extend_text_if_needed(text, n_words, total_n_words);
                        let max_offset = total_n_words.saturating_sub(n_words);
                        let real_offset = std::cmp::min(max_offset, real_offset);
                        let words: Vec<&str> = text.split_whitespace().collect();
                        words
                            .iter()
                            .skip(real_offset)
                            .chain(
                                words
                                    .iter()
                                    .take(n_words.saturating_sub(words.len() - real_offset)),
                            )
                            .take(n_words)
                            .copied()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }
                    (Some(n_words), StartEndSentence::Start) => {
                        let (text, total_n_words) =
                            extend_text_if_needed(text, n_words, total_n_words);
                        let max_offset = total_n_words.saturating_sub(n_words);
                        let real_offset = std::cmp::min(max_offset, real_offset);
                        let words: Vec<&str> = text.split_whitespace().collect();
                        let mut skip_counter = 0;
                        let iter = words.iter().skip(real_offset);
                        if real_offset > 0 {
                            for c in iter {
                                if !c.ends_with('.') {
                                    skip_counter += 1;
                                } else {
                                    break;
                                }
                            }
                            skip_counter += 1;
                        }

                        words
                            .iter()
                            .skip(real_offset + skip_counter)
                            .chain(words.iter().take(
                                n_words.saturating_sub(words.len() - (real_offset + skip_counter)),
                            ))
                            .take(n_words)
                            .copied()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }
                    (Some(n_words), StartEndSentence::StartEnd) => {
                        let (text, total_n_words) =
                            extend_text_if_needed(text, n_words, total_n_words);
                        let max_offset = total_n_words.saturating_sub(n_words);
                        let real_offset = std::cmp::min(max_offset, real_offset);
                        let mut counter_words = n_words + 1;
                        text.split_whitespace()
                            .skip(real_offset)
                            .skip_while(|c| !c.ends_with('.'))
                            .skip(1)
                            .take_while(|word| {
                                if counter_words > 1 {
                                    counter_words -= 1;
                                    true
                                } else if counter_words == 1 && !word.ends_with('.') {
                                    true
                                } else if counter_words == 1 && word.ends_with('.') {
                                    counter_words -= 1;
                                    true
                                } else {
                                    false
                                }
                            })
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }
                })
            },
            move || {
                dispatcher_tx
                    .send(Action::AsyncCachedRecievedData(None))
                    .unwrap()
            },
        ))
    }

    pub fn is_available(&self) -> bool {
        match self {
            AsyncTextSource::StaticText(async_cache) => async_cache.is_ready(),
            AsyncTextSource::Generator(async_cache) => async_cache.is_ready(),
        }
    }

    pub fn as_fail(&self) -> bool {
        assert!(self.is_available());
        match self {
            AsyncTextSource::StaticText(async_cache) => async_cache.try_get().unwrap().is_none(),
            AsyncTextSource::Generator(async_cache) => async_cache.try_get().unwrap().is_none(),
        }
    }
}

#[derive(Debug)]
struct WpsTracker {
    time_previous_word: Instant,
    pos: usize,
    ring_buffer: [f32; N_WORD_FOR_WPS],
}
impl Default for WpsTracker {
    fn default() -> Self {
        Self {
            time_previous_word: Instant::now(),
            pos: Default::default(),
            ring_buffer: [f32::INFINITY; 5],
        }
    }
}
impl WpsTracker {
    fn record_word(&mut self) {
        let now = Instant::now();
        let time_elapsed = now - self.time_previous_word;
        self.add(time_elapsed.as_secs_f32());
        self.time_previous_word = now;
    }
    fn add(&mut self, element: f32) {
        self.pos = (self.pos + 1) % N_WORD_FOR_WPS;
        self.ring_buffer[self.pos] = element;
    }
    fn mean(&self) -> f32 {
        let sum: f32 = self.ring_buffer.iter().sum();
        sum / N_WORD_FOR_WPS as f32
    }
    fn wps(&self) -> f32 {
        60.0 / self.mean()
    }
}

#[derive(Debug)]
pub struct TextWidget {
    async_text_source: AsyncTextSource,
    kind_length: KindLength,

    current_width: u16,
    lines: Option<Vec<TypeLine>>,

    pos_in_line: usize,
    n_line: usize,
    total_position: usize,

    total_size: usize,
    mistakes_counter: usize,
    state: TextWidgetState,

    wps_tracker: WpsTracker,
}

impl TextWidget {
    fn new(
        origin: TextOrigin,
        filename: String,
        dispatcher_tx: UnboundedSender<Action>,
        number_words: Option<usize>,
        offset: Option<f64>,
        seed: Option<u64>,
    ) -> Self {
        match origin {
            TextOrigin::Language => {
                let dir_path = path_languages();
                let path_source = dir_path.join(filename);
                Self::from_language(path_source, dispatcher_tx, number_words, seed)
            }
            TextOrigin::Text {
                start_end_sentence,
                starting_point: _,
            } => {
                let dir_path = path_texts();
                let path_source = dir_path.join(filename);
                Self::from_text(
                    path_source,
                    dispatcher_tx,
                    number_words,
                    offset,
                    start_end_sentence,
                )
            }
        }
    }

    fn from_language(
        path_source: impl AsRef<Path>,
        dispatcher_tx: UnboundedSender<Action>,
        number_words: Option<usize>,
        seed: Option<u64>,
    ) -> Self {
        // let first_batch = 500;
        let async_text_source = AsyncTextSource::from_language(path_source, dispatcher_tx, seed);
        // let new_chars =
        //             .into_iter()
        //             .take(first_batch)
        //             .flat_map(|text| {
        //                 std::iter::once(TypeChar::space())
        //                     .chain(text.chars().map(|c| TypeChar {
        //                         char: c,
        //                         state: CharacterState::NotTyped,
        //                     }))
        //                     .collect::<Vec<_>>()
        //                     .into_iter()
        //             }).skip(1);
        // let text_generator_into_iter = TextGenerator::from_language(name, None).await.into_iter().skip_n(first_batch);
        // let lines =  Self::get_lines_from_iterator(
        //             new_chars,
        //             100,
        //         );

        TextWidget {
            async_text_source,
            current_width: 0,
            lines: None,
            pos_in_line: 0,
            n_line: 0,
            total_position: 0,
            total_size: 0,
            state: TextWidgetState::Loading,
            mistakes_counter: 0,
            kind_length: match number_words {
                Some(n) => KindLength::Finished(n),
                None => KindLength::Unlimited,
            },
            wps_tracker: WpsTracker::default(),
        }
    }

    fn from_text(
        path_source: impl AsRef<Path>,
        dispatcher_tx: UnboundedSender<Action>,
        number_words: Option<usize>,
        offset: Option<f64>,
        text_start_end: StartEndSentence,
    ) -> Self {
        let async_text_source = AsyncTextSource::from_text(
            path_source,
            dispatcher_tx,
            number_words,
            offset,
            text_start_end,
        );
        // let typechar_text: Vec<TypeChar> = get_text(name).await.unwrap()
        //     .chars()
        //     .map(|c| TypeChar { char: c, state: CharacterState::NotTyped })
        //     .collect();
        // let total_size = typechar_text.len();
        // let lines = Self::get_lines(typechar_text, 100);

        TextWidget {
            async_text_source,
            current_width: 0,
            lines: None,
            pos_in_line: 0,
            n_line: 0,
            total_position: 0,
            total_size: 0,
            state: TextWidgetState::Loading,
            mistakes_counter: 0,
            kind_length: match number_words {
                Some(n) => KindLength::Finished(n),
                None => KindLength::Unlimited,
            },
            wps_tracker: WpsTracker::default(),
        }
    }
    fn init(&mut self) {
        if self.async_text_source.as_fail() {
            self.state = TextWidgetState::ErrorLoading;
        } else {
            self.state = TextWidgetState::InProgress;
            let number_words = match self.kind_length {
                KindLength::Unlimited => 1000, // arbitrary large number
                KindLength::Finished(n) => n,
            };
            match &self.async_text_source {
                AsyncTextSource::StaticText(_) => self.init_text(number_words as u16),
                AsyncTextSource::Generator(_) => self.genrate_new_batch(number_words as u16, 20),
            }
        }
    }

    fn get_new_typechar_text(text: &String, number_words: u16) -> Vec<TypeChar> {
        let n_words = text.split_whitespace().count();
        let text = if n_words < number_words as usize {
            let iteration = (number_words as usize).div_ceil(n_words);
            let repeated = std::iter::repeat_n(text.clone(), iteration);
            repeated.collect::<Vec<String>>().join(" ")
        } else {
            text.to_string()
        };
        let typechar_text: Vec<TypeChar> = text
            .chars()
            .map(|c| TypeChar {
                char: c,
                state: CharacterState::NotTyped,
            })
            .collect();
        typechar_text
    }
    fn init_text(&mut self, number_words: u16) {
        match &self.async_text_source {
            AsyncTextSource::StaticText(async_cache) => {
                let text = async_cache.try_get().as_ref().unwrap().as_ref().unwrap();
                let typechar_text = Self::get_new_typechar_text(text, number_words);
                self.total_size = typechar_text.len();
                self.lines = Some(Self::get_lines(typechar_text, 100));
            }
            AsyncTextSource::Generator(_async_cache) => panic!("Should never happen"),
        }
    }
    fn genrate_new_batch(&mut self, batch_size: u16, width_max: u16) {
        match &self.async_text_source {
            AsyncTextSource::Generator(async_cache) => {
                assert!(async_cache.is_ready());

                let lines = std::mem::take(&mut self.lines);
                let existing_chars = if let Some(lines) = lines {
                    let existing_chars = lines.into_iter().flat_map(|tlist| tlist.line);
                    Some(existing_chars)
                } else {
                    None
                };

                let new_chars = async_cache
                    .try_get()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .clone()
                    .iter()
                    .take(batch_size as usize)
                    .flat_map(|text| {
                        std::iter::once(TypeChar::space())
                            .chain(text.chars().map(|c| TypeChar {
                                char: c,
                                state: CharacterState::NotTyped,
                            }))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                if let Some(existing_chars) = existing_chars {
                    let combined = existing_chars.chain(new_chars);
                    self.lines = Some(Self::get_lines_from_iterator(combined, width_max));
                } else {
                    self.lines = Some(Self::get_lines_from_iterator(
                        new_chars.into_iter().skip(1),
                        width_max,
                    ));
                }
            }
            AsyncTextSource::StaticText(async_cache) => {
                // generate text as many times as poosible to have more than the requested batch size
                assert!(async_cache.is_ready());
                let batch_size = 10;
                debug!("Generating new batch of text with size: {}", batch_size);
                let text = async_cache
                    .try_get()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .expect("Text should be available")
                    .clone();
                let lines = std::mem::take(&mut self.lines);
                let existing_chars = if let Some(lines) = lines {
                    let existing_chars = lines.into_iter().flat_map(|tlist| tlist.line);
                    Some(existing_chars)
                } else {
                    None
                };
                let type_char_text = Self::get_new_typechar_text(&text, batch_size);
                if let Some(existing_chars) = existing_chars {
                    let combined = existing_chars
                        .chain(vec![TypeChar::space()])
                        .chain(type_char_text);
                    self.lines = Some(Self::get_lines_from_iterator(combined, width_max));
                } else {
                    self.lines = Some(Self::get_lines_from_iterator(
                        type_char_text.into_iter(),
                        width_max,
                    ));
                }
            }
        }
    }

    fn previous_typechar(&self, n: usize) -> TypeChar {
        let lines = self.lines.as_ref().expect("A line to exist");
        if n <= self.pos_in_line {
            lines[self.n_line].line[self.pos_in_line - n]
        } else if self.n_line == 0 {
            return lines[0].line[0];
        } else {
            let mut n = n - self.pos_in_line;
            let mut n_line = self.n_line - 1;
            loop {
                let len = lines[n_line].line.len();
                if n > len && n_line > 0 {
                    n -= len;
                    n_line -= 1;
                } else if n > len {
                    return lines[0].line[0];
                } else {
                    return lines[n_line].line[len - n];
                }
            }
        }
    }
    fn make_incorrect(&mut self) {
        if let Some(lines) = self.lines.as_mut() {
            lines[self.n_line].line[self.pos_in_line].incorrect();
        }
    }

    fn make_typed(&mut self) {
        if let Some(lines) = self.lines.as_mut() {
            lines[self.n_line].line[self.pos_in_line].typed();
        }
    }

    fn make_not_typed(&mut self) {
        if let Some(lines) = self.lines.as_mut() {
            lines[self.n_line].line[self.pos_in_line].not_typed();
        }
    }
    fn make_selected(&mut self) {
        if let Some(lines) = self.lines.as_mut() {
            lines[self.n_line].line[self.pos_in_line].selected();
        }
    }

    fn is_current_char(&self, key: char) -> bool {
        assert!(self.lines.is_some());
        if let Some(lines) = self.lines.as_ref() {
            lines[self.n_line].line[self.pos_in_line].char == key
        } else {
            false
        }
    }
    fn is_word_correctly_type(&self) -> bool {
        let mut i = 0;
        loop {
            let type_char = self.previous_typechar(i);
            if !matches!(type_char.state, CharacterState::Typed) {
                return false;
            }
            if (type_char.char == ' ' && i != 0) || self.total_position - i == 0 {
                return true;
            }
            i += 1;
        }
    }
    fn update_time_word_done(&mut self) {
        self.wps_tracker.record_word();
    }

    pub fn wpm(&self) -> f32 {
        self.wps_tracker.wps()
    }

    fn cursor_char(&self) -> TypeChar {
        assert!(self.lines.is_some());
        self.lines.as_ref().unwrap()[self.n_line].line[self.pos_in_line]
    }
    pub fn is_cursor_correct(&self) -> bool {
        matches!(self.cursor_state(), CharacterState::Typed)
    }
    fn cursor_state(&self) -> CharacterState {
        self.cursor_char().state
    }
    fn is_cursor_incorrect(&self) -> bool {
        matches!(self.cursor_state(), CharacterState::Incorrect)
    }

    fn get_lines(typechar_text: Vec<TypeChar>, width_max: u16) -> Vec<TypeLine> {
        Self::get_lines_from_iterator(typechar_text.into_iter(), width_max)
    }

    fn get_lines_from_iterator<I>(typechar_text_iter: I, width_max: u16) -> Vec<TypeLine>
    where
        I: Iterator<Item = TypeChar>,
    {
        // Define the struct locally within the function
        struct FoldState {
            lines: Vec<Vec<TypeChar>>,
            position: u16,
            current_word: Vec<TypeChar>,
        }

        let initial_state = FoldState {
            lines: vec![Vec::new()],
            position: 0,
            current_word: Vec::new(),
        };

        let final_state = typechar_text_iter
            .chain(iter::once(TypeChar::guard()))
            .fold(initial_state, |mut state, tchar| {
                state.current_word.push(tchar);
                if tchar.char == ' ' {
                    if state.position < width_max - 1 {
                        let last = state
                            .lines
                            .last_mut()
                            .expect("Lines vector should not be empty");
                        last.append(&mut state.current_word);
                        state.position += 1;
                    } else {
                        state.position = state.current_word.len() as u16;
                        state.lines.push(state.current_word);
                        state.current_word = Vec::new();
                    }
                } else {
                    state.position += 1;
                };
                state
            });

        let mut result_lines = final_state.lines;
        result_lines
            .last_mut()
            .expect("Lines vector should not be empty")
            .pop();

        result_lines.into_iter().map(TypeLine::new).collect()
    }

    fn correct_position(&mut self) {
        assert!(self.lines.is_some());
        let mut acc = 0;
        for (i, line) in self.lines.as_ref().unwrap().iter().enumerate() {
            let new_acc = acc + line.line.len();
            if new_acc >= self.total_position {
                self.n_line = i;
                break;
            } else {
                acc = new_acc
            }
        }
        self.pos_in_line = self.total_position - acc;
        // println!(" newpos: {}", self.pos_in_line);
    }

    fn reformat(&mut self, new_max_width: u16) {
        assert!(self.lines.is_some());
        let lines = std::mem::take(&mut self.lines);
        let typechar_text_iter = lines.unwrap().into_iter().flat_map(|tlist| tlist.line);
        self.lines = Some(Self::get_lines_from_iterator(
            typechar_text_iter,
            new_max_width,
        ));
        self.correct_position();
        self.current_width = new_max_width;
    }

    pub fn get_n_words_correctly_typed(&self) -> usize {
        if self.lines.is_none() {
            return 0;
        };

        let mut counter = 0;
        let mut is_correct = true;
        'lines: for line in self.lines.as_ref().unwrap() {
            for tchar in &line.line {
                match tchar.state {
                    CharacterState::Typed | CharacterState::Selected if tchar.is_space() => {
                        if is_correct {
                            counter += 1
                        };
                        is_correct = true
                    }
                    CharacterState::Incorrect => is_correct = false,
                    CharacterState::NotTyped | CharacterState::Selected => break 'lines,
                    CharacterState::Typed => {}
                }
            }
            is_correct = true
        }
        counter
    }
}

pub enum SettingsText {
    ThreeLine,
    Max,
    Centered,
}

impl TextWidget {
    fn render_n_lines(&mut self, area: Rect, buf: &mut Buffer, n: usize) {
        assert!(self.lines.is_some());
        let n = std::cmp::min(n, area.height as usize);
        if self.lines.as_ref().unwrap().is_empty() {
            return;
        }
        let start_idx = if self.n_line == 0 || self.n_line >= self.lines.as_ref().unwrap().len() {
            0
        } else {
            self.n_line.saturating_sub(n / 2)
        };
        if matches!(self.kind_length, KindLength::Unlimited)
            && self.n_line + n >= self.lines.as_ref().unwrap().len()
        {
            self.genrate_new_batch(area.width * (n * 2) as u16, area.width);
        }
        let display_lines = self.lines.as_ref().unwrap().iter().skip(start_idx).take(n); // TODO: check if access in o(1)

        let y_first_line = area.y + (area.height - n as u16) / 2;
        for (i, line) in display_lines.enumerate() {
            let new_area = Rect::new(area.x, y_first_line + i as u16, area.width, 1);
            line.clone().render(new_area, buf);
        }
    }
}

impl StatefulWidget for &mut TextWidget {
    type State = SettingsText;
    fn render(self, area: Rect, buf: &mut Buffer, _settings: &mut Self::State) { // TODO implement multiple kinds of rendering
        match self.state {
            TextWidgetState::Loading => {
                let msg = "Loading...";
                let area = centered_rect_with_length(msg.len() as u16, 1, area);
                let span = Span::styled(msg, Style::new().italic());
                span.render(area, buf);
            }
            TextWidgetState::ErrorLoading => {
                let msg = "An error occured while opening the text's file...";
                let area = centered_rect_with_length(msg.len() as u16, 1, area);
                let span = Span::styled(msg, Style::new().italic());
                span.render(area, buf);
            }
            TextWidgetState::InProgress | TextWidgetState::DoneWithMistakes => {
                if area.width != self.current_width {
                    self.reformat(area.width);
                }
                self.render_n_lines(area, buf, 5);
            }
            TextWidgetState::Done => {}
        }

        // for (i,line ) in self.lines[0..std::cmp::min(self.lines.len(), area.height as usize)].iter().enumerate(){
        //     let new_area = Rect::new(area.x, area.y + i as u16, area.width, 1);
        //     line.clone().render(new_area, buf);
        // }
    }
}

#[derive(Debug)]
pub struct TextWidgetComponent {
    dispatcher_tx: UnboundedSender<Action>,

    pub widget: TextWidget,
}
impl TextWidgetComponent {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        origin: TextOrigin,
        filename: String,
        number_words: Option<usize>,
        offset: Option<f64>,
        seed: Option<u64>,
    ) -> Self {
        let clone_dispatcher_tx = dispatcher_tx.clone();

        TextWidgetComponent {
            dispatcher_tx,
            widget: TextWidget::new(
                origin,
                filename,
                clone_dispatcher_tx,
                number_words,
                offset,
                seed,
            ),
        }
    }

    pub fn current_ratio(&self) -> f32 {
        if self.widget.total_size == 0 || self.is_done() {
            0.0
        } else {
            self.widget.total_position as f32 / self.widget.total_size as f32
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self.widget.state, TextWidgetState::Done | TextWidgetState::DoneWithMistakes)
    }
    pub fn is_done_correctly(&self) -> bool {
        matches!(self.widget.state, TextWidgetState::Done)
    }

    fn back_cursor(&mut self) {
        assert!(self.widget.lines.is_some());
        match self.widget.state {
            TextWidgetState::InProgress => {
                if let Some(pos_in_line) = self.widget.pos_in_line.checked_sub(1) {
                    self.widget.pos_in_line = pos_in_line;
                    self.widget.total_position -= 1;
                } else if let Some(n_line) = self.widget.n_line.checked_sub(1) {
                    self.widget.n_line = n_line;
                    self.widget.pos_in_line =
                        self.widget.lines.as_ref().unwrap()[n_line].line.len() - 1;
                    self.widget.total_position -= 1;
                }
            }
            TextWidgetState::DoneWithMistakes => {}
            TextWidgetState::Done => {}
            _ => panic!("Should never happen"),
        }
    }

    fn advance_cursor(&mut self) -> bool {
        assert!(self.widget.lines.is_some());
        if self.widget.pos_in_line + 1
            < self.widget.lines.as_ref().unwrap()[self.widget.n_line]
                .line
                .len()
        {
            self.widget.pos_in_line += 1;
            self.widget.total_position += 1;
            true
        } else if self.widget.n_line + 1 < self.widget.lines.as_ref().unwrap().len() {
            self.widget.pos_in_line = 0;
            self.widget.n_line += 1;
            self.widget.total_position += 1;
            true
        } else {
            false
        }
    }

    fn key_pressed(&mut self, key: char) {
        // we asume that enter are equilvalent to space
        let key = if key == '\n' { ' ' } else { key };
        match self.widget.state {
            TextWidgetState::InProgress => {
                if self.widget.is_current_char(key) {
                    self.widget.make_typed();
                    if key == ' ' && self.widget.is_word_correctly_type() {
                        self.widget.update_time_word_done();
                    }
                } else {
                    self.widget.make_incorrect();
                    if !self.is_done() {
                        self.widget.mistakes_counter += 1;
                    }
                };
                if self.advance_cursor() {
                    self.widget.make_selected();
                } else if matches!(self.widget.kind_length, KindLength::Finished(_)) {
                    self.widget.state = if self.widget.mistakes_counter > 0 {
                        TextWidgetState::DoneWithMistakes
                    } else {
                        // self.process_end_game();
                        TextWidgetState::Done
                    };
                }
            }
            TextWidgetState::DoneWithMistakes => {
                if self.widget.is_current_char(key) {
                    debug!("a: {}", self.widget.is_cursor_incorrect());
                    if self.widget.is_cursor_incorrect() {
                        self.widget.mistakes_counter -= 1;
                    }
                    self.widget.make_typed();
                } else {
                    if self.widget.is_cursor_correct() {
                        self.widget.mistakes_counter += 1;
                    }
                    self.widget.make_incorrect();
                };
                self.widget.state = if self.widget.mistakes_counter > 0 {
                    TextWidgetState::DoneWithMistakes
                } else {
                    TextWidgetState::Done
                };
            }
            TextWidgetState::Done | TextWidgetState::Loading | TextWidgetState::ErrorLoading => {}
        }
    }
    fn backspace(&mut self) {
        if self.widget.lines.is_none() {
            return;
        }
        // with the last character the postion of the cursor is a bit special
        // we check if the current character is wrong also for that
        if self.widget.is_cursor_incorrect() {
            self.widget.mistakes_counter -= 1
        }

        self.widget.make_not_typed();
        self.back_cursor();
        if self.widget.is_cursor_incorrect() {
            self.widget.mistakes_counter -= 1
        }

        self.widget.make_selected();
        if !matches!(self.widget.state, TextWidgetState::InProgress) {
            self.widget.state = TextWidgetState::InProgress
        };
    }

    pub fn wpm(&self) -> f32 {
        self.widget.wpm()
    }
    pub fn get_n_words_correctly_typed(&self) -> usize {
        self.widget.get_n_words_correctly_typed()
    }
}

impl Store for TextWidgetComponent {
    fn update(&mut self, action: Action) {
        match action {
            Action::KeyPressed(key) => self.key_pressed(key),
            Action::BackspacePressed => self.backspace(),
            Action::AsyncCachedRecievedData(_)
                if self.widget.lines.is_none() && self.widget.async_text_source.is_available() =>
            {
                self.widget.init()
            }
            _ => {}
        }
        debug!("Mistake counter: {}", self.widget.mistakes_counter)
    }
}

impl SendAction for TextWidgetComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

