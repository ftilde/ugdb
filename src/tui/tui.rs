use unsegen::base::{
    Color,
    GraphemeCluster,
    Style,
    TextFormat,
    Window,
};
use unsegen::widget::{
    RenderingHints,
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
use unsegen_terminal::{
    PseudoTerminal,
    PTYInput,
};
use super::expression_table::ExpressionTable;

pub struct Tui<'a> {
    console: Console,
    expression_table: ExpressionTable,
    process_pty: PseudoTerminal,
    src_view: CodeWindow<'a>,

    left_layout: VerticalLayout,
    right_layout: VerticalLayout,

    active_window: SubWindow, // This is a temporary solution until container management is implemented
}

#[derive(PartialEq, Eq)]
enum SubWindow {
    Console,
    CodeWindow,
    PseudoTerminal,
    ExpressionTable,
}

impl<'a> Tui<'a> {

    pub fn new(process_pty: PTYInput, highlighting_theme: &'a Theme) -> Self {
        Tui {
            console: Console::new(),
            expression_table: ExpressionTable::new(),
            process_pty: PseudoTerminal::new(process_pty),
            src_view: CodeWindow::new(highlighting_theme),
            left_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            right_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            active_window: SubWindow::CodeWindow,
        }
    }

    fn handle_async_record(&mut self, kind: AsyncKind, class: AsyncClass, results: &Object, gdb: &mut gdbmi::GDB) {
        match (kind, class) {
            (AsyncKind::Exec, AsyncClass::Stopped) => {
                self.console.add_debug_message(format!("stopped: {}", JsonValue::Object(results.clone()).pretty(2)));
                if let JsonValue::Object(ref frame) = results["frame"] {
                    self.src_view.show_frame(frame, gdb);
                }
                self.expression_table.update_results(gdb);
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
        let (window_l, rest) = window.split_h(split_pos).expect("Valid split pos guaranteed");

        let (mut separator, window_r) = rest.split_h(2).expect("Valid split size for (not too small) terminals");

        separator.set_default_style(Style::new(Color::Green, Color::Blue, TextFormat{ bold: true, underline: true, invert: false, italic: true }));
        separator.fill(GraphemeCluster::try_from('山').unwrap());

        let inactive_hints = RenderingHints {
            active: false,
            .. Default::default()
        };
        let active_hints = RenderingHints {
            active: true,
            .. Default::default()
        };

        let mut left_widgets: Vec<(&mut Widget, RenderingHints)> = vec![
            (&mut self.src_view, if self.active_window == SubWindow::CodeWindow { active_hints } else { inactive_hints }),
            (&mut self.console, if self.active_window == SubWindow::Console { active_hints } else { inactive_hints }),
        ];
        self.left_layout.draw(window_l, &mut left_widgets);

        let mut right_widgets: Vec<(&mut Widget, RenderingHints)> = vec![
            (&mut self.expression_table, if self.active_window == SubWindow::ExpressionTable { active_hints } else { inactive_hints }),
            (&mut self.process_pty, if self.active_window == SubWindow::PseudoTerminal { active_hints } else { inactive_hints }),
        ];
        self.right_layout.draw(window_r, &mut right_widgets);
    }

    pub fn event(&mut self, event: InputEvent, gdb: &mut gdbmi::GDB) { //TODO more console events
        match event {
            InputEvent::ConsoleEvent(event) => {
                self.active_window = SubWindow::Console;
                self.console.event(event, gdb);
            },
            InputEvent::PseudoTerminalEvent(event) => {
                self.active_window = SubWindow::PseudoTerminal;
                event.chain(WriteBehavior::new(&mut self.process_pty));
            },
            InputEvent::SourcePagerEvent(event) => {
                self.active_window = SubWindow::CodeWindow;
                self.src_view.event(event, gdb);
            },
            InputEvent::ExpressionTableEvent(event) => {
                self.active_window = SubWindow::ExpressionTable;
                self.expression_table.event(event, gdb);
            },
            InputEvent::Quit => {
                unreachable!("quit should have been caught in main" )
            }, //TODO this is ugly
        }
    }
}
