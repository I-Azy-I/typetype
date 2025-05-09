use ratatui::{style::{Style, Stylize}, widgets::{Block, List, ListState, StatefulWidget, Widget}};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

use crate::{action::Action, flux::SendAction, stores::Store, ui::{list::{MenuList, MenuListComponent}, text::{SettingsText, TextWidgetComponent}}};

use super::{super::*, Screen, ScreenMember};

const SCREEN: Screen = Screen::SoloSpeedGame;

#[derive(Debug)]
pub struct ScreenSoloSpeedGameComponent {
    screen: Screen,
    dispatcher_tx: UnboundedSender<Action>,
    text_component: Option<TextWidgetComponent>

}
impl ScreenSoloSpeedGameComponent {
    pub fn new(dispatcher_tx: UnboundedSender<Action>) -> Self{
        ScreenSoloSpeedGameComponent {screen: SCREEN, dispatcher_tx, text_component: None}
    }
}

impl Store for  ScreenSoloSpeedGameComponent {
    fn update(&mut self, action: Action) {
        match action {
            Action::OpeningScreen(screen) if screen == self.screen => {
                self.send(Action::InitializeSoloSpeedGame).unwrap();
                self.text_component = Some(TextWidgetComponent::new(self.dispatcher_tx.clone(), self.screen))
            }
            
            _ => {}
        }
        if let Some(text) = self.text_component.as_mut() {
            text.update_screen_member(action);
        }
    }
}
impl SendAction for ScreenSoloSpeedGameComponent {
    fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.dispatcher_tx.send(action)
    }
}
impl Widget for &mut ScreenSoloSpeedGameComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        if let Some(text) = self.text_component.as_mut() {
            text.widget.render(area, buf, &mut SettingsText::ThreeLine);
        };
        
    }
}