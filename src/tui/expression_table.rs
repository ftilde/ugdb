use unsegen::widget::widgets::{
    Column,
    JsonViewer,
    LineEdit,
    Table,
    TableRow,
};
use unsegen::widget::{
    Demand,
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
            result: JsonViewer::new(""),
        }
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

impl ExpressionTable {
    pub fn new() -> Self {
        let row_sep_style = SeparatingStyle::AlternatingStyle(StyleModifier::new().bg_color(Color::Yellow));
        let col_sep_style = SeparatingStyle::Draw(GraphemeCluster::try_from('│').unwrap());
        let focused_style = StyleModifier::new().bold(true).underline(true);
        let mut table = Table::new(row_sep_style, col_sep_style, focused_style);
        table.rows.push(ExpressionRow::new());
        table.rows.push(ExpressionRow::new());
        ExpressionTable {
            table: table,
        }
    }
    pub fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB) {
        event
            .chain(|i: Input| match i.event {
            _ => Some(i),
        })
            .chain(self.table.current_cell_behavior())
            .chain(NavigateBehavior::new(&mut self.table)
                 .up_on(Key::Up)
                 .down_on(Key::Down)
                 .left_on(Key::Left)
                 .right_on(Key::Right)
                 .left_on(Key::Left));

        self.update_results(gdb);
    }

    pub fn update_results(&mut self, gdb: &mut gdbmi::GDB) {
        for row in self.table.rows.iter_mut() {
            let expr = row.expression.get().to_owned();
            let res_text = if expr.is_empty() {
                "".to_owned()
            } else {
                let res = gdb.execute(MiCommand::data_evaluate_expression(expr)).expect("expression evaluation successful");
                match res.class {
                    ResultClass::Error => {
                        format!("<Err: {}>", res.results["msg"].as_str().expect("msg present"))
                    },
                    ResultClass::Done => {
                        format!("{}", res.results["value"].as_str().expect("value present"))
                    }
                    other => {
                        panic!("unexpected result class: {:?}", other)
                    }
                }
            };
            row.result.set(res_text);
        }
    }
}

impl Widget for ExpressionTable {
    fn space_demand(&self) -> (Demand, Demand) {
        self.table.space_demand()
    }
    fn draw(&mut self, window: Window) {
        self.table.draw(window);
    }
}
