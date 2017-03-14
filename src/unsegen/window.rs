use super::{
    FormattedChar,
    TextAttribute,
};
use ndarray::{
    ArrayViewMut,
    Axis,
    Ix
};
use std::cmp::max;
use ::unicode_segmentation::UnicodeSegmentation;

type CharMatrixView<'w> = ArrayViewMut<'w, FormattedChar, (Ix,Ix)>;
pub struct Window<'w> {
    pos_x: u32,
    pos_y: u32,
    values: CharMatrixView<'w>,
    default_format: TextAttribute,
}

impl<'w> Window<'w> {
    pub fn new(values: CharMatrixView<'w>, default_format: TextAttribute) -> Self {
        Window {
            pos_x: 0,
            pos_y: 0,
            values: values,
            default_format: default_format,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.values.dim().1 as u32
    }

    pub fn get_height(&self) -> u32 {
        self.values.dim().0 as u32
    }

    pub fn split_v(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_height(), "Invalid split_pos");
        //let split_pos = min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos as Ix);
        let w_u = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            values: first_mat,
            default_format: self.default_format,
        };
        let w_d = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y+split_pos,
            values: second_mat,
            default_format: self.default_format,
        };
        (w_u, w_d)
    }

    pub fn split_h(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_width(), "Invalid split_pos");
        //let split_pos = min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos as Ix);
        let w_l = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            values: first_mat,
            default_format: self.default_format,
        };
        let w_r = Window {
            pos_x: self.pos_x+split_pos,
            pos_y: self.pos_y,
            values: second_mat,
            default_format: self.default_format,
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

    pub fn set_default_format(&mut self, format: TextAttribute) {
        self.default_format = format;
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
    text_attribute: Option<TextAttribute>,
    x: i32,
    y: i32,
}

impl<'c, 'w> Cursor<'c, 'w> {
    pub fn new(window: &'c mut Window<'w>) -> Self {
        Cursor {
            window: window,
            wrapping_direction: WrappingDirection::Down,
            wrapping_mode: WrappingMode::NoWrap,
            text_attribute: None,
            x: 0,
            y: 0,
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

    pub fn set_text_attribute(&mut self, ta: TextAttribute) {
        self.text_attribute = Some(ta)
    }

    /*
    pub fn text_attribute(mut self, ta: TextAttribute) -> Self {
        self.set_text_attribute(ta);
        self
    }
    */
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

    fn active_text_attribute(&self) -> TextAttribute {
        if let Some(attr) = self.text_attribute {
            attr.or(&self.window.default_format)
        } else {
            self.window.default_format.clone()
        }
    }

    pub fn num_expected_wraps(&self, line: &str) -> u32 {
        if self.wrapping_mode == WrappingMode::Wrap {
            let num_chars = line.graphemes(true).count();
            max(0, ((num_chars as i32 + self.x) / (self.window.get_width() as i32)) as u32)
        } else {
            0
        }
    }

    pub fn write(&mut self, text: &str) {
        let mut line_it = text.lines().peekable();
        while let Some(line) = line_it.next() {
            let num_auto_wraps = self.num_expected_wraps(line) as i32;

            if self.wrapping_direction == WrappingDirection::Up {
                self.y -= num_auto_wraps; // reserve space for auto wraps
            }
            for grapheme_cluster in ::unicode_segmentation::UnicodeSegmentation::graphemes(line, true) {
                if self.wrapping_mode == WrappingMode::Wrap && (self.x as u32) >= self.window.get_width() {
                    self.y += 1;
                    self.x = 0;
                }
                if     0 <= self.x && (self.x as u32) < self.window.get_width()
                    && 0 <= self.y && (self.y as u32) < self.window.get_height() {

                    let text_attribute = self.active_text_attribute();
                    self.write_grapheme_cluster_unchecked(FormattedChar::new(grapheme_cluster, text_attribute));
                }
                self.x += 1;
                let cluster_width = ::unicode_width::UnicodeWidthStr::width(grapheme_cluster);
                if cluster_width > 1 {
                    let text_attribute = self.active_text_attribute();
                    for _ in 1..cluster_width {
                        if self.x >= self.window.get_width() as i32 {
                            break;
                        }
                        self.write_grapheme_cluster_unchecked(FormattedChar::new(grapheme_cluster, text_attribute.clone()));
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
