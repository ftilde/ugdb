use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ResultClass {
    Done,
    Running,
    Connected,
    Error,
    Exit,
}

#[derive(Debug)]
pub enum AsyncClass {
    Stopped,
    ThreadCreated,
    ThreadGroupStarted,
    ThreadExited,
    ThreadGroupExited,
    CmdParamChanged,
    LibraryLoaded,
    Other(String) //?
}

#[derive(Debug)]
pub enum AsyncKind {
    Exec,
    Status,
    Notify,
}


#[derive(Debug)]
pub enum StreamKind {
    Console,
    Target,
    Log,
}


#[derive(Debug)]
pub struct NamedValue {
    pub name: String,
    pub value: Value,
}

pub type NamedValues = BTreeMap<String, Value>;

#[derive(Debug)]
pub enum Value {
    Const(String),
    Tuple(NamedValues),
    ValueList(Vec<Value>),
    NamedValueList(NamedValues), //TODO: use a type alias for the map
}

impl Value {
    pub fn unwrap_const(self) -> String {
        if let Value::Const(string) = self {
            string
        } else {
            panic!("Value was not const");
        }
    }
    pub fn unwrap_tuple_or_named_value_list(self) -> NamedValues {
        match self {
            Value::Tuple(map) => map,
            Value::NamedValueList(map) => map,
            _ => panic!("Value was not tuple or named value list"),
        }
    }
    pub fn unwrap_valuelist(self) -> Vec<Value> {
        match self {
            Value::ValueList(list) => list,
            _ => panic!("Value was not value list"),
        }
    }
}

pub type Token = u64;

#[derive(Debug)]
pub struct ResultRecord {
    token: Option<Token>,
    pub class: ResultClass,
    pub results: NamedValues,
}

#[derive(Debug)]
pub enum OutOfBandRecord {
    AsyncRecord {
        token: Option<Token>,
        kind: AsyncKind,
        class: AsyncClass,
        results: NamedValues
    },
    StreamRecord {
        kind: StreamKind,
        data: String,
    },
}

#[derive(Debug)]
enum Output {
    Result(ResultRecord),
    OutOfBand(OutOfBandRecord),
    GDBLine,
    SomethingElse(String) /* Debug */
}

use nom::{IResult};
use std::io::{Read, BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use OutOfBandRecordSink;

pub fn process_output<T: Read, S: OutOfBandRecordSink>(output: T, result_pipe: Sender<ResultRecord>, out_of_band_pipe: S, is_running: Arc<AtomicBool>) {
    let mut reader = BufReader::new(output);

    //use std::fs::{File};
    //let mut f = File::create("/home/dominik/gdbmi.log").unwrap();

    loop {
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => { return; },
            Ok(_) => { /* TODO */
                //{
                //    use std::io::Write;
                //    write!(f, "{}", buffer).unwrap();
                //}
                match Output::parse(&buffer) {
                    Output::Result(record) => {
                        if let ResultRecord{token: _, class: ResultClass::Running, results: _} = record {
                            is_running.store(true, Ordering::Relaxed /*TODO: maybe something else? */);
                        }
                        result_pipe.send(record).expect("send result to pipe");
                    },
                    Output::OutOfBand(record) => {
                        if let OutOfBandRecord::AsyncRecord{token: _, kind: _, class: AsyncClass::Stopped, results: _} = record {
                            is_running.store(false, Ordering::Relaxed /*TODO: maybe something else? */);
                        }
                        out_of_band_pipe.send(record);
                    },
                    Output::GDBLine => { },
                    //Output::SomethingElse(_) => { /*println!("SOMETHING ELSE: {}", str);*/ }
                    Output::SomethingElse(text) => { out_of_band_pipe.send(OutOfBandRecord::StreamRecord{ kind: StreamKind::Target, data: text}); }
                }
            },
            Err(e) => { panic!("{}", e); },
        }
    }
}

impl Output {
    fn parse(line: &str) -> Self {
        match output(line.as_bytes() /* TODO str parsing? */) {
            IResult::Done(_, c) => { return c; },
            IResult::Incomplete(e) => { panic!("parsing line: incomplete {:?}", e) }, //Is it okay to read the next bytes then?
            IResult::Error(e) => { panic!("parse error: {}", e) }
        }
    }
}

named!(
    result_class<ResultClass>,
    alt!(
            value!(ResultClass::Done, tag!("done")) |
            value!(ResultClass::Running, tag!("running")) |
            value!(ResultClass::Connected, tag!("connected")) |
            value!(ResultClass::Error, tag!("error")) |
            value!(ResultClass::Exit, tag!("exit"))
        )
    );

fn not_bla(input: &[u8]) -> IResult<&[u8], u8> {
    let byte = input[0];
    if byte == b'\"' {
        IResult::Error(::nom::ErrorKind::Custom(1)) //what are we supposed to return here??
    } else {
        IResult::Done(&input[1..], byte)
    }
}

named!(
    escaped_character<u8>,
    alt!(
        value!(b'\n', tag!("\\n")) |
        value!(b'\t', tag!("\\t")) |
        value!(b'\"', tag!("\\\"")) |
        not_bla
        )
    );

named!(
    string<String>,
    chain!(
        tag!("\"") ~
        s: many0!(escaped_character) ~
        tag!("\""),
        || String::from_utf8_lossy(s.as_slice()).into_owned()
        )
    );

fn to_map(v: Vec<NamedValue>) -> NamedValues { //TODO: fix this and parse the map directly
    let mut map = BTreeMap::new();
    for e in v {
        map.insert(e.name, e.value);
    }
    map
}

named!(value<Value>,
       alt!(
           map!(string, |s| Value::Const(s)) |
           chain!(tag!("{") ~ results: separated_list!(tag!(","), result) ~ tag!("}"), || Value::Tuple(to_map(results))) |
           chain!(tag!("[") ~ values: separated_list!(tag!(","), value) ~ tag!("]"), || Value::ValueList(values)) |
           chain!(tag!("[") ~ results: separated_list!(tag!(","), result) ~ tag!("]"), || Value::NamedValueList(to_map(results)))
           )
       );

named!(
    result<NamedValue>,
    chain!(
        var: is_not!("=") ~
        tag!("=") ~
        val: value,
        || NamedValue {
            name: String::from_utf8_lossy(var).into_owned(),
            value: val,
        }
        )
    );

named!(
    result_record<Output>,
        chain!(
            tag!("^") ~
            c: result_class ~
            res: many0!(
                chain!(
                    tag!(",") ~
                    r: result,
                    || r
                    )
                ),
            || {
                Output::Result(ResultRecord {
                    token: None,
                    class: c,
                    results: to_map(res),
                })
            }
            )
    );

named!(
    async_kind<AsyncKind>,
    alt!(
            value!(AsyncKind::Exec, tag!("*")) |
            value!(AsyncKind::Status, tag!("+")) |
            value!(AsyncKind::Notify, tag!("="))
        )
    );

named!(
    async_class<AsyncClass>,
    alt!(
            value!(AsyncClass::Stopped, tag!("stopped")) |
            value!(AsyncClass::ThreadCreated, tag!("thread-created")) |
            value!(AsyncClass::ThreadGroupStarted, tag!("thread-group-started")) |
            value!(AsyncClass::ThreadExited, tag!("thread-exited")) |
            value!(AsyncClass::ThreadGroupExited, tag!("thread-group-exited")) |
            value!(AsyncClass::CmdParamChanged, tag!("cmd-param-changed")) |
            value!(AsyncClass::LibraryLoaded, tag!("library-loaded")) |
            map!(is_not!(","), |msg| AsyncClass::Other(String::from_utf8_lossy(msg).into_owned()))
        )
    );

named!(
    async_record<OutOfBandRecord>,
        chain!(
            //TODO: Token ~
            kind: async_kind ~
            class: async_class ~
            results: many0!(
                chain!(
                    tag!(",") ~
                    r: result,
                    || r
                    )
                ),
                || OutOfBandRecord::AsyncRecord {
                    token: None,
                    kind: kind,
                    class: class,
                    results: to_map(results),
                }
            )
    );

named!(
    stream_kind<StreamKind>,
    alt!(
            value!(StreamKind::Console, tag!("~")) |
            value!(StreamKind::Target, tag!("@")) |
            value!(StreamKind::Log, tag!("&"))
        )
    );

named!(
    stream_record<OutOfBandRecord>,
        chain!(
            kind: stream_kind ~
            msg: string,
            || OutOfBandRecord::StreamRecord {
                kind: kind,
                data: msg
            })
    );

named!(
    out_of_band_record<Output>,
        map!(
        alt!(
            stream_record |
            async_record
            ),
        |record| Output::OutOfBand(record)
        )
    );

named!(
    gdb_line<Output>,
        value!(Output::GDBLine, tag!("(gdb) ")) //TODO proper matching
    );

fn debug_line(i: &[u8]) -> IResult<&[u8], Output> {
    IResult::Done(i, Output::SomethingElse(String::from_utf8_lossy(i).into_owned()))
}

// Ends all records, but can probably ignored
//named!(
//    nl,
//    alt!(
//        tag!("\n") |
//        tag!("\r\n")
//        )
//    );

named!(
    output<Output>,
        alt!(
            result_record |
            out_of_band_record |
            gdb_line |
            debug_line
            )
    );
