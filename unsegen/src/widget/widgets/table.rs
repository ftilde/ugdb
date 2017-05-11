use base::{
    GraphemeCluster,
    Window,
    StyleModifier,
};
use input::{
    Navigatable,
};
use widget::{
    Demand,
    SeparatingStyle,
    Widget,
    layout_linearly,
};
use std::cmp::{
    min,
};

pub type ColumnAccessor<T> = fn(&T) -> &Widget;
pub type ColumnAccessorMut<T> = fn(&mut T) -> &mut Widget;

pub trait TableRow {
    fn column_accessors() -> &'static [ColumnAccessor<Self>];
    fn column_accessors_mut() -> &'static [ColumnAccessorMut<Self>];

    fn num_columns() -> usize where Self: 'static {
        Self::column_accessors().len()
    }

    fn height_demand(&self) -> Demand where Self: 'static {
        let mut y_demand = Demand::zero();
        for  accessor in Self::column_accessors().iter() {
            let (_, y) = accessor(self).space_demand();
            y_demand.max_assign(y);
        }
        y_demand
    }
}

pub struct Table<R: TableRow> {
    pub rows: Vec<R>,
    pub row_sep_style: SeparatingStyle,
    pub col_sep_style: SeparatingStyle,
    row_pos: u32,
    col_pos: u32,
}

impl<R: TableRow + 'static> Table<R> {
    pub fn new() -> Self {
        Table {
            rows: Vec::new(),
            row_sep_style: SeparatingStyle::Draw(GraphemeCluster::try_from('─').unwrap()),
            col_sep_style: SeparatingStyle::Draw(GraphemeCluster::try_from('│').unwrap()),
            row_pos: 0,
            col_pos: 1,
        }
    }

    fn layout_columns(&self, window: &Window) -> Box<[u32]> {
        let mut x_demands = vec![Demand::zero(); R::num_columns()];
        for row in self.rows.iter() {
            for (col_num, accessor) in R::column_accessors().iter().enumerate() {
                let (x, _) = accessor(row).space_demand();
                x_demands[col_num].max_assign(x);
            }
        }
        let separator_width = self.col_sep_style.width();
        layout_linearly(window.get_width(), separator_width, &x_demands)
    }
    fn ensure_valid_row_pos(&mut self) {
        self.row_pos = min(self.row_pos, (self.rows.len() as u32).checked_sub(1).unwrap_or(0));
    }
}

impl<R: TableRow + 'static> Widget for Table<R> {
    fn space_demand(&self) -> (Demand, Demand) {
        let mut x_demands = vec![Demand::exact(0); R::num_columns()];
        let mut y_demand = Demand::zero();

        let mut row_iter = self.rows.iter().peekable();
        while let Some(row) = row_iter.next() {
            let mut row_max_y = Demand::exact(0);
            for (col_num, accessor) in R::column_accessors().iter().enumerate() {
                let (x, y) = accessor(row).space_demand();
                x_demands[col_num].max_assign(x);
                row_max_y.max_assign(y)
            }
            y_demand += row_max_y;
            if row_iter.peek().is_some() {
                y_demand += Demand::exact(self.row_sep_style.height());
            }
        }

        //Account all separators between cols
        let x_demand = x_demands.iter().sum::<Demand>() + Demand::exact((x_demands.len() as u32 -1)*self.col_sep_style.width());
        (x_demand, y_demand)
    }
    fn draw(&mut self, window: Window) {
        let column_widths = self.layout_columns(&window);
        let focused_style = StyleModifier::new().invert().apply(window.default_style());

        let mut window = window;
        let mut row_iter = self.rows.iter_mut().enumerate().peekable();
        while let Some((row_index, row)) = row_iter.next() {
            let height = row.height_demand().min;
            let (mut row_window, rest_window) = window.split_v(height);
            window = rest_window;

            let mut iter = R::column_accessors_mut().iter().zip(column_widths.iter()).enumerate().peekable();
            while let Some((col_index, (accessor, &pos))) = iter.next() {
                let (mut cell_window, r) = row_window.split_h(pos);
                row_window = r;
                if row_index as u32 == self.row_pos && col_index as u32 == self.col_pos {
                    cell_window.set_default_style(focused_style);
                    cell_window.clear();
                }
                accessor(row).draw(cell_window);
                if let (Some(_), &SeparatingStyle::Draw(ref c)) = (iter.peek(), &self.col_sep_style) {
                    if row_window.get_width() > 0 {
                        let (mut sep_window, r) = row_window.split_h(c.width() as u32);
                        row_window = r;
                        sep_window.fill(c.clone());
                    }
                }
            }
            if let (Some(_), &SeparatingStyle::Draw(ref c)) = (row_iter.peek(), &self.row_sep_style) {
                let (mut sep_window, r) = window.split_v(1);
                window = r;
                sep_window.fill(c.clone());
            }
        }
    }
}

impl<R: TableRow + 'static> Navigatable for Table<R> {
    fn move_up(&mut self) {
        self.row_pos = self.row_pos.checked_sub(1).unwrap_or(0);
    }
    fn move_down(&mut self) {
        self.row_pos += 1;
        self.ensure_valid_row_pos();
    }
    fn move_left(&mut self) {
        self.col_pos = self.col_pos.checked_sub(1).unwrap_or(0);
    }
    fn move_right(&mut self) {
        self.col_pos = min(self.col_pos+1, R::num_columns() as u32 -1);
    }
}
