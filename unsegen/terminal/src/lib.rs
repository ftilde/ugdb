extern crate libc;
extern crate nix;
extern crate unsegen;
extern crate vte;
mod pty;
#[allow(dead_code)]
mod ansi;
#[allow(dead_code)]
mod index;
mod terminalwindow;

use unsegen::base::{
    Window,
};
use unsegen::input::{
    Behavior,
    Event,
    Key,
    Input,
    OperationResult,
    Scrollable,
    Writable,
};
use unsegen::widget::{
    Demand2D,
    RenderingHints,
    Widget,
};
use pty::{
    PTY,
    PTYInput,
    PTYOutput,
};
use std::ffi::{
    OsStr,
    OsString,
};
use ansi::{
    Processor,
};

use terminalwindow::TerminalWindow;

use std::fs::File;
use std::thread;

fn read_slave_input_loop<S: SlaveInputSink>(sink: S, mut reader: PTYOutput) {
    use ::std::io::Read;

    let mut buffer = [0; 1024];
    while let Ok(n) = reader.read(&mut buffer) {
        let mut bytes = vec![0; n];
        bytes.copy_from_slice(&mut buffer[..n]);
        sink.send(bytes.into_boxed_slice());
    }
}
// Sink that receives all (byte) input that is send from a slave terminal
pub trait SlaveInputSink : std::marker::Send {
    fn send(&self, data: Box<[u8]>);
}

// Passes all inputs through to the modelled terminal
pub struct PassthroughBehavior<'a>{
    term: &'a mut Terminal
}

impl<'a> PassthroughBehavior<'a> {
    pub fn new(term: &'a mut Terminal) -> Self {
        PassthroughBehavior {
            term: term,
        }
    }
}

impl<'a> Behavior for PassthroughBehavior<'a> {
    fn input(self, i: Input) -> Option<Input> {
        self.term.process_input(i);
        None
    }
}


pub struct Terminal {
    terminal_window: TerminalWindow,
    //slave_input_thread: thread::Thread,
    master_input_sink: PTYInput,

    //Hack used to keep the slave device open as long as the master exists. This may not be a good idea, we will see...
    _slave_handle: File,
    slave_name: OsString,

    ansi_processor: Processor,
}

impl Terminal {
    pub fn new<S: SlaveInputSink + 'static>(input_sink: S) -> Self {
        let process_pty = PTY::open().expect("Could not create pty.");

        let ptsname = process_pty.name().to_owned();

        let (pty_input, pty_output) = process_pty.split_io();

        /*let slave_input_thread =*/ thread::Builder::new().name("slave input thread".to_owned()).spawn(move || {
            read_slave_input_loop(input_sink, pty_output);
        }).expect("Spawn slave input thread");

        // Hack:
        // Open slave terminal, so that it does not get destroyed when a gdb process opens it and
        // closes it afterwards.
        let mut pts = std::fs::OpenOptions::new().write(true).read(true).open(&ptsname).expect("pts file");
        use std::io::Write;
        write!(pts, "").expect("initial write to pts");

        Terminal {
            terminal_window: TerminalWindow::new(),
            master_input_sink: pty_input,
            //slave_input_thread: slave_input_thread,
            _slave_handle: pts,
            slave_name: ptsname,
            ansi_processor: Processor::new(),
        }
    }

    //TODO: do we need to distinguish between input from user and from slave?
    pub fn add_byte_input(&mut self, bytes: Box<[u8]>) {
        for byte in bytes.iter() {
            self.ansi_processor.advance(&mut self.terminal_window, *byte, &mut self.master_input_sink);
        }
    }

    pub fn get_slave_name(&self) -> &OsStr {
        self.slave_name.as_ref()
    }

    pub fn process_input(&mut self, i: Input) {
        //TODO: implement more keys. Actually, we probably want to pass on the raw input bytes from
        //termion to the sink. This requires work on the termion side...
        use std::io::Write;
        match i.event {
            Event::Key(Key::Char(c)) => {
                write!(self.master_input_sink, "{}", c)
            },
            Event::Key(Key::Backspace) => {
                write!(self.master_input_sink, "\x7f")
            },
            _ => { Ok(()) }
        }.expect("Write to terminal");
    }
}

impl Widget for Terminal {
    fn space_demand(&self) -> Demand2D {
        self.terminal_window.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        self.terminal_window.draw(window, hints);
    }
}

impl Writable for Terminal {
    fn write(&mut self, c: char) -> OperationResult {
        use std::io::Write;
        write!(self.master_input_sink, "{}", c).expect("Write key to terminal");
        Ok(())
    }
}

impl Scrollable for Terminal {
    fn scroll_forwards(&mut self) -> OperationResult {
        self.terminal_window.scroll_forwards()
    }
    fn scroll_backwards(&mut self) -> OperationResult {
        self.terminal_window.scroll_backwards()
    }
}
