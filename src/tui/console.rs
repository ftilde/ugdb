use unsegen;
use gdbmi;

use unsegen::{
    VerticalLayout,
    Widget,
    Demand,
    Window,
    Event,
    Input,
    Key,
    EditBehavior,
    ScrollBehavior,
};
use unsegen::widgets::{
    LogViewer,
    PromptLine,
};


pub struct Console {
    text_area: LogViewer,
    prompt_line: PromptLine,
    layout: VerticalLayout,
}

impl Console {
    pub fn new() -> Self {
        Console {
            text_area: LogViewer::new(),
            prompt_line: PromptLine::with_prompt("(gdb) ".into()),
            layout: VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
        }
    }

    pub fn add_message(&mut self, msg: String) {
        use std::fmt::Write;
        write!(self.text_area.storage, " -=- {}\n", msg).expect("Write message");
    }

    pub fn event(&mut self, input: unsegen::Input, gdb: &mut gdbmi::GDB) { //TODO more console events
        if input.event == Event::Key(Key::Char('\n')) {
            let line = if self.prompt_line.active_line().is_empty() {
                self.prompt_line.previous_line(1).unwrap_or("").to_owned()
            } else {
                self.prompt_line.finish_line().to_owned()
            };
            match line.as_ref() {
                "!stop" => {
                    gdb.interrupt_execution().expect("interrupted gdb");

                    // This does not always seem to unblock gdb, but only hang it
                    //use gdbmi::input::MiCommand;
                    //gdb.execute(&MiCommand::exec_interrupt()).expect("Interrupt ");
                },
                // Gdb commands
                _ => {
                    self.add_message(format!("(gdb) {}", line));
                    match gdb.execute(&gdbmi::input::MiCommand::cli_exec(line)) {
                        Ok(result) => {
                            self.add_message(format!("Result: {:?}", result));
                        },
                        Err(gdbmi::ExecuteError::Quit) => { self.add_message(format!("quit")); },
                        Err(gdbmi::ExecuteError::Busy) => { self.add_message(format!("GDB is running!")); },
                        //Err(err) => { panic!("Unknown error {:?}", err) },
                    }
                },
            }
        } else {
            let _ = input.chain(
                    |i: Input| if let (&Event::Key(Key::Ctrl('c')), true) = (&i.event, self.prompt_line.line.get().is_empty()) {
                        gdb.interrupt_execution().expect("interrupted gdb");
                        None
                    } else {
                        Some(i)
                    }
                    )
                .chain(
                    EditBehavior::new(&mut self.prompt_line)
                        .left_on(Key::Left)
                        .right_on(Key::Right)
                        .up_on(Key::Up)
                        .down_on(Key::Down)
                        .delete_symbol_on(Key::Delete)
                        .remove_symbol_on(Key::Backspace)
                        .clear_on(Key::Ctrl('c'))
                    )
                .chain(
                    ScrollBehavior::new(&mut self.text_area)
                        .forwards_on(Key::PageDown)
                        .backwards_on(Key::PageUp)
                    );
        }
    }
}

impl Widget for Console {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = vec![&self.text_area, &self.prompt_line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        let mut widgets: Vec<&mut Widget> = vec![&mut self.text_area, &mut self.prompt_line];
        self.layout.draw(window, &mut widgets)
    }
}

impl ::std::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        self.text_area.storage.write_str(s)
    }
}
