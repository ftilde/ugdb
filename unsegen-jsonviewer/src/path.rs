use super::displayvalue::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ArrayPath {
    Item(usize, Box<Path>),
    Shrink,
    Toggle,
    Grow,
}
#[derive(Clone, PartialEq, Debug)]
pub enum ObjectPath {
    Item(String, Box<Path>),
    Toggle,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Path {
    Array(ArrayPath),
    Object(ObjectPath),
    Scalar,
}

impl Path {
    fn unwrap_array(self) -> ArrayPath {
        if let Path::Array(arr) = self {
            arr
        } else {
            panic!("Tried to unwrap array from other path");
        }
    }
    fn unwrap_object(self) -> ObjectPath {
        if let Path::Object(bo) = self {
            bo
        } else {
            panic!("Tried to unwrap obj from other path");
        }
    }

}

//Easily create paths (for debug purposes, as this is not exposed)
#[cfg(test)]
impl Path {
    fn scalar() -> Self {
        Path::Scalar
    }
    fn object_toggle() -> Self {
        Path::Object(ObjectPath::Toggle)
    }
    fn object<S: Into<String>>(self, key: S) -> Self {
        Path::Object(ObjectPath::Item(key.into(), Box::new(self)))
    }
    fn array_grow() -> Self {
        Path::Array(ArrayPath::Grow)
    }
    fn array_shrink() -> Self {
        Path::Array(ArrayPath::Shrink)
    }
    fn array_toggle() -> Self {
        Path::Array(ArrayPath::Toggle)
    }
    fn array(self, index: usize) -> Self {
        Path::Array(ArrayPath::Item(index, Box::new(self)))
    }
}

fn first_path_in(value: &DisplayValue) -> Path {
    match value {
        &DisplayValue::Array(_) => {
            Path::Array(ArrayPath::Toggle)
        },
        &DisplayValue::Object(_) => {
            Path::Object(ObjectPath::Toggle)
        },
        &DisplayValue::Scalar(_) => {
            Path::Scalar
        },
    }
}
fn last_path_in(value: &DisplayValue) -> Path {
    match value {
        &DisplayValue::Array(ref array) => {
            if !array.extended {
                Path::Array(ArrayPath::Toggle)
            } else if array.can_grow() {
                Path::Array(ArrayPath::Grow)
            } else if array.can_shrink() {
                Path::Array(ArrayPath::Shrink)
            } else {
                debug_assert!(array.values.is_empty(), "Non-empty array has to be able to shrink OR grow");
                Path::Array(ArrayPath::Toggle)
            }
        },
        &DisplayValue::Object(ref obj) => {
            if let (true, Some((key, value))) = (obj.extended, obj.members.iter().last()) {
                Path::Object(ObjectPath::Item(key.to_string(), Box::new(last_path_in(value))))
            } else {
                Path::Object(ObjectPath::Toggle)
            }
        },
        &DisplayValue::Scalar(_) => {
            Path::Scalar
        },
    }
}

impl Path {

    pub fn find_next_path(self, value: &DisplayValue) -> Option<Self> {
        match value {
            &DisplayValue::Array(ref array) => {
                match (self.unwrap_array(), array.extended) {
                    (ArrayPath::Toggle, false) => {
                        None
                    },
                    (ArrayPath::Toggle, true) => {
                        if let (true, Some(first)) = (array.num_extended > 0, array.values.first()) {
                            Some(Path::Array(ArrayPath::Item(0, Box::new(first_path_in(&*first)))))
                        } else if array.can_grow() {
                            Some(Path::Array(ArrayPath::Grow))
                        } else {
                            None
                        }
                    },
                    (ArrayPath::Item(i, subpath), true) => {
                        if let Some(new_sub_path) = subpath.find_next_path(&array.values[i]) {
                            Some(Path::Array(ArrayPath::Item(i, Box::new(new_sub_path))))
                        } else {
                            let potential_new_i = i+1;
                            if let (true, Some(next)) = (potential_new_i < array.num_extended, array.values.get(potential_new_i)) {
                                Some(Path::Array(ArrayPath::Item(potential_new_i, Box::new(first_path_in(&*next)))))
                            } else {
                                if array.can_shrink() {
                                    Some(Path::Array(ArrayPath::Shrink))
                                } else if array.can_grow() {
                                    Some(Path::Array(ArrayPath::Grow))
                                } else {
                                    None
                                }
                            }
                        }
                    },
                    (ArrayPath::Shrink, true) => {
                        if array.can_grow() {
                            Some(Path::Array(ArrayPath::Grow))
                        } else {
                            None
                        }
                    },
                    (ArrayPath::Grow, true) => {
                        None
                    },
                    (_, false) => {
                        panic!("Invalid path for non-extended array");
                    }
                }
            },

            &DisplayValue::Object(ref obj) => {
                match self.unwrap_object() {
                    ObjectPath::Item(key, subpath) => {
                        assert!(obj.extended, "Item path on non-extended object");
                        if let Some(new_sub_path) = subpath.find_next_path(&obj.members[&key]) {
                            Some(Path::Object(ObjectPath::Item(key, Box::new(new_sub_path))))
                        } else {
                            if let Some((first_key, first_val)) = obj.members.iter().skip_while(|&(k, _)| *k != key).skip(1).next() {
                                Some(Path::Object(ObjectPath::Item(first_key.to_string(), Box::new(first_path_in(first_val)))))
                            } else {
                                None
                            }
                        }
                    },
                    ObjectPath::Toggle => {
                        if let (true, Some((first_key, first_val))) = (obj.extended, obj.members.iter().next()) {
                            Some(Path::Object(ObjectPath::Item(first_key.to_string(), Box::new(first_path_in(first_val)))))
                        } else {
                            None
                        }
                    },
                }
            },

            &DisplayValue::Scalar(_) => {
                None
            },
        }
    }

    pub fn find_previous_path(self, value: &DisplayValue) -> Option<Self> {
        match value {
            &DisplayValue::Array(ref array) => {
                match (self.unwrap_array(), array.extended) {
                    (ArrayPath::Toggle, _) => {
                        None
                    },
                    (ArrayPath::Item(i, subpath), true) => {
                        if let Some(new_sub_path) = subpath.find_previous_path(&array.values[i]) {
                            Some(Path::Array(ArrayPath::Item(i, Box::new(new_sub_path))))
                        } else {
                            if let Some(next) = i.checked_sub(1).and_then(|new_i| array.values.get(new_i)) {
                                Some(Path::Array(ArrayPath::Item(i-1, Box::new(last_path_in(&*next)))))
                            } else {
                                Some(Path::Array(ArrayPath::Toggle))
                            }
                        }
                    },
                    (ArrayPath::Shrink, true) => {
                        Some(Path::Array(if let (true, Some(first)) = (array.num_extended > 0, array.values.iter().nth(array.num_extended-1)) {
                            ArrayPath::Item(array.num_extended-1, Box::new(last_path_in(&*first)))
                        } else {
                            ArrayPath::Toggle
                        }))
                    },
                    (ArrayPath::Grow, true) => {
                        Some(Path::Array(
                                if array.can_shrink() {
                                    ArrayPath::Shrink
                                } else {
                                    ArrayPath::Toggle
                                }
                            ))
                    },
                    (_, false) => {
                        panic!("Invalid path for non-extended array");
                    }
                }
            },

            &DisplayValue::Object(ref obj) => {
                match self.unwrap_object() {
                    ObjectPath::Item(key, subpath) => {
                        if let Some(new_sub_path) = subpath.find_previous_path(&obj.members[&key]) {
                            Some(Path::Object(ObjectPath::Item(key, Box::new(new_sub_path))))
                        } else {
                            if let Some((last_key, last_val)) = obj.members.iter().rev().skip_while(|&(k, _)| *k != key).skip(1).next() {
                                Some(Path::Object(ObjectPath::Item(last_key.to_string(), Box::new(last_path_in(last_val)))))
                            } else {
                                Some(Path::Object(ObjectPath::Toggle))
                            }
                        }
                    },
                    ObjectPath::Toggle => {
                        None
                    },
                }
            },

            &DisplayValue::Scalar(_) => {
                None
            },
        }
    }

    pub fn fix_path_for_value(self, value: &DisplayValue) -> Self {
        match value { // "E0009: cannot bind by-move and by-ref in the same pattern" is really annoying...
            &DisplayValue::Array(ref arr) => {
                if !arr.extended {
                    Path::Array(ArrayPath::Toggle)
                } else {
                    match self {
                        Path::Array(array_path) => {
                            Path::Array(match array_path {
                                ArrayPath::Toggle => {
                                    ArrayPath::Toggle
                                },
                                ArrayPath::Item(i, subpath) => {
                                    if i < arr.num_extended {
                                        let new_sub_path = subpath.fix_path_for_value(&arr.values[i]);
                                        ArrayPath::Item(i, Box::new(new_sub_path))
                                    } else {
                                        if i < arr.values.len() {
                                            ArrayPath::Grow
                                        } else {
                                            ArrayPath::Toggle
                                        }
                                    }
                                },
                                ArrayPath::Shrink => {
                                    if arr.can_shrink() {
                                        ArrayPath::Shrink
                                    } else if arr.can_grow() {
                                        ArrayPath::Grow
                                    } else{
                                        ArrayPath::Toggle
                                    }
                                },
                                ArrayPath::Grow => {
                                    if arr.can_grow() {
                                        ArrayPath::Grow
                                    } else if arr.can_shrink() {
                                        ArrayPath::Shrink
                                    } else {
                                        ArrayPath::Toggle
                                    }
                                },
                            })
                        },
                        _ => {
                            Path::Array(ArrayPath::Toggle)

                        },
                    }
                }
            },
            &DisplayValue::Object(ref obj) => {
                if !obj.extended {
                    Path::Object(ObjectPath::Toggle)
                } else {
                    match self {
                        Path::Object(obj_path) => {
                            Path::Object(match obj_path {
                                ObjectPath::Item(key, subpath) => {
                                    if let Some(val) = obj.members.get(&key) {
                                        let new_sub_path = subpath.fix_path_for_value(val);
                                        ObjectPath::Item(key, Box::new(new_sub_path))
                                    } else {
                                        ObjectPath::Toggle
                                    }
                                },
                                ObjectPath::Toggle => {
                                    ObjectPath::Toggle
                                },
                            })
                        },
                        _ => {
                            Path::Object(ObjectPath::Toggle)
                        },
                    }
                }
            },
            &DisplayValue::Scalar(_) => {
                Path::Scalar
            },
        }
    }

    pub fn find_and_act_on_element(&self, value: &mut DisplayValue) -> Result<(), ()> {
        match (value, self) {
            (&mut DisplayValue::Array(ref mut array), &Path::Array(ArrayPath::Shrink)) => {
                array.shrink();
                Ok(())
            },
            (&mut DisplayValue::Array(ref mut array), &Path::Array(ArrayPath::Toggle)) => {
                array.toggle_visibility();
                Ok(())
            },
            (&mut DisplayValue::Array(ref mut array), &Path::Array(ArrayPath::Item(i, ref subpath))) => {
                subpath.find_and_act_on_element(&mut array.values[i])
            },
            (&mut DisplayValue::Array(ref mut array), &Path::Array(ArrayPath::Grow)) => {
                array.grow();
                Ok(())
            },

            (&mut DisplayValue::Object(ref mut obj), &Path::Object(ObjectPath::Item(ref key, ref subpath))) => {
                subpath.find_and_act_on_element(obj.members.get_mut(key).unwrap())
            },
            (&mut DisplayValue::Object(ref mut obj), &Path::Object(ObjectPath::Toggle)) => {
                obj.toggle_visibility();
                Ok(())
            },

            (&mut DisplayValue::Scalar(_), &Path::Scalar) => {
                // We do not do anything with scalars.
                Err(())
            },
            _ => {
                panic!("Path does not match value");
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use json::JsonValue;

    fn aeq_first_path_in(val: JsonValue, expected: Path) {
        let val = DisplayValue::from_json(&val);
        let result = first_path_in(&val);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_first_path_in() {
        aeq_first_path_in(JsonValue::String("foo".to_string()), Path::Scalar);

        aeq_first_path_in(object!{ "bar" => "b", "foo" => "f"}, Path::object_toggle());

        aeq_first_path_in(array!{ 0, 1, 2, 3, 4}, Path::array_toggle());
    }

    fn aeq_last_path_in(val: JsonValue, expected: Path) {
        let val = DisplayValue::from_json(&val);
        let result = last_path_in(&val);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_last_path_in() {
        aeq_last_path_in(JsonValue::String("foo".to_string()), Path::Scalar);

        aeq_last_path_in(object!{ }, Path::object_toggle());
        aeq_last_path_in(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("foo"));

        aeq_last_path_in(array!{ 0, 1, 2, 3, 4}, Path::array_grow());
        aeq_last_path_in(array!{ 0, 1, 2}, Path::array_shrink());
        aeq_last_path_in(array!{ }, Path::array_toggle());
    }

    fn aeq_next_path_setup<P: Into<Option<Path>>, S: FnOnce(&mut DisplayValue)>(val: JsonValue, setup: S, before: Path, expected_after: P) {
        let expected_after = expected_after.into();
        let mut val = DisplayValue::from_json(&val);
        setup(&mut val);
        let real_after = before.find_next_path(&val);
        assert_eq!(real_after, expected_after);
    }

    fn aeq_next_path<P: Into<Option<Path>>>(val: JsonValue, before: Path, expected_after: P) {
        aeq_next_path_setup(val, |_| {}, before, expected_after);
    }

    #[test]
    fn test_find_next_path() {
        aeq_next_path(JsonValue::String("foo".to_string()), Path::scalar(), None);

        aeq_next_path(object!{ "bar" => "b", "foo" => "f"}, Path::object_toggle(), Path::scalar().object("bar"));
        aeq_next_path(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("bar"), Path::scalar().object("foo"));
        aeq_next_path(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("foo"), None);

        aeq_next_path_setup(object!{ "bar" => "b", "foo" => "f"}, |o| o.unwrap_object_ref_mut().toggle_visibility(), Path::object_toggle(), None);

        // Can grow/shrink
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::array_toggle(), Path::scalar().array(0));
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(0), Path::scalar().array(1));
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(1), Path::scalar().array(2));
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(2), Path::array_shrink());
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::array_shrink(), Path::array_grow());
        aeq_next_path(array!{ 0, 1, 2, 3, 4}, Path::array_grow(), None);

        // Cannot grow
        aeq_next_path(array!{ 0, 1, 2}, Path::array_toggle(), Path::scalar().array(0));
        aeq_next_path(array!{ 0, 1, 2}, Path::scalar().array(0), Path::scalar().array(1));
        aeq_next_path(array!{ 0, 1, 2}, Path::scalar().array(1), Path::scalar().array(2));
        aeq_next_path(array!{ 0, 1, 2}, Path::scalar().array(2), Path::array_shrink());
        aeq_next_path(array!{ 0, 1, 2}, Path::array_shrink(), None);

        // Cannot shrink
        aeq_next_path(array!{ }, Path::array_toggle(), None);

        aeq_next_path_setup(array!{ 0, 1, 2, 3, 4}, |a| a.unwrap_array_ref_mut().toggle_visibility(), Path::array_toggle(), None);

        aeq_next_path(object!{ "bar" => array!{ 1 }, "foo" => "f"}, Path::array_shrink().object("bar"), Path::scalar().object("foo"));
    }

    fn aeq_previous_path_setup<P: Into<Option<Path>>, S: FnOnce(&mut DisplayValue)>(val: JsonValue, setup: S, before: Path, expected_after: P) {
        let expected_after = expected_after.into();
        let mut val = DisplayValue::from_json(&val);
        setup(&mut val);
        let real_after = before.find_previous_path(&val);
        assert_eq!(real_after, expected_after);
    }

    fn aeq_previous_path<P: Into<Option<Path>>>(val: JsonValue, before: Path, expected_after: P) {
        aeq_previous_path_setup(val, |_| {}, before, expected_after);
    }

    #[test]
    fn test_find_previous_path() {
        aeq_previous_path(JsonValue::String("foo".to_string()), Path::scalar(), None);

        aeq_previous_path(object!{ "bar" => "b", "foo" => "f"}, Path::object_toggle(), None);
        aeq_previous_path(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("bar"), Path::object_toggle());
        aeq_previous_path(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("foo"), Path::scalar().object("bar"));

        aeq_previous_path_setup(object!{ "bar" => "b", "foo" => "f"}, |o| o.unwrap_object_ref_mut().toggle_visibility(), Path::object_toggle(), None);

        // Can grow/shrink
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::array_toggle(), None);
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(0), Path::array_toggle());
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(1), Path::scalar().array(0));
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::scalar().array(2), Path::scalar().array(1));
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::array_shrink(), Path::scalar().array(2));
        aeq_previous_path(array!{ 0, 1, 2, 3, 4}, Path::array_grow(), Path::array_shrink());

        // Cannot grow
        aeq_previous_path(array!{ 0, 1, 2}, Path::array_toggle(), None);
        aeq_previous_path(array!{ 0, 1, 2}, Path::scalar().array(0), Path::array_toggle());
        aeq_previous_path(array!{ 0, 1, 2}, Path::scalar().array(1), Path::scalar().array(0));
        aeq_previous_path(array!{ 0, 1, 2}, Path::scalar().array(2), Path::scalar().array(1));
        aeq_previous_path(array!{ 0, 1, 2}, Path::array_shrink(), Path::scalar().array(2));

        // Cannot shrink
        aeq_previous_path(array!{ }, Path::array_toggle(), None);

        aeq_previous_path_setup(array!{ 0, 1, 2, 3, 4}, |a| a.unwrap_array_ref_mut().toggle_visibility(), Path::array_toggle(), None);

        aeq_previous_path(object!{ "bar" => array!{ 1 }, "foo" => "f"}, Path::scalar().object("foo"), Path::array_shrink().object("bar"));
    }

    fn aeq_find_and_act_on_element<F: FnOnce(&DisplayValue) -> bool>(val: JsonValue, path: Path, action_valid: F, should_error: bool) {
        let mut val = DisplayValue::from_json(&val);
        assert!(should_error == path.find_and_act_on_element(&mut val).is_err());
        assert!(action_valid(&val));
    }

    #[test]
    fn test_find_and_act_on_element() {
        aeq_find_and_act_on_element(JsonValue::String("foo".to_owned()), Path::scalar(), |v| v.unwrap_scalar_ref().value == "foo", true);

        aeq_find_and_act_on_element(array!{ 0, 1, 2, 3, 4}, Path::array_grow(), |v| v.unwrap_array_ref().num_extended == 4, false);
        aeq_find_and_act_on_element(array!{ 0, 1, 2, 3, 4}, Path::array_shrink(), |v| v.unwrap_array_ref().num_extended == 2, false);
        aeq_find_and_act_on_element(array!{ 0, 1, 2, 3, 4}, Path::array_toggle(), |v| !v.unwrap_array_ref().extended, false);

        aeq_find_and_act_on_element(object!{ "bar" => "b", "foo" => "f"}, Path::object_toggle(), |v| !v.unwrap_object_ref().extended, false);

        aeq_find_and_act_on_element(object!{ "bar" => array!{0, 1, 2, 3, 4}, "foo" => "f"}, Path::array_grow().object("bar"), |v| v.unwrap_object_ref().members["bar"].unwrap_array_ref().num_extended == 4, false);
        aeq_find_and_act_on_element(object!{ "bar" => array!{0, 1, 2, 3, 4}, "foo" => "f"}, Path::array_shrink().object("bar"), |v| v.unwrap_object_ref().members["bar"].unwrap_array_ref().num_extended == 2, false);
        aeq_find_and_act_on_element(object!{ "bar" => array!{0, 1, 2, 3, 4}, "foo" => "f"}, Path::array_toggle().object("bar"), |v| !v.unwrap_object_ref().members["bar"].unwrap_array_ref().extended, false);

        aeq_find_and_act_on_element(array!{ object!{ "bar" => "b", "foo" => "f"}, 1, 2, 3}, Path::object_toggle().array(0), |v| !v.unwrap_array_ref().values[0].unwrap_object_ref().extended, false);
    }

    fn aeq_fix_path_for_value(val: JsonValue, before: Path, expected_after: Path) {
        aeq_fix_path_for_value_setup(val, |_| {}, before, expected_after);
    }

    fn aeq_fix_path_for_value_setup<S: FnOnce(&mut DisplayValue)>(val: JsonValue, setup: S, before: Path, expected_after: Path) {
        let mut val = DisplayValue::from_json(&val);
        setup(&mut val);
        let real_after = before.fix_path_for_value(&val);
        assert_eq!(real_after, expected_after);
    }

    #[test]
    fn test_fix_path_for_value() {
        // Fallback
        aeq_fix_path_for_value(JsonValue::String("foo".to_string()), Path::object_toggle(), Path::scalar());
        aeq_fix_path_for_value(JsonValue::String("foo".to_string()), Path::array_toggle(), Path::scalar());
        aeq_fix_path_for_value(JsonValue::String("foo".to_string()), Path::scalar(), Path::scalar());

        aeq_fix_path_for_value(array!{ 0, 1, 2, 3, 4}, Path::object_toggle(), Path::array_toggle());
        aeq_fix_path_for_value(array!{ 0, 1, 2, 3, 4}, Path::array_toggle(), Path::array_toggle());
        aeq_fix_path_for_value(array!{ 0, 1, 2, 3, 4}, Path::scalar(), Path::array_toggle());

        aeq_fix_path_for_value(object!{ "bar" => "b", "foo" => "f"}, Path::object_toggle(), Path::object_toggle());
        aeq_fix_path_for_value(object!{ "bar" => "b", "foo" => "f"}, Path::array_toggle(), Path::object_toggle());
        aeq_fix_path_for_value(object!{ "bar" => "b", "foo" => "f"}, Path::scalar(), Path::object_toggle());

        // More complex behavior
        // Arrays:
        aeq_fix_path_for_value_setup(array!{ 1, 2, 3}, |a| { a.unwrap_array_ref_mut().extended = false; }, Path::scalar().array(0), Path::array_toggle());
        aeq_fix_path_for_value_setup(array!{ 1, 2, 3}, |a| { a.unwrap_array_ref_mut().extended = false; }, Path::array_toggle(), Path::array_toggle());
        aeq_fix_path_for_value_setup(array!{ 1, 2, 3}, |a| { a.unwrap_array_ref_mut().extended = false; }, Path::array_grow(), Path::array_toggle());
        aeq_fix_path_for_value_setup(array!{ 1, 2, 3}, |a| { a.unwrap_array_ref_mut().extended = false; }, Path::array_shrink(), Path::array_toggle());


        aeq_fix_path_for_value(array!{ }, Path::scalar().array(1), Path::array_toggle());
        aeq_fix_path_for_value_setup(array!{ 1, 2, 3}, |a| { a.unwrap_array_ref_mut().num_extended = 0; }, Path::scalar().array(0), Path::array_grow());

        aeq_fix_path_for_value(array!{ 0, 1, 2, 3}, Path::array_grow(), Path::array_grow());
        aeq_fix_path_for_value(array!{ 0, 1}, Path::array_grow(), Path::array_shrink());
        aeq_fix_path_for_value(array!{ }, Path::array_grow(), Path::array_toggle());

        aeq_fix_path_for_value(array!{ 0, 1, 2, 3}, Path::array_shrink(), Path::array_shrink());
        aeq_fix_path_for_value_setup(array!{ 0, 1}, |a| { a.unwrap_array_ref_mut().num_extended = 0; }, Path::array_shrink(), Path::array_grow());
        aeq_fix_path_for_value(array!{ }, Path::array_shrink(), Path::array_toggle());

        // Objects:
        aeq_fix_path_for_value_setup(object!{ "bar" => "b", "foo" => "f"}, |o| { o.unwrap_object_ref_mut().extended = false }, Path::object_toggle(), Path::object_toggle());
        aeq_fix_path_for_value_setup(object!{ "bar" => "b", "foo" => "f"}, |o| { o.unwrap_object_ref_mut().extended = false }, Path::scalar().object("foo"), Path::object_toggle());

        aeq_fix_path_for_value(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("foo"), Path::scalar().object("foo"));
        aeq_fix_path_for_value(object!{ "bar" => "b", "foo" => "f"}, Path::scalar().object("nope"), Path::object_toggle());
    }
}
