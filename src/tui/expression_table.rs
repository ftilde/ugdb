use unsegen::widget::widgets::{
    ColumnAccessor,
    ColumnAccessorMut,
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

    fn get_result_as_widget(&self) -> &Widget {
        &self.result
    }

    fn get_result_as_widget_mut(&mut self) -> &mut Widget {
        &mut self.result
    }
}
impl TableRow for ExpressionRow {
    fn column_accessors() -> &'static [ColumnAccessor<ExpressionRow>] {
        const W: &'static [ColumnAccessor<ExpressionRow>] = &[ExpressionRow::get_expression_as_widget, ExpressionRow::get_result_as_widget];
        W
    }
    fn column_accessors_mut() -> &'static [ColumnAccessorMut<ExpressionRow>] {
        const W: &'static [ColumnAccessorMut<ExpressionRow>] = &[ExpressionRow::get_expression_as_widget_mut, ExpressionRow::get_result_as_widget_mut];
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
    pub fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB) {
        event.chain(|i: Input| match i.event {
            _ => Some(i),
        }).chain(NavigateBehavior::new(&mut self.table)
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
