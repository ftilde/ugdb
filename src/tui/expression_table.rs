use unsegen::widget::widgets::{
    Column,
    JsonValue,
    JsonViewer,
    LineEdit,
    Table,
    TableRow,
};
use unsegen::widget::{
    Demand2D,
    RenderingHints,
    SeparatingStyle,
    Widget,
};
use unsegen::base::{
    Color,
    GraphemeCluster,
    StyleModifier,
    Window,
};
use unsegen::input::{
    NavigateBehavior,
    EditBehavior,
    Key,
};
use input::{
    Input,
};
use gdbmi;
use gdbmi::input::{
    MiCommand,
};
use gdbmi::output::{
    ResultClass,
};

pub struct ExpressionRow {
    expression: LineEdit,
    result: JsonViewer,
}
impl ExpressionRow {
    fn new() -> Self {
        ExpressionRow {
            expression: LineEdit::new(),
            result: JsonViewer::new(&JsonValue::Null),
        }
    }

    fn is_empty(&self) -> bool {
        self.expression.get().is_empty()
    }

    fn get_expression_as_widget(&self) -> &Widget {
        &self.expression
    }

    fn get_expression_as_widget_mut(&mut self) -> &mut Widget {
        &mut self.expression
    }

    fn pass_event_to_expression(&mut self, input: Input) -> Option<Input>{
        input.chain(EditBehavior::new(&mut self.expression)
                    .left_on(Key::Left)
                    .right_on(Key::Right)
                    .up_on(Key::Up)
                    .down_on(Key::Down)
                    .delete_symbol_on(Key::Delete)
                    .remove_symbol_on(Key::Backspace)
                    .clear_on(Key::Ctrl('c'))
                    ).finish()
    }

    fn get_result_as_widget(&self) -> &Widget {
        &self.result
    }

    fn get_result_as_widget_mut(&mut self) -> &mut Widget {
        &mut self.result
    }

    fn pass_event_to_result(&mut self, input: Input) -> Option<Input> {
        //TODO
        Some(input)
    }
}
impl TableRow for ExpressionRow {
    fn columns() -> &'static [Column<ExpressionRow>] {
        const W: &'static [Column<ExpressionRow>] = &[
            Column {
                access: ExpressionRow::get_expression_as_widget,
                access_mut: ExpressionRow::get_expression_as_widget_mut,
                behavior: ExpressionRow::pass_event_to_expression,
            },
            Column {
                access: ExpressionRow::get_result_as_widget,
                access_mut: ExpressionRow::get_result_as_widget_mut,
                behavior: ExpressionRow::pass_event_to_result,
            },
        ];
        W
    }
}

pub struct ExpressionTable {
    table: Table<ExpressionRow>,
}

fn parse_gdb_value(result_string: &str) -> JsonValue {
    JsonValue::String(result_string.to_owned())
}


impl ExpressionTable {
    pub fn new() -> Self {
        let row_sep_style = SeparatingStyle::AlternatingStyle(StyleModifier::new().bg_color(Color::Yellow));
        let col_sep_style = SeparatingStyle::Draw(GraphemeCluster::try_from('│').unwrap());
        let focused_style = StyleModifier::new().bold(true).underline(true);
        let mut table = Table::new(row_sep_style, col_sep_style, focused_style);
        table.rows_mut().push(ExpressionRow::new()); //Invariant: always at least one line
        ExpressionTable {
            table: table,
        }
    }
    pub fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB) {
        event
            .chain(|i: Input| match i.event {
                _ => Some(i),
            })
            .chain(NavigateBehavior::new(&mut self.table)
                 .down_on(Key::Char('\n'))
                 )
            .chain(self.table.current_cell_behavior())
            .chain(NavigateBehavior::new(&mut self.table)
                 .up_on(Key::Up)
                 .down_on(Key::Down)
                 .left_on(Key::Left)
                 .right_on(Key::Right)
                 );

        self.shrink_to_fit();
        self.update_results(gdb);
    }

    fn shrink_to_fit(&mut self) {
        let begin_of_empty_range = {
            let iter = self.table.rows().iter().enumerate().rev();
            let mut without_trailing_empty_rows = iter.skip_while(|&(_, r)| r.is_empty());
            if let Some((i,_)) = without_trailing_empty_rows.next() {
                i+1
            } else {
                0
            }
        };
        let mut rows = self.table.rows_mut();
        rows.drain(begin_of_empty_range..);
        rows.push(ExpressionRow::new());
    }

    pub fn update_results(&mut self, gdb: &mut gdbmi::GDB) {
        for row in self.table.rows_mut().iter_mut() {
            let expr = row.expression.get().to_owned();
            let result = if expr.is_empty() {
                JsonValue::Null
            } else {
                let res = gdb.execute(MiCommand::data_evaluate_expression(expr)).expect("expression evaluation successful");
                match res.class {
                    ResultClass::Error => {
                        res.results["msg"].clone()
                    },
                    ResultClass::Done => {
                        parse_gdb_value(res.results["value"].as_str().expect("value present"))
                    },
                    other => {
                        panic!("unexpected result class: {:?}", other)
                    },
                }
            };
            row.result.reset(&result);
        }
    }
}

impl Widget for ExpressionTable {
    fn space_demand(&self) -> Demand2D {
        self.table.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        self.table.draw(window, hints);
    }
}
