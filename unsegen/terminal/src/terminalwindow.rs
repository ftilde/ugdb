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
    CursorStyle,
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

    /// OSC to set window title
    fn set_title(&mut self, _: &str) {
        //TODO: (Although this might not make sense to implement. Do we want to display a title?)
    }

    /// Set the cursor style
    fn set_cursor_style(&mut self, _: CursorStyle) {
        //TODO
    }

    /// A character to be displayed
    fn input(&mut self, c: char) {
        self.with_cursor(|cursor| {
            write!(cursor, "{}", c).unwrap();
        });
    }

    /*
    /// Set cursor to position
    fn goto(&mut self, Line, Column) {
        //TODO
    }

    /// Set cursor to specific row
    fn goto_line(&mut self, Line) {
        //TODO
    }

    /// Set cursor to specific column
    fn goto_col(&mut self, Column) {
        //TODO
    }

    /// Insert blank characters in current line starting from cursor
    fn insert_blank(&mut self, Column) {
        //TODO
    }

    /// Move cursor up `rows`
    fn move_up(&mut self, Line) {
        //TODO
    }

    /// Move cursor down `rows`
    fn move_down(&mut self, Line) {
        //TODO
    }

    /// Identify the terminal (should write back to the pty stream)
    ///
    /// TODO this should probably return an io::Result
    fn identify_terminal<W: io::Write>(&mut self, &mut W) {
        //TODO
    }

    /// Report device status
    fn device_status<W: io::Write>(&mut self, &mut W, usize) {
        //TODO
    }

    /// Move cursor forward `cols`
    fn move_forward(&mut self, Column) {
        //TODO
    }

    /// Move cursor backward `cols`
    fn move_backward(&mut self, Column) {
        //TODO
    }

    /// Move cursor down `rows` and set to column 1
    fn move_down_and_cr(&mut self, Line) {
        //TODO
    }

    /// Move cursor up `rows` and set to column 1
    fn move_up_and_cr(&mut self, Line) {
        //TODO
    }

    /// Put `count` tabs
    fn put_tab(&mut self, _count: i64) {
        //TODO
    }

    /// Backspace `count` characters
    fn backspace(&mut self) {
        //TODO
    }
    */

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

    /// Ring the bell
    fn bell(&mut self) {
        //omitted
    }

    /*
    /// Substitute char under cursor
    fn substitute(&mut self) {
        //TODO
    }

    /// Newline
    fn newline(&mut self) {
        //TODO
    }

    /// Set current position as a tabstop
    fn set_horizontal_tabstop(&mut self) {
        //TODO
    }

    /// Scroll up `rows` rows
    fn scroll_up(&mut self, Line) {
        //TODO
    }

    /// Scroll down `rows` rows
    fn scroll_down(&mut self, Line) {
        //TODO
    }

    /// Insert `count` blank lines
    fn insert_blank_lines(&mut self, Line) {
        //TODO
    }

    /// Delete `count` lines
    fn delete_lines(&mut self, Line) {
        //TODO
    }

    /// Erase `count` chars in current line following cursor
    ///
    /// Erase means resetting to the default state (default colors, no content,
    /// no mode flags)
    fn erase_chars(&mut self, Column) {
        //TODO
    }

    /// Delete `count` chars
    ///
    /// Deleting a character is like the delete key on the keyboard - everything
    /// to the right of the deleted things is shifted left.
    fn delete_chars(&mut self, Column) {
        //TODO
    }

    /// Move backward `count` tabs
    fn move_backward_tabs(&mut self, _count: i64) {
        //TODO
    }

    /// Move forward `count` tabs
    fn move_forward_tabs(&mut self, _count: i64) {
        //TODO
    }

    /// Save current cursor position
    fn save_cursor_position(&mut self) {
        //TODO
    }

    /// Restore cursor position
    fn restore_cursor_position(&mut self) {
        //TODO
    }

    /// Clear current line
    fn clear_line(&mut self, _mode: LineClearMode) {
        //TODO
    }

    /// Clear screen
    fn clear_screen(&mut self, _mode: ClearMode) {
        //TODO
    }

    /// Clear tab stops
    fn clear_tabs(&mut self, _mode: TabulationClearMode) {
        //TODO
    }

    /// Reset terminal state
    fn reset_state(&mut self) {
        //TODO
    }

    /// Reverse Index
    ///
    /// Move the active position to the same horizontal position on the
    /// preceding line. If the active position is at the top margin, a scroll
    /// down is performed
    fn reverse_index(&mut self) {
        //TODO
    }

    /// set a terminal attribute
    fn terminal_attribute(&mut self, _attr: Attr) {
        //TODO
    }

    /// Set mode
    fn set_mode(&mut self, _mode: Mode) {
        //TODO
    }

    /// Unset mode
    fn unset_mode(&mut self, Mode) {
        //TODO
    }

    /// DECSTBM - Set the terminal scrolling region
    fn set_scrolling_region(&mut self, Range<Line>) {
        //TODO
    }

    /// DECKPAM - Set keypad to applications mode (ESCape instead of digits)
    fn set_keypad_application_mode(&mut self) {
        //TODO
    }

    /// DECKPNM - Set keypad to numeric mode (digits intead of ESCape seq)
    fn unset_keypad_application_mode(&mut self) {
        //TODO
    }

    /// Set one of the graphic character sets, G0 to G3, as the active charset.
    ///
    /// 'Invoke' one of G0 to G3 in the GL area. Also refered to as shift in,
    /// shift out and locking shift depending on the set being activated
    fn set_active_charset(&mut self, CharsetIndex) {
        //TODO
    }

    /// Assign a graphic character set to G0, G1, G2 or G3
    ///
    /// 'Designate' a graphic character set as one of G0 to G3, so that it can
    /// later be 'invoked' by `set_active_charset`
    fn configure_charset(&mut self, CharsetIndex, StandardCharset) {
        //TODO
    }

    /// Set an indexed color value
    fn set_color(&mut self, usize, Rgb) {
        //TODO
    }

    /// Run the dectest routine
    fn dectest(&mut self) {
        //TODO
    }
    */
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
