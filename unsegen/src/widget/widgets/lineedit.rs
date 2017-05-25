use super::super::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use base::{
    Cursor,
    StyleModifier,
    Window,
};
use super::{
    count_grapheme_clusters,
};
use input::{
    Editable,
    Navigatable,
    Writable,
    OperationResult,
};
use unicode_segmentation::UnicodeSegmentation;

pub struct LineEdit {
    text: String,
    cursor_pos: usize,
    cursor_style_active: StyleModifier,
    cursor_style_inactive: StyleModifier,
}

impl LineEdit {
    pub fn new() -> Self {
        Self::with_cursor_styles(StyleModifier::new().invert(), StyleModifier::new().underline(true))
    }

    pub fn with_cursor_styles(active: StyleModifier, inactive: StyleModifier) -> Self {
        LineEdit {
            text: String::new(),
            cursor_pos: 0,
            cursor_style_active: active,
            cursor_style_inactive: inactive,
        }
    }

    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: &str) {
        self.text = text.to_owned();
        self.move_cursor_to_end_of_line();
    }

    pub fn move_cursor_to_end_of_line(&mut self) {
        self.cursor_pos = count_grapheme_clusters(&self.text) as usize;
    }

    pub fn move_cursor_to_beginning_of_line(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_right(&mut self) -> Result<(), ()> {
        let new_pos = self.cursor_pos + 1;
        if new_pos <= count_grapheme_clusters(&self.text) as usize {
            self.cursor_pos = new_pos;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn move_cursor_left(&mut self) -> Result<(), ()> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn insert(&mut self, text: &str) {
        self.text = {
            let grapheme_iter = self.text.graphemes(true);
            grapheme_iter.clone().take(self.cursor_pos)
                .chain(Some(text))
                .chain(grapheme_iter.skip(self.cursor_pos))
                .collect()
        };
    }

    fn erase_symbol_at(&mut self, pos: usize) -> Result<(),()> {
        if pos < self.text.len() {
            self.text = self.text.graphemes(true).enumerate().filter_map(
                |(i, s)|  if i != pos {
                    Some(s)
                } else {
                    None
                }
                ).collect();
            Ok(())
        } else {
            Err(())
        }
    }


}

impl Widget for LineEdit {
    fn space_demand(&self) -> Demand2D {
        Demand2D {
            width: Demand::at_least((count_grapheme_clusters(&self.text) + 1) as u32),
            height: Demand::exact(1), //TODO this is not really universal
        }
    }
    fn draw(&mut self, mut window: Window, hints: RenderingHints) {
        let (maybe_cursor_pos_offset, maybe_after_cursor_offset) = {
            let mut grapheme_indices = self.text.grapheme_indices(true);
            let cursor_cluster = grapheme_indices.nth(self.cursor_pos as usize);
            let next_cluster = grapheme_indices.next();
            (cursor_cluster.map(|c: (usize, &str)| c.0), next_cluster.map(|c: (usize, &str)| c.0))
        };
        let num_graphemes = count_grapheme_clusters(&self.text);
        let right_padding = 1;
        let cursor_start_pos = ::std::cmp::min(0, window.get_width() as i32 - num_graphemes as i32 - right_padding);

        let cursor_style = if hints.active {
            self.cursor_style_active
        } else {
            self.cursor_style_inactive
        };

        let mut cursor = Cursor::new(&mut window).position(cursor_start_pos, 0);
        if let Some(cursor_pos_offset) = maybe_cursor_pos_offset {
            let (until_cursor, from_cursor) = self.text.split_at(cursor_pos_offset);
            cursor.write(until_cursor);
            if let Some(after_cursor_offset) = maybe_after_cursor_offset {
                let (cursor_str, after_cursor) = from_cursor.split_at(after_cursor_offset - cursor_pos_offset);
                {
                    let mut cursor = cursor.push_style(cursor_style);
                    cursor.write(cursor_str);
                }
                cursor.write(after_cursor);
            } else {
                let mut cursor = cursor.push_style(cursor_style);
                cursor.write(from_cursor);
            }
        } else {
            cursor.write(&self.text);
            {
                let mut cursor = cursor.push_style(cursor_style);
                cursor.write(" ");
            }
        }
    }
}

impl Navigatable for LineEdit {
    fn move_up(&mut self) -> OperationResult {
        Err(())
    }
    fn move_down(&mut self) -> OperationResult {
        Err(())
    }
    fn move_left(&mut self) -> OperationResult {
        self.move_cursor_left()
    }
    fn move_right(&mut self) -> OperationResult {
        self.move_cursor_right()
    }
}

impl Writable for LineEdit {
    fn write(&mut self, c: char) -> OperationResult {
        self.insert(&c.to_string());
        let _ = self.move_cursor_right();
        Ok(())
    }
}

impl Editable for LineEdit {
    fn delete_symbol(&mut self) -> OperationResult { //i.e., "del" key
        let to_erase = self.cursor_pos;
        self.erase_symbol_at(to_erase)
    }
    fn remove_symbol(&mut self) -> OperationResult { //i.e., "backspace"
        if self.cursor_pos > 0 {
            let to_erase = self.cursor_pos - 1;
            let _ = self.erase_symbol_at(to_erase);
            let _ = self.move_cursor_left();
            Ok(())
        } else {
            Err(())
        }
    }
    fn clear(&mut self) -> OperationResult {
        if self.text.is_empty() {
            Err(())
        } else {
            self.text.clear();
            self.cursor_pos = 0;
            Ok(())
        }
    }
}
