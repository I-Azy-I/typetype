use std::time::Duration;

use ratatui::{text::Line, widgets::Widget};



#[derive(Debug, Default)]
pub struct ClockWidget {
    duration: Duration
}
impl ClockWidget {
    pub fn new(duration: Duration) -> Self {
        ClockWidget {duration}
    }
}
impl Widget for ClockWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        let seconds = self.duration.as_secs() % 60;
        let minutes = (self.duration.as_secs() - seconds) / 60;
        let time_str = format!("{minutes:02}:{seconds:02}");
        let line = Line::from(time_str);
        buf.set_line(area.x, area.y, &line, 5);
    }
}