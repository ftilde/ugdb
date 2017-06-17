use unsegen::widget::widgets::json_ext::{
    JsonValue,
    object,
    Array,
};

use pest::prelude::*;

const ANON_KEY: &'static str = "*anon*";
const HIDDEN_KEY: &'static str = "*hidden*";
const HIDDEN_VALUE: &'static str = "hint: union printing disabled?";

/*
fn unescape_string(s: &str) -> Option<String> {
    let mut bytes = Vec::<u8>::new();
    let mut hit_escape = false;
    for b in s.bytes() {
        if hit_escape {
            let escaped = match b {
                b'\\' => b'\\',
                b'"' => b'\"',
                b'n'  => b'\n',
                b'r'  => b'\r',
                b't'  => b'\t',
                _ => return None,
            };
            bytes.push(escaped);
            hit_escape = false
        } else {
            if b == b'\\' {
                hit_escape = true
            } else {
                bytes.push(b);
            }
        }
    }
    String::from_utf8(bytes).ok()
}
*/

impl_rdp! {
    grammar! {
        json = { whitespace* ~ value ~ whitespace* ~ eoi }
        //json = { value }

        object = { object_full | object_empty | object_hidden }
        object_full = {
              ["{"] ~ pair ~ ([","] ~ pair)* ~ object_close
        }
        object_empty = {
              ["{"] ~ ["}"]
        }
        object_hidden = {
              ["{...}"]
        }
        object_close = {
            ["}"]
        }
        pair = {
              key ~ ["="] ~ value
            | object //For unnamed members
        }

        array = {
              ["["] ~ value ~ ([","] ~ value)* ~ array_close
            | ["{"] ~ value ~ ([","] ~ value)* ~ array_close_alt
            | ["["] ~ array_close
        }
        array_close_alt = {
            ["}"]
        }
        array_close = {
            ["]"]
        }

        value = { string | object | array | t_true | t_false | some_other_value }

        t_false = { ["false"] }
        t_true = { ["true"] }

        key = @{ (!(whitespace | ["="]) ~ any)+ }

        string  = @{ ["\""] ~ (escape | !(["\""]) ~ any)* ~ ["\""] }
        escape  =  _{ ["\\"] ~ (["\""] | ["\\"] | ["n"] | ["r"] | ["t"]) }

        some_other_value = @{ some_other_value_fragment ~ (whitespace* ~ some_other_value_fragment)* }
        some_other_value_fragment = _{ !(["{"] | ["["] | ["\""] | ["}"] | ["]"] | whitespace) ~ (!(["}"] | ["]"] | [","] | whitespace | ["="]) ~ any)+ }

        /* This interferes with parsing function pointers (e.g., 0x1234 <foobar>), so we disable it (at least for now)
        number = @{ ["-"]? ~ int ~ (["."] ~ ['0'..'9']+ ~ exp? | exp)? }
        int    =  _{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        exp    =  _{ (["E"] | ["e"]) ~ (["+"] | ["-"])? ~ int }
        */

        whitespace = _{ [" "] | ["\t"] | ["\r"] | ["\n"] }
    }

    process! {
        get_pair(&self) -> (String, JsonValue) {
            (&s: key, _: value, v: get_value()) => (s.to_string(), v),
            (_: object, o: get_object()) => (ANON_KEY.to_string(), JsonValue::Object(o)),
        }
        get_pair_opt(&self) -> Option<(String, JsonValue)> {
            (_: pair, p: get_pair()) => {
                Some(p)
            },
            (_: object_close) => None,
        }
        get_object(&self) -> object::Object {
            (_: object_full) => {
                let mut o = object::Object::new();
                while let Some((key, value)) = self.get_pair_opt() {
                    if key == ANON_KEY {
                        if o.get(ANON_KEY).is_some() {
                            let anon_value = o.get_mut(ANON_KEY).unwrap();
                            if let &mut JsonValue::Object(ref mut anon_obj) = anon_value {
                                if let JsonValue::Object(val_obj) = value {
                                    for (n_key, n_obj) in val_obj.iter() {
                                        anon_obj.insert(n_key, n_obj.clone());
                                    }
                                } else {
                                    panic!("New {} is not an object", ANON_KEY);
                                }
                            } else {
                                panic!("Old {} is not an object", ANON_KEY);
                            }
                        } else {

                            o.insert(&key, value);
                        }
                    } else {
                        o.insert(&key, value);
                    }
                }
                o
            },
            (_: object_empty) => {
                object::Object::new()
            },
            (_: object_hidden) => {
                let mut o = object::Object::new();
                o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
                o
            },
        }
        get_value_opt(&self) -> Option<JsonValue> {
            (_: value, v: get_value()) => {
                Some(v)
            },
            (_: array_close) => None,
            (_: array_close_alt) => None,
        }
        get_array(&self) -> Array {
            () => {
                let mut a = Array::new();
                while let Some(value) = self.get_value_opt() {
                    a.push(value);
                }
                a
            },
        }
        get_value(&self) -> JsonValue {
            (&s: string) => JsonValue::String(s.to_string()),
            (&v: some_other_value) => JsonValue::String(v.to_string()),
            //(&n: number) => JsonValue::String(n.to_string()), // See above
            (_: object, o: get_object()) => JsonValue::Object(o),
            (_: array, a: get_array()) => JsonValue::Array(a),
            (_: t_true) => JsonValue::Boolean(true),
            (_: t_false) => JsonValue::Boolean(false),
        }
        get_json(&self) -> JsonValue {
            (_: json, _: value, val: get_value()) => val,
        }
    }
}

pub fn parse_gdb_value(result_string: &str) -> JsonValue {
    let parse_result = ::std::panic::catch_unwind(|| {
        let mut parser = Rdp::new(StringInput::new(&result_string));
        parser.json();
        //println!("{:?}", parser.queue_with_captures());
        //println!("{:?}", parser.expected());
        parser.get_json()
    });

    match parse_result {
        Ok(res) => res,
        Err(_) => JsonValue::String(format!("*Error parsing*: {}", result_string))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_basic() {
        assert_eq!(parse_gdb_value("true"), JsonValue::Boolean(true));
        assert_eq!(parse_gdb_value("false"), JsonValue::Boolean(false));
        assert_eq!(parse_gdb_value("27"), JsonValue::String("27".to_string())); //This is probably sufficient for us
        assert_eq!(parse_gdb_value("27.0"), JsonValue::String("27.0".to_string()));
        assert_eq!(parse_gdb_value("\"dfd\""), JsonValue::String("\"dfd\"".to_string()));
        assert_eq!(parse_gdb_value(" l r "), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value("{}"), JsonValue::Object(object::Object::new()));
        assert_eq!(parse_gdb_value("[]"), JsonValue::Array(Array::new()));
        assert_eq!(parse_gdb_value("{...}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
            o
        }));
    }

    #[test]
    fn test_complex() {
        let testcase = "
        {
            boolean = 128,
            x = 0,
            y = \"kdf\\\\j}{\\\"\",
            {
                z = 5.88163081e-39,
                w = 0
            },
            named_inner = {
                v = 1.40129846e-45,
                w = 0
            },
            bar = {
                x = 4295032831,
                y = 140737488347120,
                z = 4197294
            },
            {...},
            uni_extern = {...},
            uni_intern = {...},
            const_array = {0, 0},
            ptr = 0x400bf0 <__libc_csu_init>,
            array = 0x7fffffffe018
        }
        ";
        let result_obj = JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("boolean", JsonValue::String("128".to_owned()));
            o.insert("x", JsonValue::String("0".to_owned()));
            o.insert("y", JsonValue::String("\"kdf\\\\j}{\\\"\"".to_owned()));
            o.insert(ANON_KEY, JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("z", JsonValue::String("5.88163081e-39".to_owned()));
                o.insert("w", JsonValue::String("0".to_owned()));
                o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
                o
            }));
            o.insert("named_inner", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("v", JsonValue::String("1.40129846e-45".to_owned()));
                o.insert("w", JsonValue::String("0".to_owned()));
                o
            }));
            o.insert("bar", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("x", JsonValue::String("4295032831".to_owned()));
                o.insert("y", JsonValue::String("140737488347120".to_owned()));
                o.insert("z", JsonValue::String("4197294".to_owned()));
                o
            }));
            o.insert("uni_extern", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
                o
            }));
            o.insert("uni_intern", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
                o
            }));
            o.insert("const_array", JsonValue::Array({
                let mut o = Array::new();
                o.push(JsonValue::String("0".to_string()));
                o.push(JsonValue::String("0".to_string()));
                o
            }));
            o.insert("ptr", JsonValue::String("0x400bf0 <__libc_csu_init>".to_owned()));
            o.insert("array", JsonValue::String("0x7fffffffe018".to_owned()));
            o
        });
        let r = parse_gdb_value(testcase);
        println!("{}", r.pretty(2));
        assert_eq!(r, result_obj);

    }

    #[test]
    fn test_parse_string() {
        //assert_eq!(parse_gdb_value("\"foo{]}]]}]<>,\\\\\""), JsonValue::String("\"foo{]}]]}]<>,\\\"".to_string()));
        assert_eq!(parse_gdb_value("\"foo\""), JsonValue::String("\"foo\"".to_string()));
        assert_eq!(parse_gdb_value("\"foo\\\"\""), JsonValue::String("\"foo\\\"\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\\\}{\\\"\""), JsonValue::String("\"\\\\}{\\\"\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\t\""), JsonValue::String("\"\\t\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\n\""), JsonValue::String("\"\\n\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\r\""), JsonValue::String("\"\\r\"".to_string()));
        assert_eq!(parse_gdb_value("\"kdf\\\\j}{\\\"\""), JsonValue::String("\"kdf\\\\j}{\\\"\"".to_string()));
    }

    #[test]
    fn test_parse_something_else() {
        assert_eq!(parse_gdb_value("l r"), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value(" l r"), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value("l r "), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value(" l r "), JsonValue::String("l r".to_string()));

        assert_eq!(parse_gdb_value("[ l r, l r]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[l r,l r]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[ l r ,l r ]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[ l r , l r ]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));

        assert_eq!(parse_gdb_value("{foo =l r,bar =l r}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo = l r ,bar =l r}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo =l r,bar = l r }"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo = l r ,bar = l r }"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));

        // GDB really does not make it easy for us...
        assert_eq!(parse_gdb_value("{int (int, int)} 0x400a76 <foo(int, int)>"), JsonValue::String("{int (int, int)} 0x400a76 <foo(int, int)>".to_string()));
    }

    #[test]
    fn test_parse_objects() {
        assert_eq!(parse_gdb_value(" { foo = 27}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{ foo = 27, bar = 37}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert("bar", JsonValue::String("37".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{{...}}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("*anon*", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert(HIDDEN_KEY, JsonValue::String(HIDDEN_VALUE.to_owned()));
                o
            }));
            o
        }));
        assert_eq!(parse_gdb_value("{{}}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("*anon*", JsonValue::Object(object::Object::new()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo = 27, { bar=37}}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert("*anon*", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("bar", JsonValue::String("37".to_owned()));
                o
            }));
            o
        }));
        assert_eq!(parse_gdb_value("{{ bar=37}, foo = 27}"), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert("*anon*", JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("bar", JsonValue::String("37".to_owned()));
                o
            }));
            o
        }));
    }

    #[test]
    fn test_parse_arrays() {
        assert_eq!(parse_gdb_value("[27]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{27}"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("[ 27, 37]"), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o.push(JsonValue::String("37".to_owned()));
            o
        }));
    }
}
