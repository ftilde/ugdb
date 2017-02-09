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
    SeparatingStyle,
};
use unsegen::widgets::{
    LogViewer,
    PromptLine,
    Pager,
    FileLineStorage,
    NoHighLighter,
};
use unsegen::input::{
    Writable,
    WriteBehavior,
    Editable,
    EditBehavior,
    Scrollable,
    ScrollBehavior,
};

struct Console {
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
        write!(self.text_area, " -=- {}\n", msg).expect("Write message");
    }

    pub fn event(&mut self, input: unsegen::Input, gdb: &mut gdbmi::GDB) { //TODO more console events
        if input.event == Event::Key(Key::Char('\n')) {
            let line = self.prompt_line.finish_line().to_owned();
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

// Terminal ---------------------------------------------------------------------------------------

use pty;
pub struct PseudoTerminal {
    //width: u32,
    //height: u32,
    pty: pty::PTYInput,
    display: unsegen::widgets::LogViewer,
    //prompt_line: unsegen::widgets::PromptLine,
    //layout: unsegen::VerticalLayout,

    input_buffer: Vec<u8>,
}

impl PseudoTerminal {
    pub fn new(pty: pty::PTYInput) -> Self {
        PseudoTerminal {
            pty: pty,
            display: unsegen::widgets::LogViewer::new(),
            //prompt_line: unsegen::widgets::PromptLine::with_prompt("".into()),
            //layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
            input_buffer: Vec::new(),
        }
    }

    fn add_byte_input(&mut self, mut bytes: Vec<u8>) {
        self.input_buffer.append(&mut bytes);

        //TODO: handle control sequences?
        if let Ok(string) = String::from_utf8(self.input_buffer.clone()) {
            use std::fmt::Write;
            self.display.write_str(&string).expect("Write byte to terminal");
            self.input_buffer.clear();
        }
    }
}

impl Widget for PseudoTerminal {
    fn space_demand(&self) -> (Demand, Demand) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.space_demand(widgets.into_iter())
        return self.display.space_demand();
    }
    fn draw(&mut self, window: Window) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.draw(window, &widgets)
        self.display.draw(window);
    }
}

impl Writable for PseudoTerminal {
    fn write(&mut self, c: char) {
        use std::io::Write;
        write!(self.pty, "{}", c).expect("Write key to terminal");
    }
}

// Gui --------------------------------------------------------------------------------
pub struct Gui {
    console: Console,
    process_pty: PseudoTerminal,
    file_viewer: Pager<FileLineStorage, NoHighLighter>,

    left_layout: VerticalLayout,
    right_layout: VerticalLayout,
}

impl Gui {
    //pub fn new(process_pty: ::pty::PTYInput, theme_set: &'a ::syntect::highlighting::ThemeSet) -> Self {
            //file_viewer: Pager::new("/home/dominik/test.rs", &theme_set.themes["base16-ocean.dark"]),

    pub fn new(process_pty: ::pty::PTYInput) -> Self {
        Gui {
            console: Console::new(),
            process_pty: PseudoTerminal::new(process_pty),
            file_viewer: Pager::new(FileLineStorage::new("/home/dominik/test.rs").expect("open file"), NoHighLighter),
            left_layout: VerticalLayout::new(SeparatingStyle::Draw('=')),
            right_layout: VerticalLayout::new(SeparatingStyle::Draw('=')),
        }
    }

    pub fn add_out_of_band_record(&mut self, record: gdbmi::output::OutOfBandRecord) {
        self.console.add_message(format!("oob: {:?}", record));
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
            ::input::InputEvent::ConsoleEvent(event) => { self.console.event(event, gdb); },
            ::input::InputEvent::PseudoTerminalEvent(event) => { event.chain(WriteBehavior::new(&mut self.process_pty)); },
            ::input::InputEvent::Quit => { unreachable!("quit should have been caught in main" ) }, //TODO this is ugly
        }
    }
}
