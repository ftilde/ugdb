use super::lexer::Span;
use json::{object, JsonValue};

pub const ANON_KEY: &'static str = "*anon*";
pub const EMPTY_SPAN: Span = (0, 0);

pub enum Node {
    Leaf(Span),
    Array(Vec<Node>),
    Map(Vec<(Span, Node)>),
}

impl Node {
    pub fn to_json(&self, origin: &str) -> JsonValue {
        match self {
            &Node::Leaf(span) => match &origin[span.0..span.1] {
                "true" => JsonValue::Boolean(true),
                "false" => JsonValue::Boolean(false),
                other => JsonValue::String(other.to_owned()),
            },
            &Node::Array(ref nodes) => {
                JsonValue::Array(nodes.iter().map(|n| n.to_json(origin)).collect::<Vec<_>>())
            }
            &Node::Map(ref map_elements) => {
                let mut o = object::Object::new();
                let mut anons = Vec::new();
                for &(ref keyspan, ref val) in map_elements.iter() {
                    let jsonval = val.to_json(origin);
                    if *keyspan == EMPTY_SPAN {
                        anons.push(jsonval);
                    } else {
                        o.insert(&origin[keyspan.0..keyspan.1], jsonval);
                    };
                }

                match (o.is_empty(), anons.len()) {
                    (true, 0) => object! {},
                    (true, _) => JsonValue::Array(anons),
                    (false, 0) => JsonValue::Object(o),
                    (false, 1) => {
                        o.insert(ANON_KEY, anons.drain(..).next().unwrap());
                        JsonValue::Object(o)
                    }
                    (false, _) => {
                        o.insert(ANON_KEY, JsonValue::Array(anons));
                        JsonValue::Object(o)
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
