extern crate json;

use super::super::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use base::{
    Cursor,
    Window,
};
use super::{
    count_grapheme_clusters,
};

pub use self::json::{
    JsonValue,
};

pub struct JsonViewer {
    value: JsonValue,
    indentation: u16,
}

impl JsonViewer {
    pub fn new(value: JsonValue) -> Self {
        JsonViewer {
            value: value,
            indentation: 2,
        }
    }

    pub fn reset(&mut self, value: JsonValue) {
        self.value = value;
    }

    fn string_to_display(&self) -> String {
        self.value.pretty(self.indentation)
    }
}

impl Widget for JsonViewer {
    fn space_demand(&self) -> Demand2D {
        let mut height = 0;
        let mut width = 0;
        for line in self.string_to_display().lines() {
            width = ::std::cmp::max(width, count_grapheme_clusters(&line));
            height += 1;
        }
        Demand2D {
            width: Demand::at_least(width),
            height: Demand::exact(height),
        }
    }
    fn draw(&mut self, mut window: Window, _: RenderingHints) {
        let mut cursor = Cursor::new(&mut window);
        cursor.write(&self.string_to_display());
    }
}
