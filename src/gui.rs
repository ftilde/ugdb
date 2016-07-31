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
        self.text_area.add_line(msg);
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
    fn draw(&self, mut window: unsegen::Window) {
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
    prompt_line: unsegen::widgets::PromptLine, //TODO Maybe we don't want a prompt line after all...
    layout: unsegen::VerticalLayout,
}

impl PseudoTerminal {
    pub fn new(pty: pty::PTYInput) -> Self {
        PseudoTerminal {
            pty: pty,
            display: unsegen::widgets::TextArea::new(),
            prompt_line: unsegen::widgets::PromptLine::with_prompt("".into()),
            layout: unsegen::VerticalLayout::new(unsegen::SeparatingStyle::Draw('=')), //TODO none?
        }
    }

    fn add_output(&mut self, output: String) {
        self.display.add_line(output);
    }
}

impl unsegen::Widget for PseudoTerminal {
    fn space_demand(&self) -> (unsegen::Demand, unsegen::Demand) {
        //return (super::Demand::MaxPossible /*TODO?*/, super::Demand::Const(self.lines.len() as u32));
        return self.display.space_demand();
    }
    fn draw(&self, window: unsegen::Window) {
        self.display.draw(window);
    }
    fn input(&mut self, event: unsegen::Event) {
        use std::io::Write;
        use unsegen::{Event,Key,Widget};
        if event == Event::Key(Key::Char('\n')) {
            let line = self.prompt_line.finish_line().to_owned();
            self.display.add_line(line.clone());
            write!(self.pty, "{}\n", line);
        } else {
            self.prompt_line.input(event);
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
        self.console.add_message(format!("oob: {:?}\n", record));
    }

    pub fn add_pty_output(&mut self, output: String) {
        self.process_pty.add_output(output);
    }

    pub fn draw(&self, window: unsegen::Window) {
        use unsegen::{Widget, TextAttribute, Color};
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

    pub fn event(&mut self, event: unsegen::Event, gdb: &mut gdbmi::GDB) { //TODO more console events
        use unsegen::Widget;
        self.console.event(event, gdb);
        self.process_pty.input(event);
    }
}
