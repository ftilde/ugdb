//extern crate json; #TODO, actually add dependency and use json when network is available

use super::super::{
    Demand,
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
    fn space_demand(&self) -> (Demand, Demand) {
        (Demand::at_least(count_grapheme_clusters(&self.text)), Demand::exact(1))
    }
    fn draw(&mut self, mut window: Window) {
        let mut cursor = Cursor::new(&mut window);
        cursor.write(&self.text);
    }
}
