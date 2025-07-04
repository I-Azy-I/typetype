use tokio::sync::mpsc::error::SendError;

use crate::action::Action;

pub trait SendAction {
    fn send(&self, action: Action) -> Result<(), SendError<Action>>;
}
