use std::iter;

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Stylize}, text::{Line, Span}, widgets::{StatefulWidget, Widget}};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::App, flux::SendAction, stores::Store};

use super::screens::{Screen, ScreenMember};

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
pub struct TextWidget {

    current_width: u16,
    lines: Vec<TypeLine>,

    pos_in_line: usize,
    n_line: usize,
    total_position: usize
}


impl TextWidget  {
    fn new(text: String, width_max: u16) -> Self {
        let typechar_text: Vec<TypeChar> = text
            .chars()
            .map(|c| TypeChar { char: c, state: CharacterState::NotTyped })
            .collect();
        let lines = Self::get_lines(typechar_text, width_max);
        TextWidget { current_width: width_max, lines, pos_in_line: 0, n_line: 0, total_position: 0 }

    }

    fn back_cursor(&mut self){
        if let Some(pos_in_line) = self.pos_in_line.checked_sub(1) {
           self.pos_in_line = pos_in_line;
           self.total_position -= 1; 
        }else {
            if let Some(n_line) = self.n_line.checked_sub(1) {
                self.n_line = n_line;
                self.pos_in_line = self.lines[n_line].line.len() - 1;
                self.total_position -= 1; 
            }
        }
    }

    fn advance_cursor(&mut self){
        if self.pos_in_line + 1 < self.lines[self.n_line].line.len(){
            self.pos_in_line += 1;
            self.total_position += 1;
        } else {
            if self.n_line + 1 < self.lines.len() {
                self.pos_in_line = 0;
                self.n_line += 1;
                self.total_position +=1;
            }
        }
    }

    fn is_current_char(&self, key: char) -> bool {
        self.lines[self.n_line].line[self.pos_in_line].char == key
    }
   
    fn key_pressed(&mut self, key: char){
        if self.is_current_char(key){
            self.make_typed();
        }else{
            self.make_incorrect();
        }
        self.advance_cursor();
        self.make_selected();

    }
    fn backspace(&mut self){
        self.make_not_typed();
        self.back_cursor();
        self.make_selected();
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
        Self::new( bee_script(), 80)
        // Self::new( LOREM_LIPSUM.to_string(), 80)
    }
}

pub enum SettingsText {
    ThreeLine,
    Max,
    Centered,
}

impl TextWidget {
    fn render_n_lines(&self, area: Rect, buf: &mut Buffer, n: usize){
        let n = std::cmp::min(n, area.height as usize);
        if self.lines.is_empty() {
        return;
        }
        let start_idx = if self.n_line == 0 || self.n_line >= self.lines.len() {
            0
        } else {
            self.n_line - n / 2
        };

        let display_lines = self.lines
            .iter()
            .skip(start_idx)
            .take(n);
           
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
        self.render_n_lines(area, buf, 4);
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
    pub widget: TextWidget

}
impl TextWidgetComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>, screen: Screen) -> Self {
        TextWidgetComponent { screen, dispatcher_tx, widget: TextWidget::default(), is_active: true }
    }
}

impl Store for TextWidgetComponent {
    fn update(&mut self, action: Action,  ){
        match action {
            Action::KeyPressed(key) if self.is_active => self.widget.key_pressed(key),
            Action::BackspacePressed if self.is_active => self.widget.backspace(),
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





use std::fs::File;
use std::io::{self, BufReader, Read};

fn read_file(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn bee_script() -> String{
    read_file("texts/bee_script.txt").unwrap()
}
fn lotr_script() -> String{
    read_file("texts/lotr.txt").unwrap()
}