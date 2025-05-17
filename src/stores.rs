
use crate::action::Action;

pub trait Store {
    fn update(&mut self, action: Action);
}