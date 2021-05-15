mod ast;
mod lexer;
mod parser;

use unsegen_jsonviewer::Value;

pub type ParseError = parser::Error;

pub fn parse_gdb_value(result_string: &str) -> Result<GDBValue, ParseError> {
    let lexer = lexer::Lexer::new(result_string);
    let ast = parser::parse(lexer)?;
    Ok(ast.to_value(result_string))
}

#[derive(PartialEq, Debug)]
pub enum GDBValue {
    String(String),
    Integer(String, i128),
    Array(Vec<GDBValue>),
    Map(Vec<(String, GDBValue)>),
}

impl unsegen_jsonviewer::Value for GDBValue {
    fn visit<'children>(&'children self) -> unsegen_jsonviewer::ValueVariant<'children> {
        match self {
            GDBValue::String(s) => unsegen_jsonviewer::ValueVariant::Scalar(s.clone()),
            GDBValue::Integer(s, _) => unsegen_jsonviewer::ValueVariant::Scalar(s.clone()), //TODO
            GDBValue::Map(val) => unsegen_jsonviewer::ValueVariant::Map(Box::new(
                val.iter().map(|(k, v)| (k.to_owned(), v as &dyn Value)),
            )),
            GDBValue::Array(val) => unsegen_jsonviewer::ValueVariant::Array(Box::new(
                val.iter().map(|v| v as &dyn Value),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ast::ANON_KEY;
    use super::*;

    #[test]
    fn test_parse_basic() {
        assert_eq!(
            parse_gdb_value("true").unwrap(),
            GDBValue::String("true".to_owned())
        );
        assert_eq!(
            parse_gdb_value("false").unwrap(),
            GDBValue::String("false".to_owned())
        );
        assert_eq!(
            parse_gdb_value("27").unwrap(),
            GDBValue::Integer("27".to_string(), 27)
        ); //This is probably sufficient for us
        assert_eq!(
            parse_gdb_value("27.1").unwrap(),
            GDBValue::String("27.1".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"dfd\"").unwrap(),
            GDBValue::String("\"dfd\"".to_string())
        );
        assert_eq!(
            parse_gdb_value(" l r ").unwrap(),
            GDBValue::String("l r".to_string())
        );
        assert_eq!(
            parse_gdb_value(" 0x123").unwrap(),
            GDBValue::Integer("0x123".to_string(), 0x123)
        );
        assert_eq!(
            parse_gdb_value(" -123").unwrap(),
            GDBValue::Integer("-123".to_string(), -123)
        );
        assert_eq!(parse_gdb_value("{}").unwrap(), GDBValue::Map(Vec::new()));
        assert_eq!(
            parse_gdb_value("{...}").unwrap(),
            GDBValue::Array(vec! { GDBValue::String("...".to_owned()) })
        );
        assert_eq!(
            parse_gdb_value("foo,bar").unwrap(),
            GDBValue::String("foo,bar".to_owned())
        );
        assert_eq!(
            parse_gdb_value("foo}bar").unwrap(),
            GDBValue::String("foo}bar".to_owned())
        );
        assert_eq!(
            parse_gdb_value("\nfoo\nbar").unwrap(),
            GDBValue::String("foo\nbar".to_owned())
        );
    }

    #[test]
    fn test_complex() {
        let testcase = "{
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
            vec_empty = std::vector of length 0, capacity 0,
            map_empty = std::map with 0 elements,
            vec = std::vector of length 1, capacity 1 = {{
                x = 1,
                y = 2,
                z = 3
            }},
            map = std::map with 2 elements = {
              [\"blub\"] = 2,
              [\"foo\"] = 123
            },
            uni_extern = {...},
            uni_intern = {...},
            const_array = {0, 0},
            ptr = 0x400bf0 <__libc_csu_init>,
            array = 0x7fffffffe018
        }";
        let result_obj = GDBValue::Map(vec![
            (
                "boolean".to_owned(),
                GDBValue::Integer("128".to_owned(), 128),
            ),
            ("x".to_owned(), GDBValue::Integer("0".to_owned(), 0)),
            (
                "y".to_owned(),
                GDBValue::String("\"kdf\\\\j}{\\\"\"".to_owned()),
            ),
            (
                "named_inner".to_owned(),
                GDBValue::Map(vec![
                    (
                        "v".to_owned(),
                        GDBValue::String("1.40129846e-45".to_owned()),
                    ),
                    ("w".to_owned(), GDBValue::Integer("0".to_owned(), 0)),
                ]),
            ),
            (
                "bar".to_owned(),
                GDBValue::Map(vec![
                    (
                        "x".to_owned(),
                        GDBValue::Integer("4295032831".to_owned(), 4295032831),
                    ),
                    (
                        "y".to_owned(),
                        GDBValue::Integer("140737488347120".to_owned(), 140737488347120),
                    ),
                    (
                        "z".to_owned(),
                        GDBValue::Integer("4197294".to_owned(), 4197294),
                    ),
                ]),
            ),
            (
                "vec_empty".to_owned(),
                GDBValue::String("std::vector of length 0, capacity 0".to_owned()),
            ),
            (
                "map_empty".to_owned(),
                GDBValue::String("std::map with 0 elements".to_owned()),
            ),
            (
                "vec".to_owned(),
                GDBValue::Array(vec![GDBValue::Map(vec![
                    ("x".to_owned(), GDBValue::Integer("1".to_owned(), 1)),
                    ("y".to_owned(), GDBValue::Integer("2".to_owned(), 2)),
                    ("z".to_owned(), GDBValue::Integer("3".to_owned(), 3)),
                ])]),
            ),
            (
                "map".to_owned(),
                GDBValue::Map(vec![
                    (
                        "[\"blub\"]".to_owned(),
                        GDBValue::Integer("2".to_owned(), 2),
                    ),
                    (
                        "[\"foo\"]".to_owned(),
                        GDBValue::Integer("123".to_owned(), 123),
                    ),
                ]),
            ),
            (
                "uni_extern".to_owned(),
                GDBValue::Array(vec![GDBValue::String("...".to_owned())]),
            ),
            (
                "uni_intern".to_owned(),
                GDBValue::Array(vec![GDBValue::String("...".to_owned())]),
            ),
            (
                "const_array".to_owned(),
                GDBValue::Array(vec![
                    GDBValue::Integer("0".to_owned(), 0),
                    GDBValue::Integer("0".to_owned(), 0),
                ]),
            ),
            (
                "ptr".to_owned(),
                GDBValue::String("0x400bf0 <__libc_csu_init>".to_owned()),
            ),
            (
                "array".to_owned(),
                GDBValue::Integer("0x7fffffffe018".to_owned(), 0x7fffffffe018),
            ),
            (
                ANON_KEY.to_owned(),
                GDBValue::Array(vec![
                    GDBValue::Map(vec![
                        (
                            "z".to_owned(),
                            GDBValue::String("5.88163081e-39".to_owned()),
                        ),
                        ("w".to_owned(), GDBValue::Integer("0".to_owned(), 0)),
                    ]),
                    GDBValue::Array(vec![GDBValue::String("...".to_owned())]),
                ]),
            ),
        ]);
        let r = parse_gdb_value(testcase).unwrap();
        //println!("{}", r.pretty(2));
        assert_eq!(r, result_obj);
    }

    #[test]
    fn test_parse_string() {
        //assert_eq!(parse_gdb_value("\"foo{]}]]}]<>,\\\\\""), GDBValue::String("\"foo{]}]]}]<>,\\\"".to_string()));
        assert_eq!(
            parse_gdb_value("\"foo\"").unwrap(),
            GDBValue::String("\"foo\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"foo\\\"\"").unwrap(),
            GDBValue::String("\"foo\\\"\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"\\\\}{\\\"\"").unwrap(),
            GDBValue::String("\"\\\\}{\\\"\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"\\t\"").unwrap(),
            GDBValue::String("\"\\t\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"\\n\"").unwrap(),
            GDBValue::String("\"\\n\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"\\r\"").unwrap(),
            GDBValue::String("\"\\r\"".to_string())
        );
        assert_eq!(
            parse_gdb_value("\"kdf\\\\j}{\\\"\"").unwrap(),
            GDBValue::String("\"kdf\\\\j}{\\\"\"".to_string())
        );
    }

    #[test]
    fn test_parse_something_else() {
        assert_eq!(
            parse_gdb_value("l r").unwrap(),
            GDBValue::String("l r".to_string())
        );
        assert_eq!(
            parse_gdb_value(" l r").unwrap(),
            GDBValue::String("l r".to_string())
        );
        assert_eq!(
            parse_gdb_value("l r ").unwrap(),
            GDBValue::String("l r".to_string())
        );
        assert_eq!(
            parse_gdb_value(" l r ").unwrap(),
            GDBValue::String("l r".to_string())
        );

        assert_eq!(
            parse_gdb_value("{ l r, l r}").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::String("l r".to_string()));
                o.push(GDBValue::String("l r".to_string()));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{l r,l r}").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::String("l r".to_string()));
                o.push(GDBValue::String("l r".to_string()));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{ l r ,l r }").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::String("l r".to_string()));
                o.push(GDBValue::String("l r".to_string()));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{ l r , l r }").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::String("l r".to_string()));
                o.push(GDBValue::String("l r".to_string()));
                o
            })
        );

        assert_eq!(
            parse_gdb_value("{foo =l r,bar =l r}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::String("l r".to_owned())));
                o.push(("bar".to_owned(), GDBValue::String("l r".to_owned())));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{foo = l r ,bar =l r}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::String("l r".to_owned())));
                o.push(("bar".to_owned(), GDBValue::String("l r".to_owned())));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{foo =l r,bar = l r }").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::String("l r".to_owned())));
                o.push(("bar".to_owned(), GDBValue::String("l r".to_owned())));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{foo = l r ,bar = l r }").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::String("l r".to_owned())));
                o.push(("bar".to_owned(), GDBValue::String("l r".to_owned())));
                o
            })
        );

        // GDB really does not make it easy for us...
        assert_eq!(
            parse_gdb_value("{int (int, int)} 0x400a76 <foo(int, int)>").unwrap(),
            GDBValue::String("{int (int, int)} 0x400a76 <foo(int, int)>".to_string())
        );

        assert_eq!(
            parse_gdb_value("{ {int (int, int)} 0x400a76 <foo(int, int)> }").unwrap(),
            GDBValue::Array(vec![GDBValue::String(
                "{int (int, int)} 0x400a76 <foo(int, int)>".to_string()
            )])
        );
    }

    #[test]
    fn test_parse_objects() {
        assert_eq!(
            parse_gdb_value(" { foo = 27}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::Integer("27".to_owned(), 27)));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{ foo = 27, bar = 37}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::Integer("27".to_owned(), 27)));
                o.push(("bar".to_owned(), GDBValue::Integer("37".to_owned(), 37)));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{\n foo = 27,\n bar = 37\n }").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::Integer("27".to_owned(), 27)));
                o.push(("bar".to_owned(), GDBValue::Integer("37".to_owned(), 37)));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{{...}}").unwrap(),
            GDBValue::Array(vec![GDBValue::Array(vec![GDBValue::String(
                "...".to_owned()
            )])])
        );
        assert_eq!(
            parse_gdb_value("{{}}").unwrap(),
            GDBValue::Array(vec![GDBValue::Map(vec![])])
        );
        assert_eq!(
            parse_gdb_value("{foo = 27, { bar=37}}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::Integer("27".to_owned(), 27)));
                o.push((
                    ANON_KEY.to_owned(),
                    GDBValue::Map({
                        let mut o = Vec::new();
                        o.push(("bar".to_owned(), GDBValue::Integer("37".to_owned(), 37)));
                        o
                    }),
                ));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{{ bar=37}, foo = 27}").unwrap(),
            GDBValue::Map({
                let mut o = Vec::new();
                o.push(("foo".to_owned(), GDBValue::Integer("27".to_owned(), 27)));
                o.push((
                    ANON_KEY.to_owned(),
                    GDBValue::Map({
                        let mut o = Vec::new();
                        o.push(("bar".to_owned(), GDBValue::Integer("37".to_owned(), 37)));
                        o
                    }),
                ));
                o
            })
        );
    }

    #[test]
    fn test_parse_arrays() {
        assert_eq!(
            parse_gdb_value("{27}").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::Integer("27".to_owned(), 27));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{ 27, 37}").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::Integer("27".to_owned(), 27));
                o.push(GDBValue::Integer("37".to_owned(), 37));
                o
            })
        );
        assert_eq!(
            parse_gdb_value("{\n 27,\n 37\n}").unwrap(),
            GDBValue::Array({
                let mut o = Vec::new();
                o.push(GDBValue::Integer("27".to_owned(), 27));
                o.push(GDBValue::Integer("37".to_owned(), 37));
                o
            })
        );
    }
}
