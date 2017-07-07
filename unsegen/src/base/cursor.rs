use super::{
    StyledGraphemeCluster,
    Style,
    StyleModifier,
    GraphemeCluster,
    Window,
};
use std::cmp::max;
use unicode_segmentation::UnicodeSegmentation;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrappingMode {
    Wrap,
    NoWrap,
}

pub trait CursorTarget {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_grapheme_cluster_mut(&mut self, x: u32, y: u32) -> Option<&mut StyledGraphemeCluster>;
    fn get_default_style(&self) -> &Style;
}


pub struct Cursor<'c, 'g: 'c, T: 'c + CursorTarget = Window<'g>> {
    window: &'c mut T,
    _dummy: ::std::marker::PhantomData<&'g ()>,
    wrapping_mode: WrappingMode,
    style_modifier: StyleModifier,
    x: i32,
    y: i32,
    line_start_column: i32,
    tab_column_width: u32,
}

impl<'c, 'g: 'c, T: 'c + CursorTarget> Cursor<'c, 'g, T> {
    pub fn new(window: &'c mut T) -> Self {
        Cursor {
            window: window,
            _dummy: ::std::marker::PhantomData::default(),
            wrapping_mode: WrappingMode::NoWrap,
            style_modifier: StyleModifier::none(),
            x: 0,
            y: 0,
            line_start_column: 0,
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

    pub fn move_by(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }

    pub fn set_wrapping_mode(&mut self, wm: WrappingMode) {
        self.wrapping_mode = wm;
    }

    pub fn wrapping_mode(mut self, wm: WrappingMode) -> Self {
        self.set_wrapping_mode(wm);
        self
    }

    pub fn set_line_start_column(&mut self, column: i32) {
        self.line_start_column = column;
    }

    pub fn move_line_start_column(&mut self, d: i32) {
        self.line_start_column += d;
    }

    pub fn line_start_column(mut self, column: i32) -> Self {
        self.set_line_start_column(column);
        self
    }

    pub fn set_style_modifier(&mut self, style_modifier: StyleModifier) {
        self.style_modifier = style_modifier;
    }

    pub fn apply_style_modifier(&mut self, style_modifier: StyleModifier) {
        self.style_modifier = self.style_modifier.if_not(style_modifier)
    }

    pub fn set_tab_column_width(&mut self, width: u32) {
        self.tab_column_width = width;
    }

    pub fn fill_and_wrap_line(&mut self) {
        let w = self.window.get_width() as i32;
        while self.x < w {
            self.write(" ");
        }
        self.wrap_line();
    }

    pub fn wrap_line(&mut self) {
        self.y += 1;
        self.carriage_return();
    }

    pub fn carriage_return(&mut self) {
        self.x = self.line_start_column;
    }


    fn active_style(&self) -> Style {
        self.style_modifier.apply(self.window.get_default_style())
    }

    pub fn num_expected_wraps(&self, line: &str) -> u32 {
        if self.wrapping_mode == WrappingMode::Wrap {
            let num_chars = line.graphemes(true).count();
            max(0, ((num_chars as i32 + self.x) / (self.window.get_width() as i32)) as u32)
        } else {
            0
        }
    }

    fn create_tab_cluster(width: u32) -> GraphemeCluster {
        use std::iter::FromIterator;
        let tab_string = String::from_iter(::std::iter::repeat(" ").take(width as usize));
        unsafe {
            GraphemeCluster::from_str_unchecked(tab_string)
        }
    }

    fn write_grapheme_cluster_unchecked(&mut self, cluster: GraphemeCluster) {
        let style = self.active_style();
        let target_cluster_x = self.x as u32;
        let y = self.y as u32;
        let old_target_cluster_width = {
            let target_cluster = self.window.get_grapheme_cluster_mut(target_cluster_x, y).expect("in bounds");
            let w = target_cluster.grapheme_cluster.width() as u32;
            *target_cluster = StyledGraphemeCluster::new(cluster, style);
            w
        };
        if old_target_cluster_width != 1 {
            // Find start of wide cluster which will be (partially) overwritten
            let mut current_x = target_cluster_x;
            let mut current_width = old_target_cluster_width;
            while current_width == 0 {
                current_x -= 1;
                current_width = self.window.get_grapheme_cluster_mut(current_x, y).expect("finding wide cluster start: read in bounds").grapheme_cluster.width() as u32;
            }

            // Clear all cells (except the newly written one)
            let start_cluster_x = current_x;
            let start_cluster_width = current_width;
            for x_to_clear in start_cluster_x..start_cluster_x+start_cluster_width {
                if x_to_clear != target_cluster_x {
                    self.window.get_grapheme_cluster_mut(x_to_clear, y).expect("overwrite cluster cells in bounds").grapheme_cluster.clear();
                }
            }
        }
        // This should cover (almost) all cases where we overwrite wide grapheme clusters.
        // Unfortunately, with the current design it is possible to split windows exactly at a
        // multicell wide grapheme cluster, e.g.: [f,o,o,b,a,r] => [f,o,沐,,a,r] => [f,o,沐|,a,r]
        // Now, when writing to to [f,o,沐| will trigger an out of bound access
        // => "overwrite cluster cells in bounds" will fail
        //
        // Alternatively: writing to |,a,r] will cause an under/overflow in
        // current_x -= 1;
        //
        // I will call this good for now, as these problems will likely not (or only rarely) arrise
        // in pratice. If they do... we have to think of something...
    }

    fn remaining_space_in_line(&self) -> i32 {
        self.window.get_width() as i32 - self.x
    }

    pub fn write(&mut self, text: &str) {
        if self.window.get_width() == 0 || self.window.get_height() == 0 {
            return;
        }

        let mut line_it = text.split('\n').peekable(); //.lines() swallows a terminal newline
        while let Some(line) = line_it.next() {
            for mut grapheme_cluster in GraphemeCluster::all_from_str(line) {
                match grapheme_cluster.as_str() {
                    "\t" => {
                        let width = self.tab_column_width - ((self.x as u32) % self.tab_column_width);
                        grapheme_cluster = Self::create_tab_cluster(width)
                    },
                    "\r" => {
                        self.carriage_return();
                        continue;
                    },
                    _ => {},
                }
                let cluster_width = grapheme_cluster.width() as i32;

                let space_in_line = self.remaining_space_in_line();
                if space_in_line < cluster_width {
                    // Overwrite spaces that we could not fill with our (too wide) grapheme cluster
                    for _ in 0..space_in_line {
                        self.write_grapheme_cluster_unchecked(GraphemeCluster::space());
                        self.x += 1;
                    }
                    if self.wrapping_mode == WrappingMode::Wrap {
                        self.wrap_line();
                        if self.remaining_space_in_line() < cluster_width {
                            // Still no space for the cluster after line wrap: We have to give up.
                            // There is no way we can write our cluster anywhere.
                            break;
                        }
                    } else {
                        // We do not wrap, so we are outside of the window now
                        break;
                    }
                }
                if     0 <= self.x && (self.x as u32) < self.window.get_width()
                    && 0 <= self.y && (self.y as u32) < self.window.get_height() {

                    self.write_grapheme_cluster_unchecked(grapheme_cluster);
                }
                self.x += 1;
                // TODO: This still probably does not work if we _overwrite_ wide clusters.
                if cluster_width > 1 && 0 <= self.y && (self.y as u32) < self.window.get_height() {
                    for _ in 1..cluster_width {
                        if 0 <= self.x && (self.x as u32) < self.window.get_width() {
                            self.write_grapheme_cluster_unchecked(unsafe {
                                GraphemeCluster::empty()
                            });
                        }
                        self.x += 1;
                    }
                }
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

    pub fn save<'a>(&'a mut self) -> CursorRestorer<'a, 'c, 'g, T> {
        CursorRestorer::new(self)
    }
}

impl<'c, 'g: 'c, T: 'c + CursorTarget> ::std::fmt::Write for Cursor<'c, 'g, T> {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[must_use]
pub struct CursorRestorer<'a, 'c: 'a, 'g: 'c, T: 'c + CursorTarget> {
    cursor: &'a mut Cursor<'c, 'g, T>,
    saved_style_modifier: Option<StyleModifier>,
    saved_line_start_column: Option<i32>,
    saved_pos_x: Option<i32>,
    saved_pos_y: Option<i32>,
}

impl<'a, 'c: 'a, 'g: 'c, T: 'c + CursorTarget> CursorRestorer<'a, 'c, 'g, T> {
    pub fn new(cursor: &'a mut Cursor<'c, 'g, T>) -> Self {
        CursorRestorer {
            cursor: cursor,
            saved_style_modifier: None,
            saved_line_start_column: None,
            saved_pos_x: None,
            saved_pos_y: None,
        }
    }

    pub fn style_modifier(mut self) -> Self {
        self.saved_style_modifier = Some(self.cursor.style_modifier);
        self
    }

    pub fn line_start_column(mut self) -> Self {
        self.saved_line_start_column = Some(self.cursor.line_start_column);
        self
    }

    pub fn pos_x(mut self) -> Self {
        self.saved_pos_x = Some(self.cursor.x);
        self
    }

    pub fn pos_y(mut self) -> Self {
        self.saved_pos_y = Some(self.cursor.y);
        self
    }
}

impl<'a, 'c: 'a, 'g: 'c, T: 'c + CursorTarget> ::std::ops::Drop for CursorRestorer<'a, 'c, 'g, T> {
    fn drop(&mut self) {
        if let Some(saved) = self.saved_style_modifier {
            self.cursor.style_modifier = saved;
        }
        if let Some(saved) = self.saved_line_start_column {
            self.cursor.line_start_column = saved;
        }
        if let Some(saved) = self.saved_pos_x {
            self.cursor.x = saved;
        }
        if let Some(saved) = self.saved_pos_y {
            self.cursor.y = saved;
        }
    }
}

impl<'a, 'c: 'a, 'g: 'c, T: 'c + CursorTarget> ::std::ops::DerefMut for CursorRestorer<'a, 'c, 'g, T> {
    fn deref_mut(&mut self) -> &mut Cursor<'c, 'g, T> {
        &mut self.cursor
    }
}


impl<'a, 'c: 'a, 'g: 'c, T: 'c + CursorTarget> ::std::ops::Deref for CursorRestorer<'a, 'c, 'g, T> {
    type Target = Cursor<'c, 'g, T>;
    fn deref(&self) -> &Cursor<'c, 'g, T> {
        &self.cursor
    }
}

#[cfg(test)]
mod test {
    use base::test::FakeTerminal;
    use super::*;

    fn test_cursor<S: Fn(&mut Cursor), F: Fn(&mut Cursor)>(window_dim: (u32, u32), after: &str, setup: S, action: F) {
        let mut term = FakeTerminal::with_size(window_dim);
        {
            let mut window = term.create_root_window();
            window.fill(GraphemeCluster::try_from('_').unwrap());
            let mut cursor = Cursor::new(&mut window);
            setup(&mut cursor);
            action(&mut cursor);
        }
        term.assert_looks_like(after);
    }
    #[test]
    fn test_cursor_simple() {
        test_cursor((5, 1), "_____", |_| {}, |c| c.write(""));
        test_cursor((5, 1), "t____", |_| {}, |c| c.write("t"));
        test_cursor((5, 1), "te___", |_| {}, |c| c.write("te"));
        test_cursor((5, 1), "tes__", |_| {}, |c| c.write("tes"));
        test_cursor((5, 1), "test_", |_| {}, |c| c.write("test"));
        test_cursor((5, 1), "testy", |_| {}, |c| c.write("testy"));
    }

    #[test]
    fn test_cursor_no_wrap() {
        test_cursor((2, 2), "__|__", |_| {}, |c| c.write(""));
        test_cursor((2, 2), "t_|__", |_| {}, |c| c.write("t"));
        test_cursor((2, 2), "te|__", |_| {}, |c| c.write("te"));
        test_cursor((2, 2), "te|__", |_| {}, |c| c.write("tes"));
        test_cursor((2, 2), "te|__", |_| {}, |c| c.write("test"));
        test_cursor((2, 2), "te|__", |_| {}, |c| c.write("testy"));
    }

    #[test]
    fn test_cursor_wrap() {
        test_cursor((2, 2), "__|__", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write(""));
        test_cursor((2, 2), "t_|__", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("t"));
        test_cursor((2, 2), "te|__", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("te"));
        test_cursor((2, 2), "te|s_", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("tes"));
        test_cursor((2, 2), "te|st", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("test"));
        test_cursor((2, 2), "te|st", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("testy"));
    }

    #[test]
    fn test_cursor_tabs() {
        test_cursor((5, 1), "  x__", |c| c.set_tab_column_width(2), |c| c.write("\tx"));
        test_cursor((5, 1), "x x__", |c| c.set_tab_column_width(2), |c| c.write("x\tx"));
        test_cursor((5, 1), "xx  x", |c| c.set_tab_column_width(2), |c| c.write("xx\tx"));
        test_cursor((5, 1), "xxx x", |c| c.set_tab_column_width(2), |c| c.write("xxx\tx"));
        test_cursor((5, 1), "    x", |c| c.set_tab_column_width(2), |c| c.write("\t\tx"));
        test_cursor((5, 1), "     ", |c| c.set_tab_column_width(2), |c| c.write("\t\t\tx"));
    }

    #[test]
    fn test_cursor_wide_cluster() {
        test_cursor((5, 1), "沐___", |_| {}, |c| c.write("沐"));
        test_cursor((5, 1), "沐沐_", |_| {}, |c| c.write("沐沐"));
        test_cursor((5, 1), "沐沐 ", |_| {}, |c| c.write("沐沐沐"));

        test_cursor((3, 2), "沐_|___", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("沐"));
        test_cursor((3, 2), "沐 |沐_", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("沐沐"));
        test_cursor((3, 2), "沐 |沐 ", |c| c.set_wrapping_mode(WrappingMode::Wrap), |c| c.write("沐沐沐"));
    }

    #[test]
    fn test_cursor_wide_cluster_overwrite() {
        test_cursor((5, 1), "X ___", |_| {}, |c| { c.write("沐"); c.set_position(0,0); c.write("X"); });
        test_cursor((5, 1), " X___", |_| {}, |c| { c.write("沐"); c.set_position(1,0); c.write("X"); });
        test_cursor((5, 1), "XYZ _", |_| {}, |c| { c.write("沐沐"); c.set_position(0,0); c.write("XYZ"); });
        test_cursor((5, 1), " XYZ_", |_| {}, |c| { c.write("沐沐"); c.set_position(1,0); c.write("XYZ"); });
        test_cursor((5, 1), "沐XYZ", |_| {}, |c| { c.write("沐沐沐"); c.set_position(2,0); c.write("XYZ"); });
    }

    #[test]
    fn test_cursor_tabs_overwrite() {
        test_cursor((5, 1), "X   _", |c| c.set_tab_column_width(4), |c| { c.write("\t"); c.set_position(0,0); c.write("X"); });
        test_cursor((5, 1), " X  _", |c| c.set_tab_column_width(4), |c| { c.write("\t"); c.set_position(1,0); c.write("X"); });
        test_cursor((5, 1), "  X _", |c| c.set_tab_column_width(4), |c| { c.write("\t"); c.set_position(2,0); c.write("X"); });
        test_cursor((5, 1), "   X_", |c| c.set_tab_column_width(4), |c| { c.write("\t"); c.set_position(3,0); c.write("X"); });
    }
}
