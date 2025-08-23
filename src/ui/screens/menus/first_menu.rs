use ratatui::widgets::{Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    flux::SendAction,
    stores::Store,
    ui::{list::MenuListComponent, screens::IsScreen},
};

use super::super::{super::*, Screen};



#[derive(Debug)]
pub struct FirstMenuScreen {
    dispatcher_tx: UnboundedSender<Action>,
    list_store: MenuListComponent,
}
impl FirstMenuScreen {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self {
        let options = ["Solo", "Multi (in progress)", "Settings", "About"]
            .into_iter()
            .map(|el| el.to_string())
            .collect();
        let actions = vec![
            Action::AskChangeToScreen(Screen::SoloGamesMenu),
            Action::None,
            Action::None,
            Action::None,
        ];
        let list_store = MenuListComponent::new(
            "Menu".to_string(),
            dispatcher_tx.clone(),
            options,
            actions,
        );

        FirstMenuScreen {
            dispatcher_tx,
            list_store,
        }
    }
}

impl Store for FirstMenuScreen {
    fn update(&mut self, action: Action) {
        self.list_store.update(action);
    }
}

impl Widget for &FirstMenuScreen {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = centered_rect_with_length(std::cmp::min(50, area.width), 6, area);
        self.list_store.render(area, buf);
    }
}

impl SendAction for FirstMenuScreen {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl IsScreen for FirstMenuScreen {}
