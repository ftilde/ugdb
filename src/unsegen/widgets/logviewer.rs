use super::super::{
    Cursor,
    Demand,
    Event,
    Widget,
    Window,
    WrappingDirection,
    WrappingMode,
};
use super::super::input::{
    Scrollable,
};

pub struct LogViewer {
    lines: Vec<String>,
} //TODO support incomplete lines

impl Widget for LogViewer {
    fn space_demand(&self) -> (Demand, Demand) {
        return (Demand::at_least(1), Demand::at_least(1));
    }
    fn draw(&mut self, mut window: Window) {
        let y_start = window.get_height() - 1;
        let mut cursor = Cursor::new(&mut window)
            .position(0, y_start as i32)
            .wrapping_direction(WrappingDirection::Up)
            .wrapping_mode(WrappingMode::Wrap);
        for line in self.lines.iter().rev() {
            cursor.writeln(&line);
        }
    }
}

impl LogViewer {
    pub fn new() -> Self {
        LogViewer {
            lines: Vec::new(),
        }
    }

    pub fn active_line_mut(&mut self) -> &mut String {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        return self.lines.last_mut().expect("last line");
    }
}

impl ::std::fmt::Write for LogViewer {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        let mut s = s.to_owned();

        while let Some(newline_offset) = s.find('\n') {
            let line: String = s.drain(..newline_offset).collect();
            s.pop(); //Remove the \n
            self.active_line_mut().push_str(&line);
            self.lines.push(String::new());
        }
        self.active_line_mut().push_str(&s);
        Ok(())
    }
}
impl Scrollable for LogViewer {
    fn scroll_forwards(&mut self) {
        unimplemented!();
    }
    fn scroll_backwards(&mut self) {
        unimplemented!();
    }
}
