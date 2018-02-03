use ndarray::{
    Axis,
};
use termion::raw::{IntoRawMode, RawTerminal};
use termion;
use base::{
    Style,
    Window,
    WindowBuffer,
    Width,
    Height,
};
use std::io;
use std::io::{
    Write,
    StdoutLock,
};

use nix::sys::signal::{
    SIGCONT,
    SIGTSTP,
    SigSet,
    SigmaskHow,
    kill,
    pthread_sigmask,
};
use nix::unistd::getpgrp;

pub struct Terminal<'a> {
    values: WindowBuffer,
    terminal: RawTerminal<StdoutLock<'a>>,
}

impl<'a> Terminal<'a> {
    pub fn new(stdout: StdoutLock<'a>) -> Self {
        let mut term = Terminal {
            values: WindowBuffer::new(Width::new(0).unwrap(), Height::new(0).unwrap()),
            terminal: stdout.into_raw_mode().expect("raw terminal"),
        };
        term.setup_terminal().expect("Setup terminal");
        term
    }

    /// This method is intended to be called when the process received a SIGTSTP.
    ///
    /// The terminal state is restored, and the process is actually stopped within this function.
    /// When the process then receives a SIGCONT it sets up the terminal state as expected again
    /// and returns from the function.
    ///
    /// The usual way to deal with SIGTSTP (and signals in general) is to block them and `waidpid`
    /// for them in a separate thread which sends the events into some fifo. The fifo can be polled
    /// in an event loop. Then, if in the main event loop a SIGTSTP turn up, *this* function should
    /// be called.
    pub fn handle_sigtstp(&mut self) {
        self.restore_terminal().expect("Restore terminal");

        let mut stop_and_cont = SigSet::empty();
        stop_and_cont.add(SIGCONT);
        stop_and_cont.add(SIGTSTP);

        // 1. Unblock SIGTSTP and SIGCONT, so that we actually stop when we receive another SIGTSTP
        pthread_sigmask(SigmaskHow::SIG_UNBLOCK, Some(&stop_and_cont), None).expect("Unblock signals");

        // 2. Reissue SIGTSTP (this time to whole the process group!)...
        kill(-getpgrp(), SIGTSTP).expect("SIGTSTP self");
        // ... and stop!
        // Now we are waiting for a SIGCONT.

        // 3. Once we receive a SIGCONT we block SIGTSTP and SIGCONT again and resume.
        pthread_sigmask(SigmaskHow::SIG_BLOCK, Some(&stop_and_cont), None).expect("Block signals");

        self.setup_terminal().expect("Setup terminal");
    }

    fn setup_terminal(&mut self) -> io::Result<()> {
        write!(self.terminal, "{}{}", termion::screen::ToAlternateScreen, termion::cursor::Hide)?;
        self.terminal.flush()?;
        Ok(())
    }

    fn restore_terminal(&mut self) -> io::Result<()> {
        write!(self.terminal, "{}{}", termion::screen::ToMainScreen, termion::cursor::Show)?;
        self.terminal.flush()?;
        Ok(())
    }

    pub fn create_root_window(&mut self) -> Window {
        let (x, y) = termion::terminal_size().expect("get terminal size");
        let x = Width::new(x as i32).unwrap();
        let y = Height::new(y as i32).unwrap();
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
        let _ = self.restore_terminal();
    }
}

pub mod test {
    use super::super::{
        Height,
        Style,
        StyledGraphemeCluster,
        Window,
        WindowBuffer,
        Width,
        GraphemeCluster,
    };

    #[derive(PartialEq)]
    pub struct FakeTerminal {
        values: WindowBuffer,
    }
    impl FakeTerminal {
        pub fn with_size((w, h): (u32, u32)) -> Self {
            FakeTerminal {
                values: WindowBuffer::new(Width::new(w as i32).unwrap(), Height::new(h as i32).unwrap())
            }
        }

        /*
        pub fn size(&self) -> (Ix, Ix) {
            (self.values.dim().1, self.values.dim().0)
        }
        */

        pub fn create_root_window(&mut self) -> Window {
            self.values.as_window()
        }

        pub fn from_str((w, h): (u32, u32), description: &str) -> Result<Self, ::ndarray::ShapeError>{
            let mut tiles = Vec::<StyledGraphemeCluster>::new();
            for c in GraphemeCluster::all_from_str(description) {
                if c.as_str() == " " || c.as_str() == "\n" {
                    continue;
                }
                tiles.push(StyledGraphemeCluster::new(c, Style::plain()));
            }
            Ok(FakeTerminal {
                values: WindowBuffer::from_storage(try!{::ndarray::Array2::from_shape_vec((h as usize, w as usize), tiles)}),
            })
        }

        pub fn assert_looks_like(&self, string_description: &str) {
            assert_eq!(format!("{:?}", self), string_description);
        }
    }

    impl ::std::fmt::Debug for FakeTerminal {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            let raw_values = self.values.storage();
            for r in 0..raw_values.dim().0 {
                for c in 0..raw_values.dim().1 {
                    let c = raw_values.get((r, c)).expect("debug: in bounds");
                    try!{write!(f, "{}", c.grapheme_cluster.as_str())};
                }
                if r != raw_values.dim().0-1 {
                    try!{write!(f, "|")};
                }
            }
            Ok(())
        }
    }
}
