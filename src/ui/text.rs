use std::{iter, slice::{self, Iter}};

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Stylize}, symbols::line, text::{Line, Span}, widgets::{StatefulWidget, Widget}};
use serde_json::map::IntoIter;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, flux::{self, SendAction}, stores::Store, text_generator::{get_text, TextGenerator, TextGeneratorIntoIter}};

use super::{gauge::{self, GaugeId}, screens::{Screen, ScreenMember}};

const LOREM_LIPSUM: &str = "Lorem ipsum dolor  sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut";

#[derive(Copy, Clone, Debug)]
enum CharacterState {
    NotTyped,
    Typed,
    Incorrect,
    Selected,
}

#[derive(Copy, Clone, Debug)]
struct TypeChar {
    char: char,
    state: CharacterState,
}
impl TypeChar  {
    fn incorrect(&mut self) -> Self {
        self.state = CharacterState::Incorrect;
        *self
    }
    fn typed(&mut self) -> Self{
        self.state = CharacterState::Typed;
        *self
    }
    fn not_typed(&mut self) -> Self{
        self.state = CharacterState::NotTyped;
        *self
    }
    fn selected(&mut self) -> Self{
        self.state = CharacterState::Selected;
        *self
    }
    fn guard() -> TypeChar {
        TypeChar::space()
    }
    fn space() -> TypeChar {
        TypeChar { char: ' ', state: CharacterState::NotTyped }
    }
}

impl<'a> From<TypeChar> for Span<'a> {
    fn from(tc: TypeChar) -> Span<'a> {
        match tc.state {
            CharacterState::NotTyped => 
                        Span::raw(tc.char.to_string()).dark_gray(),
            CharacterState::Typed => 
                        Span::raw(tc.char.to_string()).white(),
            CharacterState::Incorrect => 
                        Span::raw(tc.char.to_string()).bg(Color::Red),
            CharacterState::Selected => 
                        Span::raw(tc.char.to_string()).yellow().underlined(),
        }
    }
}

#[derive(Debug, Clone)]
struct TypeLine {
    line: Vec<TypeChar>
} 
impl TypeLine {
    fn new(line: Vec<TypeChar>) -> TypeLine {
        TypeLine { line }
    }
}

impl Widget for  TypeLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(self.line.into_iter().map(|el| el.into()).collect::<Vec<Span>>());
        buf.set_line(area.x, area.y, &line, area.width);
    }
}







#[derive(Debug)]
enum KindLength<I: Iterator<Item = String>> {
    Unlimited(I),
    Finished
}

#[derive(Debug)]
enum TextWidgetState {
    InProgress,
    DoneWithMistakes,
    Done
}

#[derive(Debug)]
pub struct TextWidget {
    kind_length: KindLength<TextGeneratorIntoIter>,

    current_width: u16,
    lines: Vec<TypeLine>,

    pos_in_line: usize,
    n_line: usize,
    total_position: usize,

    total_size: usize,
    mistakes_counter: usize,
    state: TextWidgetState
} 


impl TextWidget  {
    fn new(text: String) -> Self {
        let typechar_text: Vec<TypeChar> = text
            .chars()
            .map(|c| TypeChar { char: c, state: CharacterState::NotTyped })
            .collect();
        let total_size = typechar_text.len();
        let lines = Self::get_lines(typechar_text, 0);
        TextWidget { current_width: 0, lines, pos_in_line: 0, n_line: 0, total_position: 0, total_size, state: TextWidgetState::InProgress, mistakes_counter: 0, kind_length: KindLength::Finished }
    }

    fn from_language(name: &str) -> Self{
        let first_batch = 500;
        let new_chars = TextGenerator::from_language(name, None)
                    .into_iter()
                    .take(first_batch)
                    .flat_map(|text| {
                        std::iter::once(TypeChar::space()) 
                            .chain(text.chars().map(|c| TypeChar {
                                char: c,
                                state: CharacterState::NotTyped,
                            }))
                            .collect::<Vec<_>>() 
                            .into_iter()
                    }).skip(1);
        let text_generator_into_iter = TextGenerator::from_language(name, None).into_iter().skip_n(first_batch);
        let lines =  Self::get_lines_from_iterator(
                    new_chars,
                    100,
                );
        TextWidget { current_width: 0, lines, pos_in_line: 0, n_line: 0, total_position: 0, total_size: 0, state: TextWidgetState::InProgress, mistakes_counter: 0, kind_length: KindLength::Unlimited(text_generator_into_iter) }
    } 

    fn from_text(name: &str) -> Self {
        let typechar_text: Vec<TypeChar> = get_text(name).unwrap()
            .chars()
            .map(|c| TypeChar { char: c, state: CharacterState::NotTyped })
            .collect();
        let total_size = typechar_text.len();
        let lines = Self::get_lines(typechar_text, 100);
        TextWidget { current_width: 0, lines, pos_in_line: 0, n_line: 0, total_position: 0, total_size, state: TextWidgetState::InProgress, mistakes_counter: 0, kind_length: KindLength::Finished }
    }

    fn genrate_new_batch(&mut self, batch_size: u16, width_max: u16) {
        match &mut self.kind_length {
            KindLength::Unlimited(text_generator_into_iter) => {
                let lines = std::mem::take(&mut self.lines);
                let existing_chars = lines.into_iter()
                    .flat_map(|tlist| tlist.line);

            
                let new_chars = text_generator_into_iter
                    .take(batch_size as usize)
                    .flat_map(|text| {
                        std::iter::once(TypeChar::space()) 
                            .chain(text.chars().map(|c| TypeChar {
                                char: c,
                                state: CharacterState::NotTyped,
                            }))
                            .collect::<Vec<_>>() 
                            .into_iter()
                    });

                
                self.lines =  Self::get_lines_from_iterator(
                    existing_chars.chain(new_chars),
                    width_max,
                );

            },
            KindLength::Finished => panic!("Impossible to generate new text"),
        }
    }

    fn make_incorrect(&mut self){
         self.lines[self.n_line].line[self.pos_in_line].incorrect();
    }

    fn make_typed(&mut self){
         self.lines[self.n_line].line[self.pos_in_line].typed();
    }

    fn make_not_typed(&mut self){
         self.lines[self.n_line].line[self.pos_in_line].not_typed();
    }
    fn make_selected(&mut self){
         self.lines[self.n_line].line[self.pos_in_line].selected();
    }

    fn is_current_char(&self, key: char) -> bool {
        self.lines[self.n_line].line[self.pos_in_line].char == key
    }
    
    fn cursor_char(&self) -> TypeChar {
        self.lines[self.n_line].line[self.pos_in_line]
    }
    fn cursor_state(&self) -> CharacterState {
        self.cursor_char().state
    }
    fn cursor_incorrect(&self) -> bool {
        matches!(self.cursor_state(), CharacterState::Incorrect)
    }

    fn get_lines(typechar_text: Vec<TypeChar>, width_max: u16) -> Vec<TypeLine> {
        Self::get_lines_from_iterator(typechar_text.into_iter(), width_max)
    }

    fn get_lines_from_iterator<I>(typechar_text_iter: I, width_max: u16) -> Vec<TypeLine>
    where 
        I: Iterator<Item = TypeChar>
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
                        let last = state.lines.last_mut().expect("Lines vector should not be empty");
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
        result_lines.last_mut().expect("Lines vector should not be empty").pop();
        
        result_lines.into_iter().map(|line| {
            TypeLine::new(line)
        }).collect()
    }

    fn correct_position(&mut self){
        let mut acc = 0;
        for (i, line) in self.lines.iter().enumerate() {
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

    fn reformat(&mut self, new_max_width: u16){
        let lines = std::mem::take(&mut self.lines);
        let typechar_text_iter = lines.into_iter().flat_map(|tlist|tlist.line);
        self.lines = Self::get_lines_from_iterator(typechar_text_iter, new_max_width);
        self.correct_position();
        self.current_width = new_max_width;
    }

    

}

impl Default for TextWidget {
    fn default() -> Self {
        // Self::from_language("french_1k.json")
        Self::from_text("lotr.txt")
        //Self::new( bee_script())
        //Self::new( LOREM_LIPSUM.to_string(), 80)
    }
}

pub enum SettingsText {
    ThreeLine,
    Max,
    Centered,
}

impl TextWidget {
    fn render_n_lines(&mut self, area: Rect, buf: &mut Buffer, n: usize){
        let n = std::cmp::min(n, area.height as usize);
        if self.lines.is_empty() { return; }
        let start_idx = if self.n_line == 0 || self.n_line >= self.lines.len() {
            0
        } else {
            self.n_line.checked_sub(n / 2).unwrap_or(0)
        };
        if matches!(self.kind_length, KindLength::Unlimited(_)) {
            if self.n_line + n/2 >= self.lines.len() {
                self.genrate_new_batch(area.width * 20, area.width);
            }
        }
        let display_lines = self.lines
            .iter()
            .skip(start_idx)
            .take(n); // TODO: check if access in o(1)
           
        let y_first_line = area.y + (area.height - n as u16) / 2;
        for (i,line ) in display_lines.enumerate(){
            let new_area = Rect::new(area.x, y_first_line + i as u16, area.width, 1);
            line.clone().render(new_area, buf);
        }
    }
}

impl StatefulWidget for &mut TextWidget {
    type State = SettingsText;
    fn render(self, area: Rect, buf: &mut Buffer, settings: &mut Self::State) {
        if area.width != self.current_width {self.reformat(area.width);}
        self.render_n_lines(area, buf, 5);
        // for (i,line ) in self.lines[0..std::cmp::min(self.lines.len(), area.height as usize)].iter().enumerate(){
        //     let new_area = Rect::new(area.x, area.y + i as u16, area.width, 1);
        //     line.clone().render(new_area, buf);
        // }
    }
}




#[derive(Debug)]
pub struct TextWidgetComponent {
    is_active: bool,
    dispatcher_tx: UnboundedSender<Action>,
    screen: Screen,
    pub widget: TextWidget,
    linked_progress_bars: Vec<gauge::GaugeId>,

}
impl TextWidgetComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, screen: Screen) -> Self {
        TextWidgetComponent { screen, dispatcher_tx, widget: TextWidget::default(), is_active: true, linked_progress_bars: Vec::new() }
    }

    pub fn link_progress_bars(&mut self, id: GaugeId){
        self.linked_progress_bars.push(id);
    }

    pub fn current_ratio(&self) -> f32 {
    
        if self.widget.total_size == 0 || self.is_done() {1.0} else {self.widget.total_position as f32 / self.widget.total_size as f32}
        
    }

    pub fn is_done(&self) -> bool {
        match self.widget.state {
            TextWidgetState::InProgress => false,
            TextWidgetState::Done | TextWidgetState::DoneWithMistakes => true,
        }
    }
    pub fn is_done_correctly(&self) -> bool {
        match self.widget.state {
            TextWidgetState::InProgress | TextWidgetState::DoneWithMistakes => false,
            TextWidgetState::Done  => true,
        }
    }

    fn update_linked_gauges(&self) {
        if self.linked_progress_bars.is_empty() {return};
        let ratio = self.current_ratio();
        for  progress_bar in self.linked_progress_bars.iter() {
            self.send(Action::UpdateLineGauge(*progress_bar, ratio)).unwrap()
        }
    }

    fn back_cursor(&mut self){
        match self.widget.state {
            TextWidgetState::InProgress => {
                if let Some(pos_in_line) = self.widget.pos_in_line.checked_sub(1) {
                    self.widget.pos_in_line = pos_in_line;
                    self.widget.total_position -= 1; 
                }else {
                    if let Some(n_line) = self.widget.n_line.checked_sub(1) {
                        self.widget.n_line = n_line;
                        self.widget.pos_in_line = self.widget.lines[n_line].line.len() - 1;
                        self.widget.total_position -= 1; 
                    }
                }
            },
            TextWidgetState::DoneWithMistakes => {},
            TextWidgetState::Done => {},
        }
        
        
    }

    fn advance_cursor(&mut self) -> bool{
        if self.widget.pos_in_line + 1 < self.widget.lines[self.widget.n_line].line.len(){
            self.widget.pos_in_line += 1;
            self.widget.total_position += 1;
            true
        } else {
            if self.widget.n_line + 1 < self.widget.lines.len() {
                self.widget.pos_in_line = 0;
                self.widget.n_line += 1;
                self.widget.total_position +=1;
                true
            } else {
                false
            }
        }

    }

    fn key_pressed(&mut self, key: char){
        match self.widget.state {
            TextWidgetState::InProgress => {
                if self.widget.is_current_char(key){
                    self.widget.make_typed();
                } else {
                    self.widget.mistakes_counter += 1;
                    self.widget.make_incorrect();
                };       
                if self.advance_cursor() {
                    self.widget.make_selected();
                } else if matches!(self.widget.kind_length, KindLength::Finished){
                    self.widget.state = if self.widget.mistakes_counter > 0  { TextWidgetState::DoneWithMistakes } else { TextWidgetState::Done };
                }
            },
            TextWidgetState::DoneWithMistakes => {
                if self.widget.cursor_incorrect() {self.widget.mistakes_counter += 1;}
                if self.widget.is_current_char(key){
                    self.widget.make_typed();
                } else {
                    self.widget.mistakes_counter += 1;
                    self.widget.make_incorrect();
                };
                self.widget.state = if self.widget.mistakes_counter > 0 { TextWidgetState::DoneWithMistakes } else { TextWidgetState::Done };
            },
            TextWidgetState::Done => {},
        }
        

    }
    fn backspace(&mut self){
        self.widget.make_not_typed();
        self.back_cursor();
        if self.widget.cursor_incorrect() {
            self.widget.mistakes_counter -= 1}
        self.widget.make_selected();
         if !matches!(self.widget.state, TextWidgetState::InProgress) { self.widget.state = TextWidgetState::InProgress };

    }
    
    

}



impl Store for TextWidgetComponent {
    fn update(&mut self, action: Action,  ){
        match action {
            Action::KeyPressed(key) if self.is_active => self.key_pressed(key),
            Action::BackspacePressed if self.is_active => self.backspace(),
            _ => {}
        }
    }
}

impl ScreenMember for TextWidgetComponent {
    fn screen(&self) -> Screen {
        self.screen
    }

    fn deactivate(&mut self) {
        self.is_active = false
    }

    fn activate(&mut self) {
        self.is_active = true
    }
}
impl SendAction for TextWidgetComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}



fn bee_script() -> String{
    get_text("bee_script.txt").unwrap()
}
fn lotr_script() -> String{
    get_text("lotr.txt").unwrap()
}