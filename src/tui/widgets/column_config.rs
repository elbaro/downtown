use ratatui::{
    style::{Color, Style},
    widgets::StatefulWidget,
};

pub struct ColumnConfigState {
    latency_frequency: bool,
    garbage_collection: bool,
    memory_allocation: bool,
    network: bool,
    disk: bool,
}

pub struct ColumnConfig {}

impl StatefulWidget for ColumnConfig {
    type State = ColumnConfigState;

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        fn render_item(
            x: &mut u16,
            y: u16,
            buf: &mut ratatui::buffer::Buffer,
            state: bool,
            text: &str,
            is_selected: bool,
        ) {
            // "[v] aaa"
            let s = if state {
                format!("[v] {text}")
            } else {
                format!("[ ] {text}")
            };
            let style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            buf.set_string(*x, y, &s, style);
            *x += s.len() as u16;
        }
        let mut x = area.x;
        render_item(
            &mut x,
            area.y + 1,
            buf,
            state.latency_frequency,
            "Latency/Frequency",
            false,
        );
        render_item(
            &mut x,
            area.y + 1,
            buf,
            state.garbage_collection,
            "GC",
            false,
        );
        render_item(
            &mut x,
            area.y + 1,
            buf,
            state.memory_allocation,
            "Allocation",
            false,
        );
        render_item(&mut x, area.y + 1, buf, state.network, "Net", false);
        render_item(&mut x, area.y + 1, buf, state.disk, "Disk", false);
    }
}
