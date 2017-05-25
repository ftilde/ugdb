//extern crate json; #TODO, actually add dependency and use json when network is available

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

pub struct JsonViewer {
    text: String,
}
impl JsonViewer {
    pub fn new<S: Into<String>>(text: S) -> Self {
        JsonViewer {
            text: text.into(),
        }
    }

    pub fn set<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
}

impl Widget for JsonViewer {
    fn space_demand(&self) -> Demand2D {
        Demand2D {
            width: Demand::at_least(count_grapheme_clusters(&self.text)),
            height: Demand::exact(1),
        }
    }
    fn draw(&mut self, mut window: Window, _: RenderingHints) {
        let mut cursor = Cursor::new(&mut window);
        cursor.write(&self.text);
    }
}
