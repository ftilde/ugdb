pub use unsegen::input::{
    Input,
    Key,
    Event,
};

use chan::{
    Sender,
};

#[derive(Eq, PartialEq, Clone)]
pub enum ConsoleEvent {
    Raw(Input),
    ToggleLog,
}

#[derive(Eq, PartialEq, Clone)]
pub enum InputEvent {
    ConsoleEvent(ConsoleEvent),
    PseudoTerminalEvent(Input),
    SourcePagerEvent(Input),
    ExpressionTableEvent(Input),
    Quit,
}

pub trait InputSource {
    fn start_loop(event_sink: Sender<InputEvent>) -> Self;
}

#[derive(Clone, Copy)]
enum Mode {
    Console,
    PTY,
    SourcePager,
    ExpressionTable,
}

pub struct ViKeyboardInput {
    _thread: ::std::thread::JoinHandle<()>,
}

impl ViKeyboardInput {
    fn input_loop(output: Sender<InputEvent>) {
        use termion::input::TermRead;

        let mut mode = Mode::Console;
        let stdin = ::std::io::stdin(); //TODO lock outside of thread?
        let stdin = stdin.lock();
        for e in stdin.events() {
            let event = e.expect("event");
            if let Event::Key(Key::Ctrl('q')) = event {
                output.send(InputEvent::Quit);
            }
            let (new_mode, optional_event) = match mode {
                Mode::SourcePager => {
                    match event {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        Event::Key(Key::Char('i')) => { (Mode::Console, None) },
                        Event::Key(Key::Char('t')) => { (Mode::PTY, None) },
                        Event::Key(Key::Char('e')) => { (Mode::ExpressionTable, None) },
                        e => { (mode, Some(InputEvent::SourcePagerEvent(Input::new(e)))) },
                    }
                },
                Mode::Console => {
                    match event {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        Event::Key(Key::F(1)) => { (mode, Some(InputEvent::ConsoleEvent(ConsoleEvent::ToggleLog))) },
                        e => { (mode, Some(InputEvent::ConsoleEvent(ConsoleEvent::Raw(Input::new(e))))) },
                    }
                },
                Mode::PTY => {
                    match event {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        e => { (mode, Some(InputEvent::PseudoTerminalEvent(Input::new(e)))) },
                    }
                },
                Mode::ExpressionTable => {
                    match event {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        e => { (mode, Some(InputEvent::ExpressionTableEvent(Input::new(e)))) },
                    }
                },
            };
            mode = new_mode;
            if let Some(event) = optional_event {
                output.send(event);
            }
        }
    }
}

impl InputSource for ViKeyboardInput {
    fn start_loop(event_sink: Sender<InputEvent>) -> Self {
        let input_thread = ::std::thread::spawn(move || {
            Self::input_loop(event_sink);
        });
        ViKeyboardInput {
            _thread: input_thread,
        }
    }
}
