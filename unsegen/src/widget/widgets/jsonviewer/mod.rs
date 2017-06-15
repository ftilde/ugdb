//#[macro_use]
//extern crate json;

use super::super::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use base::{
    Cursor,
    ExtentEstimationWindow,
    Window,
};

use input::{
    Scrollable,
    OperationResult,
};

pub use json as json_ext;

use json::{
    JsonValue,
};

mod path;
mod displayvalue;

use self::path::*;
use self::displayvalue::*;

pub struct JsonViewer {
    value: DisplayValue,
    active_element: Path,
    indentation: u16,
}

impl JsonViewer {
    pub fn new(value: &JsonValue) -> Self {
        let mut res = JsonViewer {
            value: DisplayValue::from_json(&value),
            active_element: Path::Scalar, //Will be fixed ...
            indentation: 2,
        };
        res.fix_active_element_path(); //... here!
        res
    }

    pub fn reset(&mut self, value: &JsonValue) {
        self.value = DisplayValue::from_json(value);
        self.fix_active_element_path();
    }

    pub fn replace(&mut self, value: &JsonValue) {
        self.value = self.value.replace(value);
        self.fix_active_element_path();
    }

    pub fn select_next(&mut self) -> Result<(),()> {
        if let Some(new_path) = find_next_path(self.active_element.clone(), &self.value) {
            self.active_element = new_path;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn select_previous(&mut self) -> Result<(),()> {
        if let Some(new_path) = find_previous_path(self.active_element.clone(), &self.value) {
            self.active_element = new_path;
            Ok(())
        } else {
            Err(())
        }
    }

    fn fix_active_element_path(&mut self) {
        let mut tmp = Path::Scalar;
        ::std::mem::swap(&mut self.active_element, &mut tmp);
        self.active_element = fix_path_for_value(tmp, &self.value)
    }
}

impl Widget for JsonViewer {
    fn space_demand(&self) -> Demand2D {
        let mut window = ExtentEstimationWindow::unbounded();
        //TODO: We may want to consider passing hints to space_demand as well for an accurate estimate
        let hints = RenderingHints::default();
        {
            let mut cursor = Cursor::<ExtentEstimationWindow>::new(&mut window);
            self.value.draw(&mut cursor, Some(&self.active_element), hints, self.indentation);
        }
        Demand2D {
            width: Demand::at_least(window.extent_x()),
            height: Demand::exact(window.extent_y()),
        }
    }
    fn draw(&mut self, mut window: Window, hints: RenderingHints) {
        let mut cursor = Cursor::new(&mut window);
        self.value.draw(&mut cursor, Some(&self.active_element), hints, self.indentation);
    }
}

impl Scrollable for JsonViewer {
    fn scroll_forwards(&mut self) -> OperationResult {
        self.select_next()
    }
    fn scroll_backwards(&mut self) -> OperationResult {
        self.select_previous()
    }
}
