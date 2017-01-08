use unsegen;
use gdbmi;

struct Console {
    text_area: unsegen::widgets::TextArea,
    prompt_line: unsegen::widgets::PromptLine,
    layout: unsegen::VerticalLayout,
}

impl Console {
    pub fn new() -> Self {
        Console {
            text_area: unsegen::widgets::TextArea::new(),
            prompt_line: unsegen::widgets::PromptLine::with_prompt("(gdb) ".into()),
            layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
        }
    }

    pub fn add_message(&mut self, msg: String) {
        use std::fmt::Write;
        write!(self.text_area, "{}\n", msg).unwrap();
    }

    pub fn event(&mut self, event: unsegen::Event, gdb: &mut gdbmi::GDB) { //TODO more console events
        use unsegen::{Event,Key,Widget};
        if event == Event::Key(Key::Char('\n')) {
            let line = self.prompt_line.finish_line().to_owned();
            self.add_message(format!("(gdb) {}", line));
            match gdb.execute(&gdbmi::input::MiCommand::cli_exec(line)) {
                Ok(result) => {
                    self.add_message(format!("Result: {:?}", result));
                },
                Err(gdbmi::ExecuteError::Quit) => { self.add_message(format!("quit")); },
                //Err(err) => { panic!("Unknown error {:?}", err) },
            }
        } else {
            self.prompt_line.input(event);
        }
    }
}

impl unsegen::Widget for Console {
    fn space_demand(&self) -> (unsegen::Demand, unsegen::Demand) {
        let widgets: Vec<&unsegen::Widget> = vec![&self.text_area, &self.prompt_line];
        self.layout.space_demand(widgets.into_iter())
    }
    fn draw(&self, window: unsegen::Window) {
        let widgets: Vec<&unsegen::Widget> = vec![&self.text_area, &self.prompt_line];
        self.layout.draw(window, &widgets)
    }
    fn input(&mut self, event: unsegen::Event) {
        unimplemented!(); //TODO remove input from Widget into separate trait
    }
}

// Terminal ---------------------------------------------------------------------------------------

use pty;
pub struct PseudoTerminal {
    //width: u32,
    //height: u32,
    pty: pty::PTYInput,
    display: unsegen::widgets::TextArea,
    //prompt_line: unsegen::widgets::PromptLine,
    //layout: unsegen::VerticalLayout,

    input_buffer: Vec<u8>,
}

impl PseudoTerminal {
    pub fn new(pty: pty::PTYInput) -> Self {
        PseudoTerminal {
            pty: pty,
            display: unsegen::widgets::TextArea::new(),
            //prompt_line: unsegen::widgets::PromptLine::with_prompt("".into()),
            //layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
            input_buffer: Vec::new(),
        }
    }

    fn add_byte_input(&mut self, byte: u8) {
        self.input_buffer.push(byte);

        //TODO: handle control sequences?
        if let Ok(string) = String::from_utf8(self.input_buffer.clone()) {
            use std::fmt::Write;
            self.display.write_str(&string).unwrap();
            self.input_buffer.clear();
        }
    }
}

impl unsegen::Widget for PseudoTerminal {
    fn space_demand(&self) -> (unsegen::Demand, unsegen::Demand) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.space_demand(widgets.into_iter())
        return self.display.space_demand();
    }
    fn draw(&self, window: unsegen::Window) {
        //let widgets: Vec<&unsegen::Widget> = vec![&self.display, &self.prompt_line];
        //self.layout.draw(window, &widgets)
        self.display.draw(window);
    }
    fn input(&mut self, event: unsegen::Event) {
        use std::io::Write;
        //use std::fmt::Write as WriteFmt;
        use unsegen::{Event,Key};
        /*
        if event == Event::Key(Key::Char('\n')) {
            let line = self.prompt_line.finish_line().to_owned();
            write!(self.pty, "{}\n", line);
        } else {
            self.prompt_line.input(event);
        }
        */
        if let Event::Key(Key::Char(c)) = event {
            //write!(self.display, "{}", c);
            write!(self.pty, "{}", c).unwrap();
        }
    }
}

// Gui --------------------------------------------------------------------------------

pub struct Gui {
    console: Console,
    process_pty: PseudoTerminal,

    left_layout: unsegen::VerticalLayout,
    right_layout: unsegen::VerticalLayout,
}

impl Gui {
    pub fn new(process_pty: ::pty::PTYInput) -> Self {
        Gui {
            console: Console::new(),
            process_pty: PseudoTerminal::new(process_pty),
            left_layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
            right_layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')),
        }
    }

    pub fn add_out_of_band_record(&mut self, record: gdbmi::output::OutOfBandRecord) {
        self.console.add_message(format!("oob: {:?}", record));
    }

    pub fn add_pty_input(&mut self, input: u8) {
        self.process_pty.add_byte_input(input);
    }

    pub fn add_debug_message(&mut self, msg: &str) {
        self.console.add_message(format!("Debug: {}", msg));
    }

    pub fn draw(&self, window: unsegen::Window) {
        use unsegen::{TextAttribute, Color};
        let split_pos = window.get_width()/2-1;
        let (window_l, rest) = window.split_h(split_pos);

        let (mut separator, window_r) = rest.split_h(2);

        separator.set_default_format(TextAttribute::new(Some(Color::green()), Some(Color::blue()), None));
        separator.fill('|');

        let left_widgets: Vec<&unsegen::Widget> = vec![&self.console];
        self.left_layout.draw(window_l, &left_widgets);

        let right_widgets: Vec<&unsegen::Widget> = vec![&self.process_pty];
        self.right_layout.draw(window_r, &right_widgets);
    }

    pub fn event(&mut self, event: ::input::InputEvent, gdb: &mut gdbmi::GDB) { //TODO more console events
        use unsegen::Widget;
        match event {
            ::input::InputEvent::ConsoleEvent(event) => { self.console.event(event, gdb); },
            ::input::InputEvent::PseudoTerminalEvent(event) => { self.process_pty.input(event); },
            ::input::InputEvent::Quit => { unreachable!("quit should have been caught in main" ) }, //TODO this is ugly
        }
    }
}
