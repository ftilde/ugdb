use unsegen::widget::widgets::{
    Column,
    LineEdit,
    LineLabel,
    Table,
    TableRow,
};
use unsegen::widget::{
    Demand,
    Widget,
};
use unsegen::base::{
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

pub struct ExpressionRow {
    expression: LineEdit,
    result: LineLabel,
}
impl ExpressionRow {
    fn new() -> Self {
        ExpressionRow {
            expression: LineEdit::new(),
            result: LineLabel::new("<RESULT>"),
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
        let mut table = Table::new();
        table.rows.push(ExpressionRow::new());
        table.rows.push(ExpressionRow::new());
        ExpressionTable {
            table: table,
        }
    }
    pub fn event(&mut self, event: Input, _: &mut gdbmi::GDB) {
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
