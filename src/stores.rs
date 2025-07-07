use crate::{action::Action, ui::screens::IsScreen};

pub trait Store {
    fn update(&mut self, action: Action);
}
