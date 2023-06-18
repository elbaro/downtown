use ratatui::widgets::Widget;

pub struct Table {
    pub table: comfy_table::Table,
    pub highlight_row: usize,
}

impl Widget for Table {
    fn render(mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.table.set_width(area.width);

        let highlight_y = area.top() + self.highlight_row as u16 + 3;

        for (y, line) in (area.top()..area.bottom()).zip(self.table.lines()) {
            buf.set_stringn(
                area.x,
                y,
                line,
                area.width as usize,
                if y == highlight_y {
                    ratatui::style::Style::default()
                        .add_modifier(ratatui::style::Modifier::REVERSED)
                } else {
                    ratatui::style::Style::default()
                },
            );
        }
    }
}
