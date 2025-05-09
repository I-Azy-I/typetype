use ratatui::{self, widgets::Widget};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::App};

pub trait Store {
    fn update(&mut self, action: Action);
}