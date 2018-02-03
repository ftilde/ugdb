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
use unsegen::container::{Container, ContainerProvider};

pub struct Tui<'a> {
    pub console: Console,
    expression_table: ExpressionTable,
    process_pty: Terminal,
    src_view: CodeWindow<'a>,
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

    pub fn update_after_event(&mut self, p: ::UpdateParameters) {
        self.src_view.update_after_event(p);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TuiContainerType {
    SrcView,
    Console,
    ExpressionTable,
    Terminal,
}

impl<'t> ContainerProvider for Tui<'t> {
    type Parameters = ::UpdateParametersStruct;
    type Index = TuiContainerType;
    fn get<'a, 'b: 'a>(&'b self, index: &'a Self::Index) -> &'b Container<Self::Parameters> {
        match index {
            &TuiContainerType::SrcView => &self.src_view,
            &TuiContainerType::Console => &self.console,
            &TuiContainerType::ExpressionTable => &self.expression_table,
            &TuiContainerType::Terminal => &self.process_pty,
        }
    }
    fn get_mut<'a, 'b: 'a>(&'b mut self, index: &'a Self::Index) -> &'b mut Container<Self::Parameters> {
        match index {
            &TuiContainerType::SrcView => &mut self.src_view,
            &TuiContainerType::Console => &mut self.console,
            &TuiContainerType::ExpressionTable => &mut self.expression_table,
            &TuiContainerType::Terminal => &mut self.process_pty,
        }
    }
    const DEFAULT_CONTAINER: TuiContainerType = TuiContainerType::Console;
}
