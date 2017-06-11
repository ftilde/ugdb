extern crate json;

use super::super::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use base::{
    Cursor,
    CursorTarget,
    ExtentEstimationWindow,
    Window,
};

use std::cmp::{
    min,
};

use std::collections::BTreeMap;

pub use self::json as json_ext;

use self::json::{
    JsonValue,
};

use self::json::object::{
    Object,
};

struct DisplayObject {
    members: BTreeMap<String, DisplayValue>,
    extended: bool,
}

static OPEN_SYMBOL: &'static str = "[+]";
static CLOSE_SYMBOL: &'static str = "[-]";

impl DisplayObject {
    fn replace(&self, obj: &Object) -> Self {
        let mut result = DisplayObject {
            members: BTreeMap::new(),
            extended: self.extended,
        };
        for (key, value) in obj.iter() {
            let new_value = if let Some(old_val) = self.members.get(key) {
                old_val.replace(value)
            } else {
                DisplayValue::from_json(value)
            };
            result.members.insert(key.to_string(), new_value);
        }
        result
    }

    fn from_json(obj: &Object) -> Self {
        let mut result = DisplayObject {
            members: BTreeMap::new(),
            extended: true, //TODO: change default to false
        };
        for (key, value) in obj.iter() {
            result.members.insert(key.to_string(), DisplayValue::from_json(value));
        }
        result
    }

    /*
    fn space_demand(&self) -> Demand2D {
        let mut height = Demand::zero();
        let mut width = Demand::zero();
        let assignment_width = Demand::exact(count_grapheme_clusters(": "));
        if self.extended {
            for (key, value) in self.members {
                d2 = value.space_demand();
                width = max(width, count_grapheme_clusters(key) + assignment_width + d2.width);
                height += d2.height;
            }
            height += Demand::exact(2);
            width += Demand::exact(INDENTATION);
            width = max(width, Demand::exact(count_grapheme_clusters("{ ")) + CLOSE_SYMBOL_LEN); // "{ " + CLOSE_SYMBOL
        } else {
            height = Demand::exact(1);
            width = Demand::exact(OPEN_SYMBOL_LEN);
        }
        Demand2D {
            width: width,
            height: width,
        }
    }
    */

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, hints: RenderingHints) {
        use ::std::fmt::Write;
        if self.extended {
            writeln!(cursor, "{{ {}", CLOSE_SYMBOL).unwrap();
            for (key, value) in self.members.iter() {
                write!(cursor, "{}: ", key).unwrap();
                value.draw(cursor, hints);
                writeln!(cursor, ",").unwrap();
            }
            write!(cursor, "}}").unwrap();
        } else {
            write!(cursor, "{{ {} }}", OPEN_SYMBOL).unwrap();
        }
    }
}

struct DisplayArray {
    values: Vec<DisplayValue>,
    num_extended: usize,
}
impl DisplayArray {
    fn replace(&self, values: &Vec<JsonValue>) -> Self {
        let mut result = DisplayArray {
            values: Vec::new(),
            num_extended: min(self.num_extended, values.len()),
        };

        let num_old_values = self.values.len();
        for (value, old_val) in values[..num_old_values].iter().zip(self.values.iter()) {
            result.values.push(old_val.replace(value));
        }
        for value in values[num_old_values..].iter() {
            result.values.push(DisplayValue::from_json(value));
        }
        result
    }

    fn from_json(values: &Vec<JsonValue>) -> Self {
        let mut result = DisplayArray {
            values: Vec::new(),
            num_extended: 3,
        };
        for value in values {
            result.values.push(DisplayValue::from_json(value));
        }
        result
    }

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, hints: RenderingHints) {
        use ::std::fmt::Write;
        //TODO: support open/close/num_extended
        writeln!(cursor, "[ {}", CLOSE_SYMBOL).unwrap();
        for value in self.values.iter() {
            value.draw(cursor, hints);
            writeln!(cursor, ",",).unwrap();
        }
        write!(cursor, "]").unwrap();
    }
}

//TODO: we may want to support other types, but I'm not sure if that is necessary
struct DisplayScalar {
    value: String,
}
impl DisplayScalar {
    fn replace<S: ToString>(&self, value: &S) -> Self {
        DisplayScalar {
            value: value.to_string()
        }
    }

    fn from_json<S: ToString>(value: &S) -> Self {
        DisplayScalar {
            value: value.to_string()
        }
    }

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, _: RenderingHints) {
        cursor.write(&self.value);
    }
}

enum DisplayValue {
    Scalar(DisplayScalar),
    Object(DisplayObject),
    Array(DisplayArray),
}

impl DisplayValue {
    fn replace(&self, value: &JsonValue) -> Self {
        match (self, value) {
            (&DisplayValue::Scalar(ref scalar), &JsonValue::Null)             => DisplayValue::Scalar(scalar.replace(&JsonValue::Null)),
            (&DisplayValue::Scalar(ref scalar), &JsonValue::Short(ref val))   => DisplayValue::Scalar(scalar.replace(&val)),
            (&DisplayValue::Scalar(ref scalar), &JsonValue::String(ref val))  => DisplayValue::Scalar(scalar.replace(&val)),
            (&DisplayValue::Scalar(ref scalar), &JsonValue::Number(ref val))  => DisplayValue::Scalar(scalar.replace(&val)),
            (&DisplayValue::Scalar(ref scalar), &JsonValue::Boolean(ref val)) => DisplayValue::Scalar(scalar.replace(&val)),
            (&DisplayValue::Object(ref obj),    &JsonValue::Object(ref val))  => DisplayValue::Object(obj.replace(&val)),
            (&DisplayValue::Array(ref array),   &JsonValue::Array(ref val))   => DisplayValue::Array(array.replace(&val)),
            (_,                                 val)                          => Self::from_json(val),
        }
    }

    fn from_json(value: &JsonValue) -> Self {
        match value {
            &JsonValue::Null             => DisplayValue::Scalar(DisplayScalar::from_json(&JsonValue::Null)),
            &JsonValue::Short(ref val)   => DisplayValue::Scalar(DisplayScalar::from_json(&val)),
            &JsonValue::String(ref val)  => DisplayValue::Scalar(DisplayScalar::from_json(&val)),
            &JsonValue::Number(ref val)  => DisplayValue::Scalar(DisplayScalar::from_json(&val)),
            &JsonValue::Boolean(ref val) => DisplayValue::Scalar(DisplayScalar::from_json(&val)),
            &JsonValue::Object(ref val)  => DisplayValue::Object(DisplayObject::from_json(&val)),
            &JsonValue::Array(ref val)   => DisplayValue::Array(DisplayArray::from_json(&val)),
        }
    }
    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, hints: RenderingHints) {
        match self {
            &DisplayValue::Scalar(ref scalar) => scalar.draw(cursor, hints),
            &DisplayValue::Object(ref obj)    => obj.draw(cursor, hints),
            &DisplayValue::Array(ref array)   => array.draw(cursor, hints),
        }
    }
}

pub struct JsonViewer {
    value: DisplayValue,
    indentation: u16,
}

impl JsonViewer {
    pub fn new(value: &JsonValue) -> Self {
        JsonViewer {
            value: DisplayValue::from_json(&value),
            indentation: 2,
        }
    }

    pub fn reset(&mut self, value: &JsonValue) {
        self.value = DisplayValue::from_json(value);
    }

    pub fn replace(&mut self, value: &JsonValue) {
        self.value = self.value.replace(value);
    }
}

impl Widget for JsonViewer {
    fn space_demand(&self) -> Demand2D {
        let mut window = ExtentEstimationWindow::unbounded();
        //TODO: We may want to consider passing hints to space_demand as well for an accurate estimate
        let hints = RenderingHints::default();
        {
            let mut cursor = Cursor::<ExtentEstimationWindow>::new(&mut window);
            self.value.draw(&mut cursor, hints);
        }
        Demand2D {
            width: Demand::at_least(window.extent_x()),
            height: Demand::exact(window.extent_y()),
        }
    }
    fn draw(&mut self, mut window: Window, hints: RenderingHints) {
        let mut cursor = Cursor::new(&mut window);
        self.value.draw(&mut cursor, hints);
    }
}
