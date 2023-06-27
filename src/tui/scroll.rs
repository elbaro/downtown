pub struct Scroll {
    num_lines: usize,
    widget_height: usize,

    pub current_line: usize, // 1-based
    pub scroll_offset: usize,
}

impl Scroll {
    pub fn new(num_lines: usize, widget_height: usize) -> Self {
        Self {
            num_lines,
            widget_height,
            current_line: 1,
            scroll_offset: 1,
        }
    }
    pub fn up(&mut self, delta: usize) {
        if delta < self.current_line {
            self.current_line -= delta;
        } else {
            self.current_line = 1;
        }

        if self.scroll_offset > self.current_line {
            self.scroll_offset = self.current_line;
        }
    }
    pub fn down(&mut self, delta: usize) {
        if self.current_line + delta <= self.num_lines {
            self.current_line += delta;
        } else {
            self.current_line = self.num_lines;
        }
        if self.scroll_offset + self.widget_height <= self.current_line {
            self.scroll_offset = self.current_line - self.widget_height + 1;
        }
    }

    pub fn resize_height(&mut self, new_height: usize) {
        if self.current_line >= self.scroll_offset + new_height {
            self.scroll_offset = self.current_line - new_height + 1;
        }
        self.widget_height = new_height;
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        (self.scroll_offset)..(self.scroll_offset + self.widget_height).min(self.num_lines + 1)
    }
}
