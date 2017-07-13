use unsegen::input::{
    Input,
    Key,
    Event,
};

use chan::{
    Sender,
};

use time::{
    Duration,
    SteadyTime,
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

// waiting for const fn:
//const DOUBLE_ESCAPE_DURATION_THRESHOLD: Duration = Duration::milliseconds(100);

impl ViKeyboardInput {

    fn input_loop(output: Sender<InputEvent>) {
        let double_escape_duration_threshold = Duration::milliseconds(200);
        // Slight hack: define last_escape_time so that is always farther in the past than the
        // DOUBLE_ESCAPE_DURATION_THRESHOLD.
        let mut last_escape_time = SteadyTime::now() - double_escape_duration_threshold;

        let mut mode = Mode::Console;
        let stdin = ::std::io::stdin(); //TODO lock outside of thread?
        let stdin = stdin.lock();

        for e in Input::real_all(stdin) {
            let input = e.expect("event");
            if let &Event::Key(Key::Ctrl('q')) = &input.event {
                output.send(InputEvent::Quit);
            }
            // Work around need to clone event for update of last_escape_time
            let event_is_escape = &Event::Key(Key::Esc) == &input.event;

            let (new_mode, optional_event) = match mode {
                Mode::SourcePager => {
                    match input.event.clone() {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        Event::Key(Key::Char('i')) => { (Mode::Console, None) },
                        Event::Key(Key::Char('t')) => { (Mode::PTY, None) },
                        Event::Key(Key::Char('e')) => { (Mode::ExpressionTable, None) },
                        _ => { (mode, Some(InputEvent::SourcePagerEvent(input))) },
                    }
                },
                Mode::Console => {
                    match input.event.clone() {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        Event::Key(Key::F(1)) => { (mode, Some(InputEvent::ConsoleEvent(ConsoleEvent::ToggleLog))) },
                        _ => { (mode, Some(InputEvent::ConsoleEvent(ConsoleEvent::Raw(input)))) },
                    }
                },
                Mode::PTY => {
                    if event_is_escape && SteadyTime::now() - last_escape_time < double_escape_duration_threshold {
                        (Mode::SourcePager, None)
                    } else {
                        (mode, Some(InputEvent::PseudoTerminalEvent(input)))
                    }
                },
                Mode::ExpressionTable => {
                    match input.event.clone() {
                        Event::Key(Key::Esc) => { (Mode::SourcePager, None) },
                        _ => { (mode, Some(InputEvent::ExpressionTableEvent(input))) },
                    }
                },
            };
            mode = new_mode;

            if event_is_escape {
                last_escape_time = SteadyTime::now();
            }

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
