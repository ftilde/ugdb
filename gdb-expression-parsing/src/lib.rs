#[macro_use]
extern crate json;
extern crate lalrpop_util;

mod lexer;
mod ast;
mod parser;

use json::{
    JsonValue,
};


pub type ParseError = lalrpop_util::ParseError<lexer::Location, lexer::Token, lexer::LexicalError>;


pub fn parse_gdb_value(result_string: &str) -> Result<JsonValue, ParseError> {
    let lexer = lexer::Lexer::new(result_string);
    let ast = parser::parse_Value(lexer)?;
    Ok(ast.to_json(result_string))
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::ast::ANON_KEY;
    use unsegen_jsonviewer::json_ext::{
        JsonValue,
        Array,
        object,
    };

    #[test]
    fn test_parse_basic() {
        assert_eq!(parse_gdb_value("true").unwrap(), JsonValue::Boolean(true));
        assert_eq!(parse_gdb_value("false").unwrap(), JsonValue::Boolean(false));
        assert_eq!(parse_gdb_value("27").unwrap(), JsonValue::String("27".to_string())); //This is probably sufficient for us
        assert_eq!(parse_gdb_value("27.0").unwrap(), JsonValue::String("27.0".to_string()));
        assert_eq!(parse_gdb_value("\"dfd\"").unwrap(), JsonValue::String("\"dfd\"".to_string()));
        assert_eq!(parse_gdb_value(" l r ").unwrap(), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value("{}").unwrap(), JsonValue::Object(object::Object::new()));
        assert_eq!(parse_gdb_value("[]").unwrap(), JsonValue::Array(Array::new()));
        assert_eq!(parse_gdb_value("{...}").unwrap(), array!{ "..." });
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
        let result_obj = object! {
            "boolean" => "128",
            "x" => "0",
            "y" => "\"kdf\\\\j}{\\\"\"",
            ANON_KEY => array! {
                object! {
                    "z"  => "5.88163081e-39",
                    "w"  => "0"
                },
                array! { "..." }
            },
            "named_inner" => object! {
                "v" => "1.40129846e-45",
                "w" => "0"
            },
            "bar" => object! {
                "x" => "4295032831",
                "y" => "140737488347120",
                "z" => "4197294",
            },
            "uni_extern" => array! {
                "..."
            },
            "uni_intern" => array! {
                "..."
            },
            "const_array" => array! { "0", "0" },
            "ptr" => "0x400bf0 <__libc_csu_init>",
            "array" => "0x7fffffffe018"
        };
        let r = parse_gdb_value(testcase).unwrap();
        println!("{}", r.pretty(2));
        assert_eq!(r, result_obj);

    }

    #[test]
    fn test_parse_string() {
        //assert_eq!(parse_gdb_value("\"foo{]}]]}]<>,\\\\\""), JsonValue::String("\"foo{]}]]}]<>,\\\"".to_string()));
        assert_eq!(parse_gdb_value("\"foo\"").unwrap(), JsonValue::String("\"foo\"".to_string()));
        assert_eq!(parse_gdb_value("\"foo\\\"\"").unwrap(), JsonValue::String("\"foo\\\"\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\\\}{\\\"\"").unwrap(), JsonValue::String("\"\\\\}{\\\"\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\t\"").unwrap(), JsonValue::String("\"\\t\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\n\"").unwrap(), JsonValue::String("\"\\n\"".to_string()));
        assert_eq!(parse_gdb_value("\"\\r\"").unwrap(), JsonValue::String("\"\\r\"".to_string()));
        assert_eq!(parse_gdb_value("\"kdf\\\\j}{\\\"\"").unwrap(), JsonValue::String("\"kdf\\\\j}{\\\"\"".to_string()));
    }

    #[test]
    fn test_parse_something_else() {
        assert_eq!(parse_gdb_value("l r").unwrap(), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value(" l r").unwrap(), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value("l r ").unwrap(), JsonValue::String("l r".to_string()));
        assert_eq!(parse_gdb_value(" l r ").unwrap(), JsonValue::String("l r".to_string()));

        assert_eq!(parse_gdb_value("[ l r, l r]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[l r,l r]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[ l r ,l r ]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));
        assert_eq!(parse_gdb_value("[ l r , l r ]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("l r".to_string()));
            o.push(JsonValue::String("l r".to_string()));
            o
        }));

        assert_eq!(parse_gdb_value("{foo =l r,bar =l r}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo = l r ,bar =l r}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo =l r,bar = l r }").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{foo = l r ,bar = l r }").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("l r".to_owned()));
            o.insert("bar", JsonValue::String("l r".to_owned()));
            o
        }));

        // GDB really does not make it easy for us...
        assert_eq!(parse_gdb_value("{int (int, int)} 0x400a76 <foo(int, int)>").unwrap(), JsonValue::String("{int (int, int)} 0x400a76 <foo(int, int)>".to_string()));

        assert_eq!(parse_gdb_value("[ {int (int, int)} 0x400a76 <foo(int, int)> ]").unwrap(), array! {"{int (int, int)} 0x400a76 <foo(int, int)>"});
    }

    #[test]
    fn test_parse_objects() {
        assert_eq!(parse_gdb_value(" { foo = 27}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{ foo = 27, bar = 37}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert("bar", JsonValue::String("37".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{{...}}").unwrap(), array!{ array! { "..." } });
        assert_eq!(parse_gdb_value("{{}}").unwrap(), array!{ object!{} });
        assert_eq!(parse_gdb_value("{foo = 27, { bar=37}}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert(ANON_KEY, JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("bar", JsonValue::String("37".to_owned()));
                o
            }));
            o
        }));
        assert_eq!(parse_gdb_value("{{ bar=37}, foo = 27}").unwrap(), JsonValue::Object({
            let mut o = object::Object::new();
            o.insert("foo", JsonValue::String("27".to_owned()));
            o.insert(ANON_KEY, JsonValue::Object({
                let mut o = object::Object::new();
                o.insert("bar", JsonValue::String("37".to_owned()));
                o
            }));
            o
        }));
    }

    #[test]
    fn test_parse_arrays() {
        assert_eq!(parse_gdb_value("[27]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("{27}").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o
        }));
        assert_eq!(parse_gdb_value("[ 27, 37]").unwrap(), JsonValue::Array({
            let mut o = Array::new();
            o.push(JsonValue::String("27".to_owned()));
            o.push(JsonValue::String("37".to_owned()));
            o
        }));
    }
}
