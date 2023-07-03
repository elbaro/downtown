use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub header_block: Rect,
    pub code: Rect,
    pub code_block: Rect,
}

pub struct LayoutToggle {
    _header_toggle: bool,
}

pub fn layout(size: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(size)
}
