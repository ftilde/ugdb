pub trait TableRow {
    fn column_accessors() -> &'static [fn(&Self) -> &Widget];
}

pub struct Table<R: TableRow> {
    pub rows: Vec<R>,
}

impl<R: TableRow> Table<R> {
    pub fn new() {
        Table {
            rows: Vec::new(),
        }
    }

    fn num_columns() -> usize {
        R::column_accessors.len()
    }

    fn layout_columns(&self, window: &window) -> Box<[u32]> {
        let x_demands = [Demand::exact(0); Self::num_columns()];
        for row in rows {
            for (col_num, accessor) in R::column_accessors().enumerate() {
                let (x, _) = accessor(row).space_demand();
                x_demands[col_num].max_assign(x);
            }
        }
        let separator_width = 1; //TODO
        layout_linearly(window.get_width(), separator_width, x_demands.iter())
    }
}

impl<R: TableRow> Widget for Table<R> {
    fn space_demand(&self) -> (Demand, Demand) {
        let x_demands = [Demand::exact(0); Self::num_columns()];
        let y_demand = Demand::exact(0);
        for row in rows {
            row_max_y = Demand::exact(0);
            for (col_num, accessor) in R::column_accessors().enumerate() {
                let (x, y) = accessor(row).space_demand();
                x_demands[col_num].max_assign(x);
                row_max_y.max_assign(y)
            }
            y_demand += row_max_y;
        }
        (x_demands.iter().sum(), y_demand)
    }
    fn draw(&mut self, window: Window) {
        let column_widths = layout_columns(&window);
        //TODO
    }
}
