use std::iter;

use ratatui::{layout::Size, style::{Style, Stylize}, widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget}};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::{action::Action, flux::SendAction, stores::Store, ui::list::{MenuList, MenuListComponent}};

use super::super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::Settings;

#[derive(Debug)]
pub struct SettingsScreenComponent {
    // a_component: Component
}
impl SettingsScreenComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        SettingsScreenComponent {}
    }
}

impl Store for  SettingsScreenComponent {
    fn update(&mut self, action: Action) {
        
    }
}

impl Widget for &SettingsScreenComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        //self.a_component.render(area, buf);
        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Percentage(100),
            ])
            .split(area);
         let line_numbers = (1..=100).map(|i| format!("{:>3} ", i)).collect::<String>();
        let content =
            iter::repeat("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n")
                .take(100)
                .collect::<String>();

        let content_size = Size::new(area.width, area.height);
        let mut scroll_view = ScrollView::new(content_size);

        // the layout doesn't have to be hardcoded like this, this is just an example
        scroll_view.render_widget(Paragraph::new(line_numbers), area);
        scroll_view.render_widget(Paragraph::new(content), area);

        let mut state = ScrollViewState::new();
        scroll_view.render(buf.area, buf, &mut state);
    }
}



