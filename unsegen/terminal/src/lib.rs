extern crate libc;
extern crate nix;
extern crate unsegen;
extern crate vte;

use unsegen::base::{
    Window,
};
use unsegen::input::{
    OperationResult,
    Writable,
};
use unsegen::widget::{
    Demand2D,
    RenderingHints,
    Widget,
};
use unsegen::widget::widgets::{
    LogViewer,
};
mod pty;
#[allow(dead_code)]
mod ansi;
#[allow(dead_code)]
mod index;

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
    Handler,
    TermInfo,
};

use index::{
    Line,
    Column,
};

use std::fs::File;
use std::thread;
use ::std::fmt::Write;

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

// Resembles the terminal buffer itself.
struct TerminalBuffer {
    display: LogViewer,
    //input_buffer: Vec<u8>,
}

impl TerminalBuffer {
    fn new() -> Self {
        TerminalBuffer  {
            display: LogViewer::new(),
            //input_buffer: Vec::new(),
        }
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

impl Handler for TerminalBuffer {

    /// A character to be displayed
    fn input(&mut self, c: char) {
        write!(self.display.storage, "{}", c).unwrap();
    }

    /// Carriage return
    fn carriage_return(&mut self) {
        write!(self.display.storage, "\r").unwrap();
    }

    /// Linefeed
    fn linefeed(&mut self) {
        write!(self.display.storage, "\n").unwrap();
    }
}

impl TermInfo for TerminalBuffer {
    fn lines(&self) -> Line {
        Line(0)
    }
    fn cols(&self) -> Column {
        Column(0)
    }
}

pub struct Terminal {
    buffer: TerminalBuffer,
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
            buffer: TerminalBuffer::new(),
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
            self.ansi_processor.advance(&mut self.buffer, *byte, &mut self.master_input_sink);
        }
    }

    pub fn get_slave_name(&self) -> &OsStr {
        self.slave_name.as_ref()
    }
}

impl Widget for Terminal {
    fn space_demand(&self) -> Demand2D {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.space_demand(widgets.into_iter())
        self.buffer.display.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.draw(window, &widgets)
        self.buffer.display.draw(window, hints);
    }
}

impl Writable for Terminal {
    fn write(&mut self, c: char) -> OperationResult {
        use std::io::Write;
        write!(self.master_input_sink, "{}", c).expect("Write key to terminal");
        Ok(())
    }
}
