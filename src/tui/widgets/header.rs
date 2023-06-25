use ratatui::{
    style::Style,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

pub struct HeaderState {
    pub pid: String,
    pub python_bin: String,
    pub code_path: String,
}
pub struct Header;

impl StatefulWidget for Header {
    type State = HeaderState;

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        Block::default().borders(Borders::ALL).render(area, buf);
        buf.set_string(
            area.x + 2,
            area.y + 1,
            format!("pid: {} / python binary: {}", state.pid, state.python_bin),
            Style::default(),
        );
        buf.set_string(
            area.x + 2,
            area.y + 2,
            format!("code: {}", state.code_path),
            Style::default(),
        );
    }
}
