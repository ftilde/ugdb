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
use std::cmp::max;
use base::ranges::{
    Bound,
    RangeArgument,
};
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
    pub fn new(width: u32, height: u32) -> Self {
        WindowBuffer {
            storage: CharMatrix::default(Ix2(height as usize, width as usize)),
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
        write!(f, "Window {{ w: {}, h: {} }}", self.get_width(), self.get_height())
    }
}

impl<'w> Window<'w> {
    pub fn new(values: CharMatrixView<'w>) -> Self {
        Window {
            values: values,
            default_style: Style::default(),
        }
    }

    pub fn get_width(&self) -> u32 {
        self.values.dim().1 as u32
    }

    pub fn get_height(&self) -> u32 {
        self.values.dim().0 as u32
    }

    pub fn clone_mut<'a>(&'a mut self) -> Window<'a> {
        let mat_view_clone = self.values.view_mut();
        Window {
            values: mat_view_clone,
            default_style: self.default_style,
        }
    }

    pub fn create_subwindow<'a, WX: RangeArgument<u32>, WY: RangeArgument<u32>>(&'a mut self, x_range: WX, y_range: WY) -> Window<'a> {
        let x_range_start = match x_range.start() {
            Bound::Unbound => 0,
            Bound::Inclusive(i) => i,
            Bound::Exclusive(i) => i-1,
        };
        let x_range_end = match x_range.end() {
            Bound::Unbound => self.get_width(),
            Bound::Inclusive(i) => i-1,
            Bound::Exclusive(i) => i,
        };
        let y_range_start = match y_range.start() {
            Bound::Unbound => 0,
            Bound::Inclusive(i) => i,
            Bound::Exclusive(i) => i-1,
        };
        let y_range_end = match y_range.end() {
            Bound::Unbound => self.get_height(),
            Bound::Inclusive(i) => i-1,
            Bound::Exclusive(i) => i,
        };
        assert!(x_range_start <= x_range_end, "Invalid x_range: start > end");
        assert!(y_range_start <= y_range_end, "Invalid y_range: start > end");
        assert!(x_range_end <= self.get_width(), "Invalid x_range: end > width");
        assert!(y_range_end <= self.get_height(), "Invalid y_range: end > height");

        let sub_mat = self.values.slice_mut(s![y_range_start as isize..y_range_end as isize, x_range_start as isize..x_range_end as isize]);
        Window {
            values: sub_mat,
            default_style: self.default_style,
        }
    }

    pub fn split_v(self, split_pos: u32) -> Result<(Self, Self), Self> {
        if split_pos <= self.get_height() {
            let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos as Ix);
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

    pub fn split_h(self, split_pos: u32) -> Result<(Self, Self), Self> {
        if split_pos <= self.get_width() {

            let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos as Ix);
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
        let empty = StyledGraphemeCluster::new(unsafe {GraphemeCluster::empty()}, self.default_style);
        let space = StyledGraphemeCluster::new(GraphemeCluster::space(), self.default_style);
        let right_border = (self.get_width() - (self.get_width() % cluster_width as u32)) as usize;
        for ((_, x), cell) in self.values.indexed_iter_mut() {
            if x >= right_border {
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
    fn get_width(&self) -> u32 {
        self.get_width()
    }
    fn get_height(&self) -> u32 {
        self.get_height()
    }
    fn get_grapheme_cluster_mut(&mut self, x: u32, y: u32) -> Option<&mut StyledGraphemeCluster> {
        self.values.get_mut((y as usize, x as usize))
    }
    fn get_default_style(&self) -> &Style {
        &self.default_style
    }
}


pub struct ExtentEstimationWindow {
    some_value: StyledGraphemeCluster,
    default_style: Style,
    width: u32,
    extent_x: u32,
    extent_y: u32,
}

pub const UNBOUNDED_EXTENT: u32 = 2147483647;//i32::max_value() as u32;

impl ExtentEstimationWindow {
    pub fn with_width(width: u32) -> Self {
        let style = Style::default();
        ExtentEstimationWindow {
            some_value: StyledGraphemeCluster::new(GraphemeCluster::space().into(), style),
            default_style: style,
            width: width,
            extent_x: 0,
            extent_y: 0,
        }
    }

    pub fn unbounded() -> Self {
        Self::with_width(UNBOUNDED_EXTENT)
    }

    pub fn extent_x(&self) -> u32 {
        self.extent_x
    }

    pub fn extent_y(&self) -> u32 {
        self.extent_y
    }

    fn reset_value(&mut self) {
        self.some_value = StyledGraphemeCluster::new(GraphemeCluster::space().into(), self.default_style);
    }
}

impl CursorTarget for ExtentEstimationWindow {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        UNBOUNDED_EXTENT
    }
    fn get_grapheme_cluster_mut(&mut self, x: u32, y: u32) -> Option<&mut StyledGraphemeCluster> {
        self.extent_x = max(self.extent_x, x+1);
        self.extent_y = max(self.extent_y, y+1);
        self.reset_value();
        if x < self.width {
            Some(&mut self.some_value)
        } else {
            None
        }
    }
    fn get_default_style(&self) -> &Style {
        &self.default_style
    }
}
