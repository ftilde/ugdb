use unsegen::base::{
    Color,
    GraphemeCluster,
    Style,
    TextFormat,
    Window,
};
use unsegen::widget::{
    SeparatingStyle,
    VerticalLayout,
    Widget,
};
use unsegen::input::{
    WriteBehavior,
};
use input::{
    InputEvent,
};
use syntect::highlighting::{
    Theme,
};

use gdbmi;
use gdbmi::output::{
    OutOfBandRecord,
    AsyncKind,
    AsyncClass,
    Object,
    JsonValue,
};

use super::console::Console;
use super::srcview::CodeWindow;
use super::pseudoterminal::PseudoTerminal;

pub struct Tui<'a> {
    console: Console,
    process_pty: PseudoTerminal,
    src_view: CodeWindow<'a>,

    left_layout: VerticalLayout,
    right_layout: VerticalLayout,
}

impl<'a> Tui<'a> {

    pub fn new(process_pty: ::pty::PTYInput, highlighting_theme: &'a Theme) -> Self {
        Tui {
            console: Console::new(),
            process_pty: PseudoTerminal::new(process_pty),
            src_view: CodeWindow::new(highlighting_theme),
            left_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            right_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
        }
    }

    fn handle_async_record(&mut self, kind: AsyncKind, class: AsyncClass, results: &Object, gdb: &mut gdbmi::GDB) {
        match (kind, class) {
            (AsyncKind::Exec, AsyncClass::Stopped) => {
                self.console.add_debug_message(format!("stopped: {}", JsonValue::Object(results.clone()).pretty(2)));
                if let JsonValue::Object(ref frame) = results["frame"] {
                    self.src_view.show_frame(frame, gdb);
                }
            },
            (AsyncKind::Notify, AsyncClass::BreakPoint(event)) => {
                self.console.add_debug_message(format!("bkpoint {:?}: {}", event, JsonValue::Object(results.clone()).pretty(2)));
                self.src_view.handle_breakpoint_event(event, &results);
            },
            (kind, class) => {
                self.console.add_debug_message(format!("unhandled async_record: [{:?}, {:?}] {}", kind, class, JsonValue::Object(results.clone()).pretty(2)));
            },
        }
    }

    pub fn add_out_of_band_record(&mut self, record: OutOfBandRecord, gdb: &mut gdbmi::GDB) {
        match record {
            OutOfBandRecord::StreamRecord{ kind: _, data} => {
                self.console.add_message(data);
            },
            OutOfBandRecord::AsyncRecord{token: _, kind, class, results} => {
                self.handle_async_record(kind, class, &results, gdb);
            },

        }
    }

    pub fn add_pty_input(&mut self, input: Vec<u8>) {
        self.process_pty.add_byte_input(input);
    }

    pub fn add_debug_message(&mut self, msg: &str) {
        self.console.add_debug_message(format!("Debug: {}", msg));
    }

    pub fn draw(&mut self, window: Window) {
        let split_pos = window.get_width()/2-1;
        let (window_l, rest) = window.split_h(split_pos);

        let (mut separator, window_r) = rest.split_h(2);

        separator.set_default_style(Style::new(Color::Green, Color::Blue, TextFormat::new().bold().italic().underline()));
        separator.fill(GraphemeCluster::try_from('山').unwrap());

        let mut left_widgets: Vec<&mut Widget> = vec![&mut self.src_view, &mut self.console];
        self.left_layout.draw(window_l, &mut left_widgets);

        let mut right_widgets: Vec<&mut Widget> = vec![&mut self.process_pty];
        self.right_layout.draw(window_r, &mut right_widgets);
    }

    pub fn event(&mut self, event: InputEvent, gdb: &mut gdbmi::GDB) { //TODO more console events
        match event {
            InputEvent::ConsoleEvent(event) => {
                self.console.event(event, gdb);
            },
            InputEvent::PseudoTerminalEvent(event) => {
                event.chain(WriteBehavior::new(&mut self.process_pty));
            },
            InputEvent::SourcePagerEvent(event) => {
                self.src_view.event(event, gdb)
            },
            InputEvent::Quit => {
                unreachable!("quit should have been caught in main" )
            }, //TODO this is ugly
        }
    }
}
