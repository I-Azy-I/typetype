use ratatui::layout::{Constraint, Direction, Layout, Rect};
pub mod clock;
pub mod gauge;
pub mod list;
pub mod screens;
mod setting_screen;
pub mod text;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
pub fn centered_rect_with_length(size_x: u16, size_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.checked_sub(size_y)).unwrap_or_default() / 2),
            Constraint::Length(size_y),
            Constraint::Length((r.height.checked_sub(size_y)).unwrap_or_default() / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.checked_sub(size_x)).unwrap_or_default() / 2),
            Constraint::Length(size_x),
            Constraint::Length((r.width.checked_sub(size_x)).unwrap_or_default() / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
