extern crate libc;
extern crate nix;
extern crate unsegen;

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

pub use pty::{
    PTY,
    PTYInput,
    PTYOutput,
};

pub struct PseudoTerminal {
    //width: u32,
    //height: u32,
    pty: pty::PTYInput,
    display: LogViewer,
    //prompt_line: unsegen::widgets::PromptLine,
    //layout: unsegen::VerticalLayout,

    input_buffer: Vec<u8>,
}

impl PseudoTerminal {
    pub fn new(pty: pty::PTYInput) -> Self {
        PseudoTerminal {
            pty: pty,
            display: LogViewer::new(),
            //prompt_line: unsegen::widgets::PromptLine::with_prompt("".into()),
            //layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
            input_buffer: Vec::new(),
        }
    }

    pub fn add_byte_input(&mut self, mut bytes: Vec<u8>) {
        self.input_buffer.append(&mut bytes);

        //TODO: handle control sequences?
        if let Ok(string) = String::from_utf8(self.input_buffer.clone()) {
            use std::fmt::Write;
            self.display.storage.write_str(&string).expect("Write byte to terminal");
            self.input_buffer.clear();
        }
    }
}

impl Widget for PseudoTerminal {
    fn space_demand(&self) -> Demand2D {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.space_demand(widgets.into_iter())
        self.display.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.draw(window, &widgets)
        self.display.draw(window, hints);
    }
}

impl Writable for PseudoTerminal {
    fn write(&mut self, c: char) -> OperationResult {
        use std::io::Write;
        write!(self.pty, "{}", c).expect("Write key to terminal");
        Ok(())
    }
}
