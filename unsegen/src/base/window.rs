use super::{
    FormattedChar,
    Style,
    StyleModifier,
};
use ndarray::{
    ArrayViewMut,
    Axis,
    Ix,
    Ix2,
};
use std::cmp::max;
use std::borrow::Cow;
use base::ranges::{
    Bound,
    RangeArgument,
};
use ::unicode_segmentation::UnicodeSegmentation;

type CharMatrixView<'w> = ArrayViewMut<'w, FormattedChar, Ix2>;
pub struct Window<'w> {
    values: CharMatrixView<'w>,
    default_style: Style,
}


impl<'w> Window<'w> {
    pub fn new(values: CharMatrixView<'w>, default_style: Style) -> Self {
        Window {
            values: values,
            default_style: default_style,
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

    pub fn split_v(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_height(), "Invalid split_pos");

        let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos as Ix);
        let w_u = Window {
            values: first_mat,
            default_style: self.default_style,
        };
        let w_d = Window {
            values: second_mat,
            default_style: self.default_style,
        };
        (w_u, w_d)
    }

    pub fn split_h(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_width(), "Invalid split_pos");

        let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos as Ix);
        let w_l = Window {
            values: first_mat,
            default_style: self.default_style,
        };
        let w_r = Window {
            values: second_mat,
            default_style: self.default_style,
        };
        (w_l, w_r)
    }

    pub fn fill(&mut self, c: char) {
        let mut line = String::with_capacity(self.get_width() as usize);
        for _ in 0..self.get_width() {
            line.push(c);
        }
        let height = self.get_height();
        let mut cursor = Cursor::new(self);
        for _ in 0..height {
            cursor.writeln(&line);
        }
    }

    pub fn set_default_style(&mut self, style: Style) {
        self.default_style = style;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrappingDirection {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrappingMode {
    Wrap,
    NoWrap,
}


pub struct Cursor<'c, 'w: 'c> {
    window: &'c mut Window<'w>,
    wrapping_direction: WrappingDirection,
    wrapping_mode: WrappingMode,
    style_modifier: StyleModifier,
    x: i32,
    y: i32,
    tab_column_width: usize,
}

impl<'c, 'w> Cursor<'c, 'w> {
    pub fn new(window: &'c mut Window<'w>) -> Self {
        Cursor {
            window: window,
            wrapping_direction: WrappingDirection::Down,
            wrapping_mode: WrappingMode::NoWrap,
            style_modifier: StyleModifier::none(),
            x: 0,
            y: 0,
            tab_column_width: 4,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.set_position(x, y);
        self
    }

    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_wrapping_direction(&mut self, wrapping_direction: WrappingDirection) {
        self.wrapping_direction = wrapping_direction;
    }

    pub fn wrapping_direction(mut self, wrapping_direction: WrappingDirection) -> Self {
        self.set_wrapping_direction(wrapping_direction);
        self
    }

    pub fn set_wrapping_mode(&mut self, wm: WrappingMode) {
        self.wrapping_mode = wm;
    }

    pub fn wrapping_mode(mut self, wm: WrappingMode) -> Self {
        self.set_wrapping_mode(wm);
        self
    }

    pub fn set_style_modifier(&mut self, style_modifier: StyleModifier) {
        self.style_modifier = style_modifier;
    }

    pub fn fill_and_wrap_line(&mut self) {
        while self.x < self.window.get_width() as i32 {
            self.write(" ");
        }
        self.wrap_line();
    }

    pub fn wrap_line(&mut self) {
        match self.wrapping_direction {
            WrappingDirection::Down => {
                self.y += 1;
            },
            WrappingDirection::Up => {
                self.y -= 1;
            },
        }
        self.x = 0;
    }

    fn write_grapheme_cluster_unchecked(&mut self, cluster: FormattedChar) {
        *self.window.values.get_mut((self.y as Ix, self.x as Ix)).expect("in bounds") = cluster;
    }

    fn active_style(&self) -> Style {
        self.style_modifier.apply(&self.window.default_style)
    }

    pub fn num_expected_wraps(&self, line: &str) -> u32 {
        if self.wrapping_mode == WrappingMode::Wrap {
            let num_chars = line.graphemes(true).count();
            max(0, ((num_chars as i32 + self.x) / (self.window.get_width() as i32)) as u32)
        } else {
            0
        }
    }

    fn current_cluster_width(&self, grapheme_cluster: &str) -> usize {
        match grapheme_cluster {
            "\t" => self.tab_column_width - ((self.x as usize) % self.tab_column_width),
            g => ::unicode_width::UnicodeWidthStr::width(g),
        }
    }

    pub fn write(&mut self, text: &str) {
        if self.window.get_width() == 0 || self.window.get_height() == 0 {
            return;
        }

        let mut line_it = text.lines().peekable();
        while let Some(line) = line_it.next() {
            let num_auto_wraps = self.num_expected_wraps(line) as i32;

            if self.wrapping_direction == WrappingDirection::Up {
                self.y -= num_auto_wraps; // reserve space for auto wraps
            }
            for grapheme_cluster_ref in ::unicode_segmentation::UnicodeSegmentation::graphemes(line, true) {
                let grapheme_cluster = if grapheme_cluster_ref == "\t" {
                    use std::iter::FromIterator;
                    let width = self.tab_column_width - ((self.x as usize) % self.tab_column_width);
                    Cow::Owned(String::from_iter(::std::iter::repeat(" ").take(width)))
                } else {
                    Cow::Borrowed(grapheme_cluster_ref)
                };
                if self.wrapping_mode == WrappingMode::Wrap && (self.x as u32) >= self.window.get_width() {
                    self.y += 1;
                    self.x = 0;
                }
                if     0 <= self.x && (self.x as u32) < self.window.get_width()
                    && 0 <= self.y && (self.y as u32) < self.window.get_height() {

                    let style = self.active_style();
                    self.write_grapheme_cluster_unchecked(FormattedChar::new(grapheme_cluster.as_ref(), style));
                }
                let cluster_width = self.current_cluster_width(grapheme_cluster.as_ref());
                self.x += 1;
                if cluster_width > 1 && 0 <= self.y && (self.y as u32) < self.window.get_height() {
                    let style = self.active_style();
                    for _ in 1..cluster_width {
                        if 0 <= self.x && (self.x as u32) < self.window.get_width() {
                            self.write_grapheme_cluster_unchecked(FormattedChar::new("", style.clone()));
                        }
                        self.x += 1;
                    }
                }
            }
            if self.wrapping_direction == WrappingDirection::Up {
                self.y -= num_auto_wraps; // Jump back to first line
            }
            if line_it.peek().is_some() {
                self.wrap_line();
            }
        }
    }

    pub fn writeln(&mut self, text: &str) {
        self.write(text);
        self.wrap_line();
    }

}

impl<'c, 'w> ::std::fmt::Write for Cursor<'c, 'w> {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        self.write(s);
        Ok(())
    }
}
