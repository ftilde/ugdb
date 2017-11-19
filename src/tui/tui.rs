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
};
use logging::{
    LogMsgType,
};
use syntect::highlighting::{
    Theme,
};

use gdbmi::output::{
    AsyncClass,
    AsyncKind,
    JsonValue,
    Object,
    OutOfBandRecord,
    ThreadEvent,
};

use super::console::Console;
use super::srcview::CodeWindow;
use unsegen_terminal::{
    Terminal,
};
use super::expression_table::ExpressionTable;
use unsegen::container::{Accessor, ContainerProvider};

pub struct Tui<'a> {
    pub console: Console,
    expression_table: ExpressionTable,
    process_pty: Terminal,
    src_view: CodeWindow<'a>,

    left_layout: VerticalLayout,
    right_layout: VerticalLayout,

    active_window: SubWindow, // This is a temporary solution until container management is implemented
}

#[derive(PartialEq, Eq)]
enum SubWindow {
    Console,
    CodeWindow,
    Terminal,
    ExpressionTable,
}

const WELCOME_MSG: &'static str = r#"
       Welcome to          
 _   _  __ _  __| | |__    
| | | |/ _` |/ _` | '_ \   
| |_| | (_| | (_| | |_) |  
 \__,_|\__, |\__,_|_.__/   
       |___/               
"#;

impl<'a> Tui<'a> {

    pub fn new(terminal: Terminal, highlighting_theme: &'a Theme) -> Self {
        Tui {
            console: Console::new(),
            expression_table: ExpressionTable::new(),
            process_pty: terminal,
            src_view: CodeWindow::new(highlighting_theme, WELCOME_MSG),
            left_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            right_layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            active_window: SubWindow::CodeWindow,
        }
    }

    fn handle_async_record(&mut self, kind: AsyncKind, class: AsyncClass, results: &Object, p: ::UpdateParameters) {
        match (kind, class) {
            (AsyncKind::Exec, AsyncClass::Stopped) | (AsyncKind::Notify, AsyncClass::Thread(ThreadEvent::Selected))=> {
                p.logger.log(LogMsgType::Debug, format!("stopped: {}", JsonValue::Object(results.clone()).pretty(2)));
                if let JsonValue::Object(ref frame) = results["frame"] {
                    self.src_view.show_frame(frame, p);
                }
                self.expression_table.update_results(p);
            },
            (AsyncKind::Notify, AsyncClass::BreakPoint(event)) => {
                p.logger.log(LogMsgType::Debug, format!("bkpoint {:?}: {}", event, JsonValue::Object(results.clone()).pretty(2)));
                p.gdb.handle_breakpoint_event(event, &results);
            },
            (kind, class) => {
                p.logger.log(LogMsgType::Debug, format!("unhandled async_record: [{:?}, {:?}] {}", kind, class, JsonValue::Object(results.clone()).pretty(2)));
            },
        }
    }

    pub fn add_out_of_band_record(&mut self, record: OutOfBandRecord, p: ::UpdateParameters) {
        match record {
            OutOfBandRecord::StreamRecord{ kind: _, data} => {
                self.console.write_to_gdb_log(data);
            },
            OutOfBandRecord::AsyncRecord{token: _, kind, class, results} => {
                self.handle_async_record(kind, class, &results, p);
            },

        }
    }

    pub fn add_pty_input(&mut self, input: Box<[u8]>) {
        self.process_pty.add_byte_input(input);
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

        self.left_layout.draw(window_l, &[
                              (&self.src_view, if self.active_window == SubWindow::CodeWindow { active_hints } else { inactive_hints }),
                              (&self.console, if self.active_window == SubWindow::Console { active_hints } else { inactive_hints }),
        ]);

        self.right_layout.draw(window_r, &[
            (&self.expression_table, if self.active_window == SubWindow::ExpressionTable { active_hints } else { inactive_hints }),
            (&self.process_pty, if self.active_window == SubWindow::Terminal { active_hints } else { inactive_hints }),
        ]);
    }

    pub fn update_after_event(&mut self, p: ::UpdateParameters) {
        self.src_view.update_after_event(p);
    }
}

impl<'a> ContainerProvider for Tui<'a> {
    type Parameters = ::UpdateParametersStruct;
    fn get_accessor(identifier: &str) -> Option<Accessor<Self>> {
        match identifier {
            "srcview" => Some(Accessor {
                access: |s| &s.src_view,
                access_mut: |s| &mut s.src_view,
            }),
            "console" => Some(Accessor {
                access: |s| &s.console,
                access_mut: |s| &mut s.console,
            }),
            "expressiontable" => Some(Accessor {
                access: |s| &s.expression_table,
                access_mut: |s| &mut s.expression_table,
            }),
            "terminal" => Some(Accessor {
                access: |s| &s.process_pty,
                access_mut: |s| &mut s.process_pty,
            }),
            _ => None,
        }
    }
    const DEFAULT_CONTAINER: &'static str = "console";
}
