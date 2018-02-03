use super::{
    CursorTarget,
    Style,
    StyleModifier,
    GraphemeCluster,
};
use ndarray::{
    Array,
    ArrayViewMut,
    Axis,
    Ix,
    Ix2,
};
use std::cmp::{max};
use base::ranges::{
    Bound,
    RangeArgument,
};
use base::basic_types::*;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct StyledGraphemeCluster {
    pub grapheme_cluster: GraphemeCluster,
    pub style: Style,
}

impl StyledGraphemeCluster {
    pub fn new(grapheme_cluster: GraphemeCluster, style: Style) -> Self {
        StyledGraphemeCluster {
            grapheme_cluster: grapheme_cluster,
            style: style,
        }
    }
}

impl Default for StyledGraphemeCluster {
    fn default() -> Self {
        Self::new(GraphemeCluster::space(), Style::default())
    }
}

pub type CharMatrix = Array<StyledGraphemeCluster, Ix2>;

#[derive(PartialEq)]
pub struct WindowBuffer {
    storage: CharMatrix,
}

impl WindowBuffer {
    pub fn new(width: Width, height: Height) -> Self {
        WindowBuffer {
            storage: CharMatrix::default(Ix2(height.into(), width.into())),
        }
    }

    pub fn from_storage(storage: CharMatrix) -> Self {
        WindowBuffer {
            storage: storage,
        }
    }

    pub fn as_window<'a>(&'a mut self) -> Window<'a> {
        Window::new(self.storage.view_mut())
    }

    pub fn storage(&self) -> &CharMatrix {
        &self.storage
    }
}

type CharMatrixView<'w> = ArrayViewMut<'w, StyledGraphemeCluster, Ix2>;
pub struct Window<'w> {
    values: CharMatrixView<'w>,
    default_style: Style,
}

impl<'w> ::std::fmt::Debug for Window<'w> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: usize = self.get_width().into();
        let h: usize = self.get_height().into();
        write!(f, "Window {{ w: {}, h: {} }}", w, h)
    }
}

impl<'w> Window<'w> {
    pub fn new(values: CharMatrixView<'w>) -> Self {
        Window {
            values: values,
            default_style: Style::default(),
        }
    }

    pub fn get_width(&self) -> Width {
        Width::new(self.values.dim().1 as i32).unwrap()
    }

    pub fn get_height(&self) -> Height {
        Height::new(self.values.dim().0 as i32).unwrap()
    }

    pub fn clone_mut<'a>(&'a mut self) -> Window<'a> {
        let mat_view_clone = self.values.view_mut();
        Window {
            values: mat_view_clone,
            default_style: self.default_style,
        }
    }

    pub fn create_subwindow<'a, WX: RangeArgument<ColIndex>, WY: RangeArgument<RowIndex>>(&'a mut self, x_range: WX, y_range: WY) -> Window<'a> {
        let x_range_start = match x_range.start() {
            Bound::Unbound => ColIndex::new(0),
            Bound::Inclusive(i) => i,
            Bound::Exclusive(i) => i-1,
        };
        let x_range_end = match x_range.end() {
            Bound::Unbound => self.get_width().from_origin(),
            Bound::Inclusive(i) => i-1,
            Bound::Exclusive(i) => i,
        };
        let y_range_start = match y_range.start() {
            Bound::Unbound => RowIndex::new(0),
            Bound::Inclusive(i) => i,
            Bound::Exclusive(i) => i-1,
        };
        let y_range_end = match y_range.end() {
            Bound::Unbound => self.get_height().from_origin(),
            Bound::Inclusive(i) => i-1,
            Bound::Exclusive(i) => i,
        };
        assert!(x_range_start <= x_range_end, "Invalid x_range: start > end");
        assert!(y_range_start <= y_range_end, "Invalid y_range: start > end");
        assert!(x_range_end <= self.get_width().from_origin(), "Invalid x_range: end > width");
        assert!(y_range_end <= self.get_height().from_origin(), "Invalid y_range: end > height");

        let sub_mat = self.values.slice_mut(s![y_range_start.into()..y_range_end.into(), x_range_start.into()..x_range_end.into()]);
        Window {
            values: sub_mat,
            default_style: self.default_style,
        }
    }

    pub fn split_v(self, split_pos: RowIndex) -> Result<(Self, Self), Self> {
        if (self.get_height()+Height::new(1).unwrap()).origin_range_contains(split_pos) {
            let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos.raw_value() as Ix);
            let w_u = Window {
                values: first_mat,
                default_style: self.default_style,
            };
            let w_d = Window {
                values: second_mat,
                default_style: self.default_style,
            };
            Ok((w_u, w_d))
        } else {
            Err(self)
        }
    }

    pub fn split_h(self, split_pos: ColIndex) -> Result<(Self, Self), Self> {
        if (self.get_width()+Width::new(1).unwrap()).origin_range_contains(split_pos) {
            let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos.raw_value() as Ix);
            let w_l = Window {
                values: first_mat,
                default_style: self.default_style,
            };
            let w_r = Window {
                values: second_mat,
                default_style: self.default_style,
            };
            Ok((w_l, w_r))
        } else {
            Err(self)
        }
    }

    pub fn fill(&mut self, c: GraphemeCluster) {
        let cluster_width = c.width();
        let template = StyledGraphemeCluster::new(c, self.default_style);
        let empty = StyledGraphemeCluster::new(GraphemeCluster::empty(), self.default_style);
        let space = StyledGraphemeCluster::new(GraphemeCluster::space(), self.default_style);
        let w: i32 = self.get_width().into();
        let right_border = (w - (w % cluster_width as i32)) as usize;
        for ((_, x), cell) in self.values.indexed_iter_mut() {
            if x >= right_border.into() {
                *cell = space.clone();
            } else if x % cluster_width == 0 {
                *cell = template.clone();
            } else {
                *cell = empty.clone();
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill(GraphemeCluster::space());
    }

    pub fn set_default_style(&mut self, style: Style) {
        self.default_style = style;
    }

    pub fn modify_default_style(&mut self, modifier: &StyleModifier) {
        modifier.modify(&mut self.default_style);
    }

    pub fn default_style(&self) -> &Style {
        &self.default_style
    }
}

impl<'a> CursorTarget for Window<'a> {
    fn get_width(&self) -> Width {
        self.get_width()
    }
    fn get_height(&self) -> Height {
        self.get_height()
    }
    fn get_cell_mut(&mut self, x: ColIndex, y: RowIndex) -> Option<&mut StyledGraphemeCluster> {
        if x < 0 || y < 0 {
            None
        } else {
            let x: isize = x.into();
            let y: isize = y.into();
            self.values.get_mut((y as usize, x as usize))
        }
    }
    fn get_cell(&self, x: ColIndex, y: RowIndex) -> Option<&StyledGraphemeCluster> {
        if x < 0 || y < 0 {
            None
        } else {
            let x: isize = x.into();
            let y: isize = y.into();
            self.values.get((y as usize, x as usize))
        }
    }
    fn get_default_style(&self) -> &Style {
        &self.default_style
    }
}


pub struct ExtentEstimationWindow {
    some_value: StyledGraphemeCluster,
    default_style: Style,
    width: Width,
    extent_x: Width,
    extent_y: Height,
}

//FIXME: compile time evaluation, see https://github.com/rust-lang/rust/issues/24111
//pub const UNBOUNDED_WIDTH: Width = Width::new(2147483647).unwrap();//i32::max_value() as u32;
//pub const UNBOUNDED_HEIGHT: Height = Height::new(2147483647).unwrap();//i32::max_value() as u32;
pub const UNBOUNDED_WIDTH: i32 = 2147483647;//i32::max_value() as u32;
pub const UNBOUNDED_HEIGHT: i32 = 2147483647;//i32::max_value() as u32;

impl ExtentEstimationWindow {
    pub fn with_width(width: Width) -> Self {
        let style = Style::default();
        ExtentEstimationWindow {
            some_value: StyledGraphemeCluster::new(GraphemeCluster::space().into(), style),
            default_style: style,
            width: width,
            extent_x: Width::new(0).unwrap(),
            extent_y: Height::new(0).unwrap(),
        }
    }

    pub fn unbounded() -> Self {
        Self::with_width(Width::new(UNBOUNDED_WIDTH).unwrap())
    }

    pub fn extent_x(&self) -> Width {
        self.extent_x
    }

    pub fn extent_y(&self) -> Height {
        self.extent_y
    }

    fn reset_value(&mut self) {
        self.some_value = StyledGraphemeCluster::new(GraphemeCluster::space().into(), self.default_style);
    }
}

impl CursorTarget for ExtentEstimationWindow {
    fn get_width(&self) -> Width {
        self.width
    }
    fn get_height(&self) -> Height {
        Height::new(UNBOUNDED_HEIGHT).unwrap()
    }
    fn get_cell_mut(&mut self, x: ColIndex, y: RowIndex) -> Option<&mut StyledGraphemeCluster> {
        self.extent_x = max(self.extent_x, (x.diff_to_origin()+1).positive_or_zero());
        self.extent_y = max(self.extent_y, (y.diff_to_origin()+1).positive_or_zero());
        self.reset_value();
        if x < self.width.from_origin() {
            Some(&mut self.some_value)
        } else {
            None
        }
    }

    fn get_cell(&self, x: ColIndex, _: RowIndex) -> Option<&StyledGraphemeCluster> {
        if x < self.width.from_origin() {
            Some(&self.some_value)
        } else {
            None
        }
    }
    fn get_default_style(&self) -> &Style {
        &self.default_style
    }
}
