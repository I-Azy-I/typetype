use ratatui::{style::{Style, Stylize}, widgets::{Block, List, ListState, StatefulWidget, Widget}};
use tokio::sync::mpsc::UnboundedSender;
use crate::{action::Action, flux::SendAction, stores::Store};

use super::{screens::{Screen, ScreenMember}};

pub struct MenuList<'a> {
    store: &'a MenuListComponent,
}

impl<'a> MenuList<'a> {
    pub fn new(store: &'a MenuListComponent) -> Self {
        MenuList { store}
    }
    
}

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
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true); 
        StatefulWidget::render(list, area, buf, &mut self.list_state.clone());
    }
}