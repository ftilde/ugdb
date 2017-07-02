use std::collections::BTreeMap;
use unsegen::widget::{
    RenderingHints,
};
use unsegen::base::{
    Cursor,
    CursorTarget,
    StyleModifier,
};

use json::{
    JsonValue,
};

use json::object::{
    Object,
};

use std::cmp::{
    min,
};

use super::path::*;

pub struct RenderingInfo {
    pub hints: RenderingHints,
    pub active_focused_style: StyleModifier,
    pub inactive_focused_style: StyleModifier,
}

impl RenderingInfo {
    fn get_focused_style(&self) -> StyleModifier {
        if self.hints.active {
            self.active_focused_style
        } else {
            self.inactive_focused_style
        }
    }
}

pub struct DisplayObject {
    pub members: BTreeMap<String, DisplayValue>,
    pub extended: bool,
}

static OPEN_SYMBOL: &'static str = "[+]";
static CLOSE_SYMBOL: &'static str = "[-]";

impl DisplayObject {
    pub fn toggle_visibility(&mut self) {
        self.extended ^= true;
    }

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
            extended: true,
        };
        for (key, value) in obj.iter() {
            result.members.insert(key.to_string(), DisplayValue::from_json(value));
        }
        result
    }

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, path: Option<&ObjectPath>, info: &RenderingInfo, indentation: u16) {
        use ::std::fmt::Write;
        if self.extended {

            {
                write!(cursor, "{{ ").unwrap();
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ObjectPath::Toggle) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "{}", CLOSE_SYMBOL).unwrap();
            }
            {
                let mut cursor = cursor.save().line_start_column();
                cursor.move_line_start_column(indentation as i32);
                for (key, value) in self.members.iter() {
                    cursor.wrap_line();
                    write!(cursor, "{}: ", key).unwrap();
                    let subpath = if let Some(&ObjectPath::Item(ref active_key, ref subpath)) = path {
                        if active_key == key {
                            Some(subpath.as_ref())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    value.draw(&mut cursor, subpath, info, indentation);
                    write!(cursor, ",").unwrap();
                }
            }
            write!(cursor, "\n}}").unwrap();
        } else {
            write!(cursor, "{{ ").unwrap();
            {
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ObjectPath::Toggle) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "{}", OPEN_SYMBOL).unwrap();
            }
            write!(cursor, " }}").unwrap();
        }
    }
}

pub struct DisplayArray {
    pub values: Vec<DisplayValue>,
    pub extended: bool,
    pub num_extended: usize,
}
impl DisplayArray {
    pub fn toggle_visibility(&mut self) {
        self.extended ^= true;
    }
    pub fn grow(&mut self) {
        self.num_extended += 1;
        assert!(self.num_extended <= self.values.len());
    }
    pub fn shrink(&mut self) {
        self.num_extended -= 1;
    }

    pub fn can_grow(&self) -> bool {
        self.num_extended < self.values.len()
    }

    pub fn can_shrink(&self) -> bool {
        self.num_extended > 0
    }

    fn replace(&self, values: &Vec<JsonValue>) -> Self {
        let mut result = DisplayArray {
            values: Vec::new(),
            extended: self.extended,
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
            extended: true,
            num_extended: min(3, values.len()),
        };
        for value in values {
            result.values.push(DisplayValue::from_json(value));
        }
        result
    }

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, path: Option<&ArrayPath>, info: &RenderingInfo, indentation: u16) {
        use ::std::fmt::Write;

        if self.extended {
            write!(cursor, " [").unwrap();
            {
                write!(cursor, " ").unwrap();
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ArrayPath::Toggle) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "{}", CLOSE_SYMBOL).unwrap();
            }
            {
                let mut cursor = cursor.save().line_start_column();
                cursor.move_line_start_column(indentation as i32);
                for (i, value) in self.values.iter().enumerate().take(self.num_extended) {
                    cursor.wrap_line();

                    let subpath = if let Some(&ArrayPath::Item(active_i, ref subpath)) = path {
                        if i == active_i {
                            Some(subpath.as_ref())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    value.draw(&mut cursor, subpath, info, indentation);
                    write!(cursor, ",",).unwrap();
                }
            }
            write!(cursor, "\n] <").unwrap();
            if self.can_shrink() {
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ArrayPath::Shrink) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "-").unwrap();
            } else {
                write!(cursor, " ").unwrap();
            }
            write!(cursor, "{}/{}", self.num_extended, self.values.len()).unwrap();
            if self.can_grow() {
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ArrayPath::Grow) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "+").unwrap();
            } else {
                write!(cursor, " ").unwrap();
            }
            write!(cursor, ">").unwrap();
        } else {
            write!(cursor, "[ ").unwrap();
            {
                let mut cursor = cursor.save().style_modifier();
                if let Some(&ArrayPath::Toggle) = path {
                    cursor.apply_style_modifier(info.get_focused_style());
                }
                write!(cursor, "{}", OPEN_SYMBOL).unwrap();
            }
            write!(cursor, " ]").unwrap();
        }
    }
}

//TODO: we may want to support other types, but I'm not sure if that is necessary
pub struct DisplayScalar {
    pub value: String,
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

    fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, active: bool, info: &RenderingInfo) {
        let mut cursor = cursor.save().style_modifier();
        if active {
            cursor.apply_style_modifier(info.get_focused_style());
        }
        cursor.write(&self.value);
    }
}

pub enum DisplayValue {
    Scalar(DisplayScalar),
    Object(DisplayObject),
    Array(DisplayArray),
}

impl DisplayValue {
    pub fn replace(&self, value: &JsonValue) -> Self {
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

    pub fn from_json(value: &JsonValue) -> Self {
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
    pub fn draw<T: CursorTarget>(&self, cursor: &mut Cursor<T>, path: Option<&Path>, info: &RenderingInfo, indentation: u16) {
        match (self, path) {
            (&DisplayValue::Scalar(ref scalar), Some(&Path::Scalar)) => scalar.draw(cursor, true, info),
            (&DisplayValue::Scalar(ref scalar), None) => scalar.draw(cursor, false, info),
            (&DisplayValue::Object(ref obj), Some(&Path::Object(ref op))) => obj.draw(cursor, Some(op), info, indentation),
            (&DisplayValue::Object(ref obj), None) => obj.draw(cursor, None, info, indentation),
            (&DisplayValue::Array(ref array), Some(&Path::Array(ref ap))) => array.draw(cursor, Some(ap), info, indentation),
            (&DisplayValue::Array(ref array), None) => array.draw(cursor, None, info, indentation),
            _ => panic!("Mismatched DisplayValue and path type!"),
        }
    }
}

#[cfg(test)]
impl DisplayValue {

    pub fn unwrap_scalar_ref(&self) -> &DisplayScalar {
        if let &DisplayValue::Scalar(ref val) = self {
            val
        } else {
            panic!("Tried to unwrap non-scalar DisplayValue");
        }
    }

    pub fn unwrap_object_ref(&self) -> &DisplayObject {
        if let &DisplayValue::Object(ref val) = self {
            val
        } else {
            panic!("Tried to unwrap non-object DisplayValue");
        }
    }
    pub fn unwrap_object_ref_mut(&mut self) -> &mut DisplayObject {
        if let &mut DisplayValue::Object(ref mut val) = self {
            val
        } else {
            panic!("Tried to unwrap non-object DisplayValue");
        }
    }

    pub fn unwrap_array_ref(&self) -> &DisplayArray {
        if let &DisplayValue::Array(ref val) = self {
            val
        } else {
            panic!("Tried to unwrap non-array DisplayValue");
        }
    }
    pub fn unwrap_array_ref_mut(&mut self) -> &mut DisplayArray {
        if let &mut DisplayValue::Array(ref mut val) = self {
            val
        } else {
            panic!("Tried to unwrap non-array DisplayValue");
        }
    }
}
