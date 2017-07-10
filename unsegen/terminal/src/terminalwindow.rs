use unsegen::base::{
    Cursor,
    CursorState,
    CursorTarget,
    Style,
    StyledGraphemeCluster,
    Window,
    WrappingMode,
    UNBOUNDED_EXTENT,
};
use unsegen::widget::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use unsegen::input::{
    Scrollable,
    OperationResult,
};
use ansi::{
    Handler,
    TermInfo,
};

use std::fmt::Write;
use index;

#[derive(Clone)]
struct Line {
    content: Vec<StyledGraphemeCluster>,
}

impl Line {
    fn empty() -> Self {
        Line {
            content: Vec::new(),
        }
    }

    fn length(&self) -> u32 {
        self.content.len() as u32
    }

    fn get_grapheme_cluster_mut(&mut self, x: u32) -> Option<&mut StyledGraphemeCluster> {
        // Grow horizontally to desired position
        let missing_elements = (x as usize+ 1).checked_sub(self.content.len()).unwrap_or(0);
        self.content.extend(::std::iter::repeat(StyledGraphemeCluster::default()).take(missing_elements));

        let element = self.content.get_mut(x as usize).expect("element existent assured previously");
        Some(element)
    }
}

struct LineBuffer {
    lines: Vec<Line>,
    window_width: u32,
    default_style: Style,
}
impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer {
            lines: Vec::new(),
            window_width: 0,
            default_style: Style::default(),
        }
    }

    pub fn set_window_width(&mut self, w: u32) {
        self.window_width = w;
    }
}

impl CursorTarget for LineBuffer {
    fn get_width(&self) -> u32 {
        UNBOUNDED_EXTENT
    }
    fn get_soft_width(&self) -> u32 {
        self.window_width
    }
    fn get_height(&self) -> u32 {
        UNBOUNDED_EXTENT
    }
    fn get_grapheme_cluster_mut(&mut self, x: u32, y: u32) -> Option<&mut StyledGraphemeCluster> {
        // Grow vertically to desired position
        let missing_elements = (y as usize + 1).checked_sub(self.lines.len()).unwrap_or(0);
        self.lines.extend(::std::iter::repeat(Line::empty()).take(missing_elements));

        let line = self.lines.get_mut(y as usize).expect("line existence assured previously");

        line.get_grapheme_cluster_mut(x)
    }
    fn get_default_style(&self) -> &Style {
        &self.default_style
    }
}

pub struct TerminalWindow {
    window_width: u32,
    window_height: u32,
    buffer: LineBuffer,
    cursor_state: CursorState,
    //input_buffer: Vec<u8>,
    scrollback_position: Option<usize>,
    scroll_step: usize,
}

impl TerminalWindow {
    pub fn new() -> Self {
        TerminalWindow  {
            window_width: 0,
            window_height: 0,
            buffer: LineBuffer::new(),
            cursor_state: CursorState::default(),
            //input_buffer: Vec::new(),
            scrollback_position: None,
            scroll_step: 1,
        }
    }

    fn current_scrollback_line(&self) -> usize {
        self.scrollback_position.unwrap_or(self.buffer.lines.len().checked_sub(1).unwrap_or(0))
    }

    fn set_width(&mut self, w: u32) {
        self.window_width = w;
        self.buffer.set_window_width(w);
    }

    fn set_height(&mut self, h: u32) {
        self.window_height = h;
    }

    fn with_cursor<F: FnOnce(&mut Cursor<LineBuffer>)>(&mut self, f: F) {
        let mut state = CursorState::default();
        ::std::mem::swap(&mut state, &mut self.cursor_state);
        let mut cursor = Cursor::with_state(&mut self.buffer, state);
        f(&mut cursor);
        self.cursor_state = cursor.into_state();
    }
    /*
    pub fn display_byte(&mut self, byte: u8) {
        self.input_buffer.push(byte);

        if let Ok(string) = String::from_utf8(self.input_buffer.clone()) {
            use std::fmt::Write;
            self.display.storage.write_str(&string).expect("Write byte to terminal");
            self.input_buffer.clear();
        }
    }
    */
}

impl Widget for TerminalWindow {
    fn space_demand(&self) -> Demand2D {
        // at_least => We can grow if there is space
        Demand2D {
            width: Demand::at_least(self.cols().0 as u32),
            height: Demand::at_least(self.lines().0 as u32),
        }
    }

    fn draw(&mut self, mut window: Window, _: RenderingHints) {
        let height = window.get_height();
        let width = window.get_width();

        self.set_width(width);
        self.set_height(height);

        if height == 0 || width == 0 || self.buffer.lines.is_empty() {
            return;
        }

        let y_start = height as usize - 1;
        let mut cursor = Cursor::new(&mut window)
            .position(0, y_start as i32)
            .wrapping_mode(WrappingMode::Wrap);
        let end_line = self.current_scrollback_line();
        let start_line = end_line.checked_sub(height as usize).unwrap_or(0);
        for line in self.buffer.lines[start_line..(end_line+1)].iter().rev() {
            let num_auto_wraps = (line.length().checked_sub(1).unwrap_or(0) / width) as i32;
            cursor.move_by(0, -num_auto_wraps);
            cursor.write_preformatted(line.content.as_slice());
            cursor.carriage_return();
            cursor.move_by(0, -num_auto_wraps-1);
        }
    }
}

impl Handler for TerminalWindow {

    /// A character to be displayed
    fn input(&mut self, c: char) {
        self.with_cursor(|cursor| {
            write!(cursor, "{}", c).unwrap();
        });
    }

    /// Carriage return
    fn carriage_return(&mut self) {
        self.with_cursor(|cursor| {
            cursor.carriage_return()
        });
    }

    /// Linefeed
    fn linefeed(&mut self) {
        self.with_cursor(|cursor| {
            cursor.wrap_line()
        });
    }
}

impl TermInfo for TerminalWindow {
    fn lines(&self) -> index::Line {
        index::Line(self.window_height as usize) //TODO: is this even correct? do we want 'unbounded'?
    }
    fn cols(&self) -> index::Column {
        index::Column(self.window_width as usize) //TODO: see above
    }
}

impl Scrollable for TerminalWindow {
    fn scroll_forwards(&mut self) -> OperationResult {
        let current = self.current_scrollback_line();
        let candidate = current + self.scroll_step;
        self.scrollback_position = if candidate < self.buffer.lines.len() {
            Some(candidate)
        } else {
            None
        };
        if self.scrollback_position.is_some() {
            Ok(())
        } else {
            Err(())
        }
    }
    fn scroll_backwards(&mut self) -> OperationResult {
        let current = self.current_scrollback_line();
        let op_res = if current != 0 {
            Ok(())
        } else {
            Err(())
        };
        self.scrollback_position = Some(current.checked_sub(self.scroll_step).unwrap_or(0));
        op_res
    }
}

#[cfg(test)]
impl TerminalWindow {
    fn write(&mut self, s: &str) {
        for c in s.chars() {
            self.input(c);
        }
    }
}
#[cfg(test)]
mod test {
    use unsegen::base::terminal::test::FakeTerminal;
    use super::*;
    use unsegen::base::{
        GraphemeCluster,
    };

    fn test_terminal_window<F: Fn(&mut TerminalWindow)>(window_dim: (u32, u32), after: &str, action: F) {
        let mut term = FakeTerminal::with_size(window_dim);
        {
            let mut window = term.create_root_window();
            window.fill(GraphemeCluster::try_from('_').unwrap());
            let mut tw = TerminalWindow::new();
            action(&mut tw);
            tw.draw(window, RenderingHints::default());
        }
        term.assert_looks_like(after);
    }
    #[test]
    fn test_terminal_window_simple() {
        test_terminal_window((5, 1), "_____", |w| w.write(""));
        test_terminal_window((5, 1), "t____", |w| w.write("t"));
        test_terminal_window((5, 1), "te___", |w| w.write("te"));
        test_terminal_window((5, 1), "tes__", |w| w.write("tes"));
        test_terminal_window((5, 1), "test_", |w| w.write("test"));
        test_terminal_window((5, 1), "testy", |w| w.write("testy"));
        test_terminal_window((5, 1), "o____", |w| w.write("testyo"));

        test_terminal_window((2, 2), "te|st", |w| w.write("te\nst"));
    }
}
