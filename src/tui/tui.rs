use gdbmi;

use unsegen::{
    FileLineStorage,
    Key,
    ScrollBehavior,
    SeparatingStyle,
    VerticalLayout,
    Widget,
    Window,
    WriteBehavior,
};
use unsegen::widgets::{
    Pager,
    SyntectHighLighter,
};
use input::{
    InputEvent,
};
use syntect::highlighting::{
    Theme,
};
use syntect::parsing::{
    SyntaxSet,
};
use std::io;
use std::path::Path;
use gdbmi::output::{
    OutOfBandRecord,
    AsyncKind,
    AsyncClass,
    NamedValues,
};

use super::console::Console;
use super::pseudoterminal::PseudoTerminal;

pub struct Tui<'a> {
    console: Console,
    process_pty: PseudoTerminal,
    highlighting_theme: &'a Theme,
    file_viewer: Pager<FileLineStorage, SyntectHighLighter<'a>>,
    syntax_set: SyntaxSet,

    left_layout: VerticalLayout,
    right_layout: VerticalLayout,
}

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(io::Error),
    LineDoesNotExist(usize),
}

impl<'a> Tui<'a> {

    pub fn new(process_pty: ::pty::PTYInput, highlighting_theme: &'a Theme) -> Self {
        Tui {
            console: Console::new(),
            process_pty: PseudoTerminal::new(process_pty),
            highlighting_theme: highlighting_theme,
            file_viewer: Pager::new(),
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            left_layout: VerticalLayout::new(SeparatingStyle::Draw('=')),
            right_layout: VerticalLayout::new(SeparatingStyle::Draw('=')),
        }
    }

    pub fn show_in_file_viewer<P: AsRef<Path>>(&mut self, path: P, line: usize) -> Result<(), PagerShowError> {
        let need_to_reload = if let Some(ref content) = self.file_viewer.content {
            content.storage.get_file_path() != path.as_ref()
        } else {
            true
        };
        if need_to_reload {
            try!{self.load_in_file_viewer(path).map_err(|e| PagerShowError::CouldNotOpenFile(e))};
        }
        self.file_viewer.go_to_line(line).map_err(|_| PagerShowError::LineDoesNotExist(line))
    }

    pub fn load_in_file_viewer<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file_storage = try!{FileLineStorage::new(path.as_ref())};
        let syntax = self.syntax_set.find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see file storage")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.file_viewer.load(file_storage, SyntectHighLighter::new(syntax, self.highlighting_theme));
        Ok(())
    }

    fn handle_async_record(&mut self, kind: AsyncKind, class: AsyncClass, mut results: NamedValues) {
        match (kind, class) {
            (AsyncKind::Exec, AsyncClass::Stopped) => {
                self.console.add_message(format!("stopped: {:?}", results));
                if let Some(frame_object) = results.remove("frame") {
                    let mut frame = frame_object.unwrap_tuple_or_named_value_list();
                    let path = frame.remove("fullname").expect("fullname present").unwrap_const();
                    let line = frame.remove("line").expect("line present").unwrap_const().parse::<usize>().expect("parse usize") - 1; //TODO we probably want to treat the conversion line_number => buffer index somewhere else...
                    self.show_in_file_viewer(path, line).expect("loaded file at location indicated by gdb");
                }
            },
            (kind, class) => self.console.add_message(format!("unhandled async_record: [{:?}, {:?}] {:?}", kind, class, results)),
        }
    }

    pub fn add_out_of_band_record(&mut self, record: OutOfBandRecord) {
        match record {
            OutOfBandRecord::StreamRecord{ kind: _, data} => {
                use std::fmt::Write;
                write!(self.console, "{}", data).expect("Write message");
            },
            OutOfBandRecord::AsyncRecord{token: _, kind, class, results} => {
                self.handle_async_record(kind, class, results);
            },

        }
    }

    pub fn add_pty_input(&mut self, input: Vec<u8>) {
        self.process_pty.add_byte_input(input);
    }

    pub fn add_debug_message(&mut self, msg: &str) {
        self.console.add_message(format!("Debug: {}", msg));
    }

    pub fn draw(&mut self, window: Window) {
        use unsegen::{TextAttribute, Color, Style};
        let split_pos = window.get_width()/2-1;
        let (window_l, rest) = window.split_h(split_pos);

        let (mut separator, window_r) = rest.split_h(2);

        separator.set_default_format(TextAttribute::new(Color::green(), Color::blue(), Style::new().bold().italic().underline()));
        separator.fill('|');

        let mut left_widgets: Vec<&mut Widget> = vec![&mut self.console];
        self.left_layout.draw(window_l, &mut left_widgets);

        let mut right_widgets: Vec<&mut Widget> = vec![&mut self.file_viewer, &mut self.process_pty];
        self.right_layout.draw(window_r, &mut right_widgets);
    }

    pub fn event(&mut self, event: ::input::InputEvent, gdb: &mut gdbmi::GDB) { //TODO more console events
        match event {
            InputEvent::ConsoleEvent(event) => {
                self.console.event(event, gdb);
            },
            InputEvent::PseudoTerminalEvent(event) => {
                event.chain(WriteBehavior::new(&mut self.process_pty));
            },
            InputEvent::SourcePagerEvent(event) => {
                event.chain(ScrollBehavior::new(&mut self.file_viewer)
                            .forwards_on(Key::PageDown)
                            .backwards_on(Key::PageUp)
                            );
            },
            InputEvent::Quit => {
                unreachable!("quit should have been caught in main" )
            }, //TODO this is ugly
        }
    }
}
