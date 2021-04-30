use super::lexer::Span;
use super::GDBValue;

pub const ANON_KEY: &'static str = "*anon*";
pub const EMPTY_SPAN: Span = (0, 0);

pub enum Node {
    Leaf(Span),
    Array(Vec<Node>),
    Map(Vec<(Span, Node)>),
}

impl Node {
    pub fn to_value(&self, origin: &str) -> GDBValue {
        match self {
            &Node::Leaf(span) => {
                let s = &origin[span.0..span.1];
                if let Ok(i) = parse_int::parse(s) {
                    GDBValue::Integer(s.to_owned(), i)
                } else {
                    GDBValue::String(s.to_owned())
                }
            }
            &Node::Array(ref nodes) => {
                GDBValue::Array(nodes.iter().map(|n| n.to_value(origin)).collect::<Vec<_>>())
            }
            &Node::Map(ref map_elements) => {
                let mut o = Vec::new();
                let mut anons = Vec::new();
                for &(ref keyspan, ref val) in map_elements.iter() {
                    let val = val.to_value(origin);
                    if *keyspan == EMPTY_SPAN {
                        anons.push(val);
                    } else {
                        o.push((origin[keyspan.0..keyspan.1].to_owned(), val));
                    };
                }

                match (o.is_empty(), anons.len()) {
                    (true, 0) => GDBValue::Map(Vec::new()),
                    (true, _) => GDBValue::Array(anons),
                    (false, 0) => GDBValue::Map(o),
                    (false, 1) => {
                        o.push((ANON_KEY.to_owned(), anons.drain(..).next().unwrap()));
                        GDBValue::Map(o)
                    }
                    (false, _) => {
                        o.push((ANON_KEY.to_owned(), GDBValue::Array(anons)));
                        GDBValue::Map(o)
                    }
                }
            }
        }
    }
}

pub fn build_vec<T>(mut v: Vec<T>, a: T) -> Vec<T> {
    v.push(a);
    v
}
