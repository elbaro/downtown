pub struct Scroll {
    num_lines: usize,
    widget_height: usize,

    pub current_line: usize, // 0-based
    pub scroll_offset: usize,
}

impl Scroll {
    pub fn new(num_lines: usize, widget_height: usize) -> Self {
        Self {
            num_lines,
            widget_height,
            current_line: 0,
            scroll_offset: 0,
        }
    }
    pub fn up(&mut self) {
        if self.current_line == 0 {
            return;
        }

        if self.scroll_offset == self.current_line {
            self.scroll_offset -= 1;
        }
        self.current_line -= 1;
    }
    pub fn down(&mut self) {
        if self.current_line + 1 == self.num_lines {
            return;
        }

        self.current_line += 1;
        if self.scroll_offset + self.widget_height == self.current_line {
            self.scroll_offset += 1;
        }
    }

    pub fn resize_height(&mut self, new_height: usize) {
        if self.current_line >= self.scroll_offset + new_height {
            self.scroll_offset = self.current_line - new_height + 1;
        }
        self.widget_height = new_height;
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        (1 + self.scroll_offset)
            ..(1 + self.scroll_offset + self.widget_height).min(self.num_lines + 1)
    }
}
