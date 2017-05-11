use super::super::{
    Demand,
    Widget,
};
use base::{
    Cursor,
    Window,
    StyleModifier,
};
use super::{
    count_grapheme_clusters,
};
use input::{
    Editable,
    Navigatable,
    Writable,
};
use unicode_segmentation::UnicodeSegmentation;

pub struct LineEdit {
    text: String,
    cursor_pos: usize,
    cursor_style: StyleModifier,
}

impl LineEdit {
    pub fn new() -> Self {
        LineEdit {
            text: String::new(),
            cursor_pos: 0,
            cursor_style: StyleModifier::new().invert(),
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

    pub fn move_cursor_right(&mut self) {
        self.cursor_pos = ::std::cmp::min(self.cursor_pos + 1, count_grapheme_clusters(&self.text) as usize);
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
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
        self.move_cursor_right();
    }

    fn erase_symbol_at(&mut self, pos: usize) {
        self.text = self.text.graphemes(true).enumerate().filter_map(
                |(i, s)|  if i != pos {
                    Some(s)
                } else {
                    None
                }
            ).collect();
    }


}

impl Widget for LineEdit {
    fn space_demand(&self) -> (Demand, Demand) {
        (Demand::at_least((count_grapheme_clusters(&self.text) + 1) as u32), Demand::exact(1)) //TODO this is not really universal
    }
    fn draw(&mut self, mut window: Window) {
        let (maybe_cursor_pos_offset, maybe_after_cursor_offset) = {
            let mut grapheme_indices = self.text.grapheme_indices(true);
            let cursor_cluster = grapheme_indices.nth(self.cursor_pos as usize);
            let next_cluster = grapheme_indices.next();
            (cursor_cluster.map(|c: (usize, &str)| c.0), next_cluster.map(|c: (usize, &str)| c.0))
        };
        let num_graphemes = count_grapheme_clusters(&self.text);
        let right_padding = 2;
        let cursor_start_pos = ::std::cmp::min(0, window.get_width() as i32 - num_graphemes as i32 - right_padding);

        let mut cursor = Cursor::new(&mut window).position(cursor_start_pos, 0);
        if let Some(cursor_pos_offset) = maybe_cursor_pos_offset {
            let (until_cursor, from_cursor) = self.text.split_at(cursor_pos_offset);
            cursor.write(until_cursor);
            if let Some(after_cursor_offset) = maybe_after_cursor_offset {
                let (cursor_str, after_cursor) = from_cursor.split_at(after_cursor_offset - cursor_pos_offset);
                {
                    let mut cursor = cursor.push_style(self.cursor_style);
                    cursor.write(cursor_str);
                }
                cursor.write(after_cursor);
            } else {
                let mut cursor = cursor.push_style(self.cursor_style);
                cursor.write(from_cursor);
            }
        } else {
            cursor.write(&self.text);
            {
                let mut cursor = cursor.push_style(self.cursor_style);
                cursor.write(" ");
            }
        }
    }
}

impl Navigatable for LineEdit {
    fn move_up(&mut self) {
    }
    fn move_down(&mut self) {
    }
    fn move_left(&mut self) {
        self.move_cursor_left();
    }
    fn move_right(&mut self) {
        self.move_cursor_right();
    }
}

impl Writable for LineEdit {
    fn write(&mut self, c: char) {
        self.insert(&c.to_string());
    }
}

impl Editable for LineEdit {
    fn delete_symbol(&mut self) { //i.e., "del" key
        let to_erase = self.cursor_pos;
        self.erase_symbol_at(to_erase);
    }
    fn remove_symbol(&mut self) { //i.e., "backspace"
        if self.cursor_pos > 0 {
            let to_erase = self.cursor_pos - 1;
            self.erase_symbol_at(to_erase);
            self.move_cursor_left();
        }
    }
    fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }
}
