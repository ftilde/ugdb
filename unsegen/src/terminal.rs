use ndarray::{
    Array,
    Axis,
    Ix,
    Ix2,
};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen};
use termion;
use super::{
    TextAttribute,
    Window,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FormattedChar {
    // Invariant: the contents of graphemeCluster is always valid utf8!
    grapheme_cluster: ::smallvec::SmallVec<[u8; 16]>,
    format: TextAttribute,
}

impl FormattedChar {
    pub fn new(grapheme_cluster: &str, format: TextAttribute) -> Self {
        let mut vec = ::smallvec::SmallVec::<[u8; 16]>::new();
        for byte in grapheme_cluster.bytes() {
            vec.push(byte);
        }
        FormattedChar {
            grapheme_cluster: vec,
            format: format,
        }
    }

    pub fn grapheme_cluster_as_str<'a>(&'a self) -> &'a str {
        // This is safe because graphemeCluster is always valid utf8.
        unsafe {
            ::std::str::from_utf8_unchecked(&self.grapheme_cluster)
        }
    }
}

impl Default for FormattedChar {
    fn default() -> Self {
        Self::new(" ", TextAttribute::default())
    }
}

pub type CharMatrix = Array<FormattedChar, Ix2>;
pub struct Terminal<'a> {
    values: CharMatrix,
    terminal: AlternateScreen<RawTerminal<::std::io::StdoutLock<'a>>>,
}

impl<'a> Terminal<'a> {
    pub fn new(stdout: ::std::io::StdoutLock<'a>) -> Self {
        use std::io::Write;
        let mut terminal = AlternateScreen::from(stdout.into_raw_mode().expect("raw terminal"));
        write!(terminal, "{}", termion::cursor::Hide).expect("write: hide cursor");
        Terminal {
            values: CharMatrix::default(Ix2(0,0)),
            terminal: terminal
        }
    }

    pub fn create_root_window(&mut self, default_format: TextAttribute) -> Window {
        let (x, y) = termion::terminal_size().expect("get terminal size");
        let dim = Ix2(y as Ix, x as Ix);
        //if dim != self.values.dim() {
        self.values = CharMatrix::default(dim);
        //}

        Window::new(self.values.view_mut(), default_format)
    }

    pub fn present(&mut self) {
        use std::io::Write;
        //write!(self.terminal, "{}", termion::clear::All).expect("clear screen"); //Causes flickering and is unnecessary

        let mut current_format = TextAttribute::default();

        for (y, line) in self.values.axis_iter(Axis(0)).enumerate() {
            write!(self.terminal, "{}", termion::cursor::Goto(1, (y+1) as u16)).expect("move cursor");
            let mut buffer = String::with_capacity(line.len());
            for c in line.iter() {
                //TODO style
                if c.format != current_format {
                    current_format.set_terminal_attributes(&mut self.terminal);
                    write!(self.terminal, "{}", buffer).expect("write buffer");
                    buffer.clear();
                    current_format = c.format;
                }
                let grapheme_cluster = match c.grapheme_cluster_as_str() {
                    "\t" | "\n" | "\r" | "\0" => panic!("Invalid grapheme cluster written to terminal"),
                    x => x,
                };
                buffer.push_str(grapheme_cluster);
            }
            current_format.set_terminal_attributes(&mut self.terminal);
            write!(self.terminal, "{}", buffer).expect("write leftover buffer contents");
        }
        self.terminal.flush().expect("flush terminal");
    }
}

impl<'a> Drop for Terminal<'a> {
    fn drop(&mut self) {
        use std::io::Write;
        write!(self.terminal, "{}", termion::cursor::Show).expect("show cursor");
    }
}
