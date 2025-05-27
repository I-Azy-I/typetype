use std::default;

use ratatui::{layout::{self, Constraint, Direction, Layout}, style::{Style, Stylize}, text::Span, widgets::{canvas::Line, Block, List, ListState, StatefulWidget, Widget}};
use tokio::{sync::mpsc::UnboundedSender, time::error::Elapsed};
use crate::{action::Action, flux::SendAction, stores::Store};

use super::{centered_rect, centered_rect_with_length, screens::{Screen, ScreenMember}};


#[derive(Debug)]
struct Entry {
    option: String,
    action: Action
}

#[derive(Debug)]
pub struct MenuListComponent {
    dispatcher_tx: UnboundedSender<Action>,
    is_active: bool,
    list_state: ListState,
    entries: Vec<Entry>,
    screen: Screen,
    title: String
}

impl MenuListComponent {
    pub fn new(title: String, dispatcher_tx: UnboundedSender<Action>, options:  Vec<String>, actions: Vec<Action>, screen: Screen) -> Self {
        assert!(options.len() == actions.len());
        let entries = options.into_iter().zip(actions.into_iter()).map(|(option, action)| Entry {option, action: action});
        MenuListComponent { title, dispatcher_tx, is_active: false, list_state: ListState::default().with_selected(Some(0)), entries: entries.collect(), screen}
    }
    pub fn get_options(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.option.clone()).collect()
    }

    pub fn get_actions(&self) -> Vec<Action> {
        self.entries.iter().map(|entry| entry.action.clone()).collect()
    }
}
impl Store for MenuListComponent {
    fn update(&mut self, action: crate::action::Action) {
        match action { 
            Action::DownPressed if self.is_active && self.list_state.selected().is_some_and(|value| value < self.entries.len() -1) => {self.list_state.select_next()},
            Action::UpPressed if self.is_active => self.list_state.select_previous(),
            Action::EnterPressed if self.is_active =>  {
                if let Some(selected) = self.list_state.selected() {
                        self.send(self.get_actions()[selected]).unwrap()
                    }
                }
            _ => {}
        }
    }
}

impl SendAction for MenuListComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl ScreenMember for MenuListComponent {
    fn screen(&self) -> super::screens::Screen {
        self.screen
    }

    fn deactivate(&mut self) {
        self.is_active = false
    }

    fn activate(&mut self) {
        self.is_active = true
    }
}


impl Widget for &MenuListComponent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        if !self.is_active{return;}
        // let area = centered_rect(70, 70, area);
        let items = self.get_options();
        let list = List::new(items)
            .block(Block::bordered().title(self.title.clone()))
            .highlight_style(Style::new().reversed())
            .highlight_symbol("•")
            .repeat_highlight_symbol(true); 
        StatefulWidget::render(list, area, buf, &mut self.list_state.clone());
    }
}

#[derive(Debug)]
pub struct MutipleEntry {
    entries: Vec<Entry>,
}

impl MutipleEntry {
    pub fn new(options: Vec<String>, actions: Vec<Action>) -> Self {
        let entries = options.into_iter().zip(actions.into_iter()).map(|(option, action)|
        Entry { option, action}
        ).collect();
        MutipleEntry { entries }
    }
}

#[derive(Debug)]
pub struct MenuMultipleListComponent {
    dispatcher_tx: UnboundedSender<Action>,
    is_active: bool,
    list_state_line: ListState, // verticaly
    list_state_entry: ListState, // horizontaly
    entries: Vec<MutipleEntry>,
    constraints: Vec<Constraint>,
    screen: Screen,
    title: String
}

impl MenuMultipleListComponent {
    pub fn new(title: String, dispatcher_tx: UnboundedSender<Action>, entries: Vec<MutipleEntry>, constraints: Vec<Constraint>, screen: Screen) -> Self {
        assert!(entries.iter().all(|entry| entry.entries.len() == constraints.len()));

        MenuMultipleListComponent { 
            title, 
            dispatcher_tx, 
            is_active: false, 
            list_state_line: ListState::default().with_selected(Some(0)), 
            list_state_entry:  ListState::default().with_selected(Some(0)),
            constraints,
            entries, 
            screen
        }
    }
    pub fn get_options(&self, line: usize) -> Vec<String> {
        self.entries[line].entries.iter().map(|entry| entry.option.clone()).collect()
    }
    pub fn get_options_column(&self, column: usize) -> Vec<String> {
        self.entries.iter().map(|entry| entry.entries[column].option.clone()).collect()
    }

    pub fn get_actions(&self, line: usize) -> Vec<Action> {
        self.entries[line].entries.iter().map(|entry| entry.action.clone()).collect()
    }

    fn try_move_cursor_right(&mut self){
        match  (self.list_state_line.selected(), self.list_state_entry.selected()) {
            (Some(n_line), Some(n_entry)) => {
                if self.entries[n_line].entries.len() > n_entry + 1 {
                    self.list_state_entry.select_next();
                }
            }
            (_, _) => {
                println!("cursor not well initialized")
            }
        }
    }

    fn try_move_cursor_left(&mut self){
        match  (self.list_state_line.selected(), self.list_state_entry.selected()) {
            (Some(n_line), Some(n_entry)) => {
                if n_entry > 0 {
                    self.list_state_entry.select(Some((n_entry % self.entries[n_line].entries.len()) - 1));
                }
            }
            (_, _) => {
                println!("cursor not well initialized")
            }
        }
    }

    fn current_selected_action(&self) -> Option<Action>  {
        match  (self.list_state_line.selected(), self.list_state_entry.selected()) {
            (Some(n_line), Some(n_entry)) => {
                Some(self.entries[n_line].entries[n_entry % self.entries[n_line].entries.len()].action)
            }
            (_, _) => {
                println!("cursor not well initialized");
                None
            }
        }
    }
    fn get_state_list(&self, column: usize) -> Option<ListState>{
        if let Some(n_entry) = self.list_state_entry.selected() {
            if n_entry == column {
                Some(self.list_state_line.clone())
            } else {
                Some(ListState::default().with_selected(None))
            }
        } else {
            None
        }
    }


    
    
}
impl Store for MenuMultipleListComponent {
    fn update(&mut self, action: crate::action::Action) {
        match action { 
            Action::DownPressed if self.is_active && self.list_state_line.selected().is_some_and(|value| value < self.entries.len() -1) => {self.list_state_line.select_next()},
            Action::UpPressed if self.is_active => self.list_state_line.select_previous(),
            Action::RightPressed if self.is_active => self.try_move_cursor_right(),
            Action::LeftPressed if self.is_active => self.try_move_cursor_left(),
            Action::EnterPressed if self.is_active =>  self.send(self.current_selected_action().unwrap()).unwrap(),
            _ => {}
        }
    }
}

impl SendAction for MenuMultipleListComponent {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}

impl ScreenMember for MenuMultipleListComponent {
    fn screen(&self) -> super::screens::Screen {
        self.screen
    }

    fn deactivate(&mut self) {
        self.is_active = false
    }

    fn activate(&mut self) {
        self.is_active = true
    }
}


impl Widget for &MenuMultipleListComponent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        if !self.is_active{return;}
        let block = Block::bordered().title(self.title.clone());
        let inner_area_block = block.inner(area);
        block.render(area, buf);


        let list_layout = Layout::default() // cannot do a list into list, so list are splitted before, and not inside each line...
            .direction(Direction::Horizontal)
            .constraints(&self.constraints)
            .split(inner_area_block);
        // let area = centered_rect(70, 70, area);
        for (i, &list_area) in list_layout.into_iter().enumerate() {
            let items = self.get_options_column(i);
            let list = List::new(items)
            .highlight_style(Style::new().reversed())
            .highlight_symbol("•")
            .repeat_highlight_symbol(true); 
            StatefulWidget::render(list, list_area, buf, &mut self.get_state_list(i).unwrap());
        }
    }
}

pub struct HorizontalList<'a> {
    block: Option<Block<'a>>,
    items: Vec<String>,
    highlight_style: Style,
}
impl<'a> HorizontalList<'a> {
    pub fn new(items: Vec<String>) ->  HorizontalList<'a> {
        HorizontalList {
            block: None,
            items,
            highlight_style: Style::default()
        }
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlight_style = style.into();
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for &HorizontalList<'a> {
    type State = ListState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let size = self.items.len();
        let area =  if let Some(block) = self.block.as_ref() {
            let new_area = block.inner(area);
            block.render(area, buf);
            new_area
        } else {
            area
        };
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![Constraint::Ratio(1,size as u32);size]
                )
            .split(area);
        for (i, (&area, item)) in layout.iter().zip(self.items.iter()).enumerate() {
            let area = centered_rect_with_length(item.len() as u16, area.height, area);
            let style = if let Some(selected) = state.selected() {
                if i == selected {
                    self.highlight_style
                } else {
                    Style::default()
                }
            } else {
                Style::default()
            };

            let span =  Span::styled(
            item,
            style);
            span.render(area, buf);
        }
    }
}



#[derive(Default)]
pub struct ScrollableList {
    state: ListState,
    is_selected: bool,
    default_state: usize,
    content: Vec<String>,
    keep_selected: bool,
}

impl ScrollableList {
    pub fn with_default_state(mut self, value: usize) -> Self {
        self.default_state = value;
        self
    }

    pub fn with_content(mut self, content: Vec<String>) -> Self {
        self.content = content;
        self
    }
    pub fn keep_selected(mut self, keep: bool) -> Self {
        self.keep_selected = keep;
        self
    } 
    pub fn select(&mut self) -> &Self {
        self.is_selected = true;
        self
    }
    pub fn unselect(&mut self) -> &Self {
        self.is_selected = false;
        self
    }

    pub fn next(&mut self){
        if !self.is_selected{return}
   
    }

    pub fn previous(&mut self){
        if !self.is_selected{return}
    }
}

