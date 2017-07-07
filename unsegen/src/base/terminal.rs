use ndarray::{
    Axis,
};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen};
use termion;
use base::{
    GraphemeCluster,
    Style,
    Window,
    WindowBuffer,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StyledGraphemeCluster {
    pub grapheme_cluster: GraphemeCluster,
    pub style: Style,
}

impl StyledGraphemeCluster {
    pub fn new(grapheme_cluster: GraphemeCluster, style: Style) -> Self {
        StyledGraphemeCluster {
            grapheme_cluster: grapheme_cluster,
            style: style,
        }
    }
}

impl Default for StyledGraphemeCluster {
    fn default() -> Self {
        Self::new(GraphemeCluster::space(), Style::default())
    }
}

pub struct Terminal<'a> {
    values: WindowBuffer,
    terminal: AlternateScreen<RawTerminal<::std::io::StdoutLock<'a>>>,
}

impl<'a> Terminal<'a> {
    pub fn new(stdout: ::std::io::StdoutLock<'a>) -> Self {
        use std::io::Write;
        let mut terminal = AlternateScreen::from(stdout.into_raw_mode().expect("raw terminal"));
        write!(terminal, "{}", termion::cursor::Hide).expect("write: hide cursor");
        Terminal {
            values: WindowBuffer::new(0, 0),
            terminal: terminal,
        }
    }

    pub fn create_root_window(&mut self) -> Window {
        let (x, y) = termion::terminal_size().expect("get terminal size");
        let x = x as u32;
        let y = y as u32;
        if x != self.values.as_window().get_width() || y != self.values.as_window().get_height() {
            self.values = WindowBuffer::new(x, y)
        } else {
            self.values.as_window().clear();
        }

        self.values.as_window()
    }

    pub fn present(&mut self) {
        use std::io::Write;
        //write!(self.terminal, "{}", termion::clear::All).expect("clear screen"); //Causes flickering and is unnecessary

        let mut current_style = Style::default();

        for (y, line) in self.values.storage().axis_iter(Axis(0)).enumerate() {
            write!(self.terminal, "{}", termion::cursor::Goto(1, (y+1) as u16)).expect("move cursor");
            let mut buffer = String::with_capacity(line.len());
            for c in line.iter() {
                //TODO style
                if c.style != current_style {
                    current_style.set_terminal_attributes(&mut self.terminal);
                    write!(self.terminal, "{}", buffer).expect("write buffer");
                    buffer.clear();
                    current_style = c.style;
                }
                let grapheme_cluster = match c.grapheme_cluster.as_str() {
                    c @ "\t" | c @ "\n" | c @ "\r" | c @ "\0" => panic!("Invalid grapheme cluster written to terminal: {:?}", c),
                    x => x,
                };
                buffer.push_str(grapheme_cluster);
            }
            current_style.set_terminal_attributes(&mut self.terminal);
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

#[cfg(test)]
pub mod test {
    use super::super::{
        CharMatrix,
        StyledGraphemeCluster,
        Style,
        Window,
        GraphemeCluster,
    };
    use ndarray::{
        Ix,
    };

    #[derive(PartialEq)]
    pub struct FakeTerminal {
        values: CharMatrix,
    }
    impl FakeTerminal {
        pub fn with_size((w, h): (Ix, Ix)) -> Self {
            FakeTerminal {
                values: CharMatrix::default((h, w)),
            }
        }

        pub fn size(&self) -> (Ix, Ix) {
            (self.values.dim().1, self.values.dim().0)
        }

        pub fn create_root_window(&mut self) -> Window {
            Window::new(self.values.view_mut(), Style::plain())
        }

        pub fn from_str((w, h): (Ix, Ix), description: &str) -> Result<Self, ::ndarray::ShapeError>{
            let mut tiles = Vec::<StyledGraphemeCluster>::new();
            for c in GraphemeCluster::all_from_str(description) {
                if c.as_str() == " " || c.as_str() == "\n" {
                    continue;
                }
                tiles.push(StyledGraphemeCluster::new(c, Style::plain()));
            }
            Ok(FakeTerminal {
                values: try!{::ndarray::Array2::from_shape_vec((h, w), tiles)},
            })
        }

        pub fn assert_looks_like(&self, string_description: &str) {
            assert_eq!(format!("{:?}", self), string_description);
        }
    }

    impl ::std::fmt::Debug for FakeTerminal {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            for r in 0..self.values.dim().0 {
                for c in 0..self.values.dim().1 {
                    let c = self.values.get((r, c)).expect("debug: in bounds");
                    try!{write!(f, "{}", c.grapheme_cluster.as_str())};
                }
                if r != self.values.dim().0-1 {
                    try!{write!(f, "|")};
                }
            }
            Ok(())
        }
    }
}
