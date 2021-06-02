mod ast;
mod lexer;
mod parser;

const ANON_KEY: &'static str = "*anon*";

pub type ParseError = parser::Error;

pub fn parse_gdb_value<'s>(result_string: &'s str) -> Result<Node, ParseError> {
    let lexer = lexer::Lexer::new(result_string);
    parser::parse(lexer, result_string)
}

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Leaf(&'a str),
    Array(Option<&'a str>, Vec<Node<'a>>),
    Map(Option<&'a str>, Vec<(&'a str, Node<'a>)>),
}

#[derive(Clone, Copy)]
pub enum Format {
    Decimal,
    Hex,
    Octal,
    Binary,
}

#[derive(Clone)]
pub struct Value<'s> {
    pub node: &'s Node<'s>,
    pub format: Option<Format>,
}

impl<'n> unsegen_jsonviewer::Value for Value<'n> {
    fn visit<'s>(self) -> unsegen_jsonviewer::ValueVariant<'s, Self> {
        match self.node {
            Node::Leaf(s) => {
                let res = if let Some(format) = self.format {
                    match parse_int::parse::<i128>(s) {
                        Err(_) => s.to_string(),
                        Ok(i) => match format {
                            Format::Decimal => i.to_string(),
                            Format::Hex => format!("{:#x}", i),
                            Format::Octal => format!("{:#o}", i),
                            Format::Binary => format!("{:#b}", i),
                        },
                    }
                } else {
                    s.to_string()
                };
                unsegen_jsonviewer::ValueVariant::Scalar(res)
            }
            Node::Map(description, items) => unsegen_jsonviewer::ValueVariant::Map(
                description.map(|s| s.to_owned()),
                Box::new(items.iter().map(move |(s, v)| {
                    (
                        s.to_string(),
                        Value {
                            node: v,
                            format: self.format,
                        },
                    )
                })),
            ),
            Node::Array(description, items) => unsegen_jsonviewer::ValueVariant::Array(
                description.map(|s| s.to_owned()),
                Box::new(items.iter().map(move |v| Value {
                    node: v,
                    format: self.format,
                })),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_basic() {
        assert_eq!(parse_gdb_value("true").unwrap(), Node::Leaf("true"));
        assert_eq!(parse_gdb_value("false").unwrap(), Node::Leaf("false"));
        assert_eq!(parse_gdb_value("27").unwrap(), Node::Leaf("27")); //This is probably sufficient for us
        assert_eq!(parse_gdb_value("27.1").unwrap(), Node::Leaf("27.1"));
        assert_eq!(parse_gdb_value("\"dfd\"").unwrap(), Node::Leaf("\"dfd\""));
        assert_eq!(parse_gdb_value(" l r ").unwrap(), Node::Leaf("l r"));
        assert_eq!(parse_gdb_value(" 0x123").unwrap(), Node::Leaf("0x123"));
        assert_eq!(parse_gdb_value(" -123").unwrap(), Node::Leaf("-123"));
        assert_eq!(
            parse_gdb_value("{}").unwrap(),
            Node::Array(None, Vec::new())
        );
        assert_eq!(
            parse_gdb_value("{...}").unwrap(),
            Node::Array(None, vec! { Node::Leaf("...") })
        );
        assert_eq!(parse_gdb_value("foo,bar").unwrap(), Node::Leaf("foo,bar"));
        assert_eq!(parse_gdb_value("foo}bar").unwrap(), Node::Leaf("foo}bar"));
        assert_eq!(
            parse_gdb_value("\nfoo\nbar").unwrap(),
            Node::Leaf("foo\nbar")
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
        let result_obj = Node::Map(
            None,
            vec![
                ("boolean", Node::Leaf("128")),
                ("x", Node::Leaf("0")),
                ("y", Node::Leaf("\"kdf\\\\j}{\\\"\"")),
                (
                    "named_inner",
                    Node::Map(
                        None,
                        vec![("v", Node::Leaf("1.40129846e-45")), ("w", Node::Leaf("0"))],
                    ),
                ),
                (
                    "bar",
                    Node::Map(
                        None,
                        vec![
                            ("x", Node::Leaf("4295032831")),
                            ("y", Node::Leaf("140737488347120")),
                            ("z", Node::Leaf("4197294")),
                        ],
                    ),
                ),
                (
                    "vec_empty",
                    Node::Leaf("std::vector of length 0, capacity 0"),
                ),
                ("map_empty", Node::Leaf("std::map with 0 elements")),
                (
                    "vec",
                    Node::Array(
                        Some("std::vector of length 1, capacity 1 ="),
                        vec![Node::Map(
                            None,
                            vec![
                                ("x", Node::Leaf("1")),
                                ("y", Node::Leaf("2")),
                                ("z", Node::Leaf("3")),
                            ],
                        )],
                    ),
                ),
                (
                    "map",
                    Node::Map(
                        Some("std::map with 2 elements ="),
                        vec![
                            ("[\"blub\"]", Node::Leaf("2")),
                            ("[\"foo\"]", Node::Leaf("123")),
                        ],
                    ),
                ),
                ("uni_extern", Node::Array(None, vec![Node::Leaf("...")])),
                ("uni_intern", Node::Array(None, vec![Node::Leaf("...")])),
                (
                    "const_array",
                    Node::Array(None, vec![Node::Leaf("0"), Node::Leaf("0")]),
                ),
                ("ptr", Node::Leaf("0x400bf0 <__libc_csu_init>")),
                ("array", Node::Leaf("0x7fffffffe018")),
                (
                    ANON_KEY,
                    Node::Array(
                        None,
                        vec![
                            Node::Map(
                                None,
                                vec![("z", Node::Leaf("5.88163081e-39")), ("w", Node::Leaf("0"))],
                            ),
                            Node::Array(None, vec![Node::Leaf("...")]),
                        ],
                    ),
                ),
            ],
        );
        let r = parse_gdb_value(testcase).unwrap();
        //println!("{}", r.pretty(2));
        assert_eq!(r, result_obj);
    }

    #[test]
    fn test_parse_string() {
        //assert_eq!(parse_gdb_value("\"foo{]}]]}]<>,\\\\\""), Node::Leaf("\"foo{]}]]}]<>,\\\"".to_string()));
        assert_eq!(parse_gdb_value("\"foo\"").unwrap(), Node::Leaf("\"foo\""));
        assert_eq!(
            parse_gdb_value("\"foo\\\"\"").unwrap(),
            Node::Leaf("\"foo\\\"\"")
        );
        assert_eq!(
            parse_gdb_value("\"\\\\}{\\\"\"").unwrap(),
            Node::Leaf("\"\\\\}{\\\"\"")
        );
        assert_eq!(parse_gdb_value("\"\\t\"").unwrap(), Node::Leaf("\"\\t\""));
        assert_eq!(parse_gdb_value("\"\\n\"").unwrap(), Node::Leaf("\"\\n\""));
        assert_eq!(parse_gdb_value("\"\\r\"").unwrap(), Node::Leaf("\"\\r\""));
        assert_eq!(
            parse_gdb_value("\"kdf\\\\j}{\\\"\"").unwrap(),
            Node::Leaf("\"kdf\\\\j}{\\\"\"")
        );
    }

    #[test]
    fn test_parse_something_else() {
        assert_eq!(parse_gdb_value("l r").unwrap(), Node::Leaf("l r"));
        assert_eq!(parse_gdb_value(" l r").unwrap(), Node::Leaf("l r"));
        assert_eq!(parse_gdb_value("l r ").unwrap(), Node::Leaf("l r"));
        assert_eq!(parse_gdb_value(" l r ").unwrap(), Node::Leaf("l r"));

        assert_eq!(
            parse_gdb_value("{ l r, l r}").unwrap(),
            Node::Array(None, vec![Node::Leaf("l r"), Node::Leaf("l r")])
        );
        assert_eq!(
            parse_gdb_value("{l r,l r}").unwrap(),
            Node::Array(None, vec![Node::Leaf("l r"), Node::Leaf("l r")])
        );
        assert_eq!(
            parse_gdb_value("{ l r ,l r }").unwrap(),
            Node::Array(None, vec![Node::Leaf("l r"), Node::Leaf("l r")])
        );
        assert_eq!(
            parse_gdb_value("{ l r , l r }").unwrap(),
            Node::Array(None, vec![Node::Leaf("l r"), Node::Leaf("l r")])
        );

        assert_eq!(
            parse_gdb_value("{foo =l r,bar =l r}").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("l r")), ("bar", Node::Leaf("l r"))]
            )
        );
        assert_eq!(
            parse_gdb_value("{foo = l r ,bar =l r}").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("l r")), ("bar", Node::Leaf("l r"))]
            )
        );
        assert_eq!(
            parse_gdb_value("{foo =l r,bar = l r }").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("l r")), ("bar", Node::Leaf("l r"))]
            )
        );
        assert_eq!(
            parse_gdb_value("{foo = l r ,bar = l r }").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("l r")), ("bar", Node::Leaf("l r"))]
            )
        );

        // GDB really does not make it easy for us...
        assert_eq!(
            parse_gdb_value("{int (int, int)} 0x400a76 <foo(int, int)>").unwrap(),
            Node::Leaf("{int (int, int)} 0x400a76 <foo(int, int)>")
        );

        assert_eq!(
            parse_gdb_value("{ {int (int, int)} 0x400a76 <foo(int, int)> }").unwrap(),
            Node::Array(
                None,
                vec![Node::Leaf("{int (int, int)} 0x400a76 <foo(int, int)>")]
            )
        );
    }

    #[test]
    fn test_parse_objects() {
        assert_eq!(
            parse_gdb_value(" { foo = 27}").unwrap(),
            Node::Map(None, vec![("foo", Node::Leaf("27"))])
        );
        assert_eq!(
            parse_gdb_value("{ foo = 27, bar = 37}").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("27")), ("bar", Node::Leaf("37"))]
            )
        );
        assert_eq!(
            parse_gdb_value("{\n foo = 27,\n bar = 37\n }").unwrap(),
            Node::Map(
                None,
                vec![("foo", Node::Leaf("27")), ("bar", Node::Leaf("37"))]
            )
        );
        assert_eq!(
            parse_gdb_value("{{...}}").unwrap(),
            Node::Array(None, vec![Node::Array(None, vec![Node::Leaf("...")])])
        );
        assert_eq!(
            parse_gdb_value("{{}}").unwrap(),
            Node::Array(None, vec![Node::Array(None, vec![])])
        );
        assert_eq!(
            parse_gdb_value("{foo = 27, { bar=37}}").unwrap(),
            Node::Map(
                None,
                vec![
                    ("foo", Node::Leaf("27")),
                    (ANON_KEY, Node::Map(None, vec![("bar", Node::Leaf("37"))]))
                ]
            )
        );
        assert_eq!(
            parse_gdb_value("{{ bar=37}, foo = 27}").unwrap(),
            Node::Map(
                None,
                vec![
                    ("foo", Node::Leaf("27")),
                    (ANON_KEY, Node::Map(None, vec![("bar", Node::Leaf("37"))]))
                ]
            )
        );
    }

    #[test]
    fn test_parse_arrays() {
        assert_eq!(
            parse_gdb_value("{27}").unwrap(),
            Node::Array(None, vec![Node::Leaf("27")])
        );
        assert_eq!(
            parse_gdb_value("{ 27, 37}").unwrap(),
            Node::Array(None, vec![Node::Leaf("27"), Node::Leaf("37")])
        );
        assert_eq!(
            parse_gdb_value("{\n 27,\n 37\n}").unwrap(),
            Node::Array(None, vec![Node::Leaf("27"), Node::Leaf("37")])
        );
    }
}
