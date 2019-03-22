use super::Token;
pub use json::object::Object;
pub use json::JsonValue;

use log::{error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultClass {
    Done,
    Running,
    Connected,
    Error,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakPointEvent {
    Created,
    Deleted,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadEvent {
    Created,
    GroupStarted,
    Exited,
    GroupExited,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncClass {
    Stopped,
    CmdParamChanged,
    LibraryLoaded,
    Thread(ThreadEvent),
    BreakPoint(BreakPointEvent),
    Other(String), //?
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
pub struct ResultRecord {
    pub(crate) token: Option<Token>,
    pub class: ResultClass,
    pub results: Object,
}

#[derive(Debug)]
pub enum OutOfBandRecord {
    AsyncRecord {
        token: Option<Token>,
        kind: AsyncKind,
        class: AsyncClass,
        results: Object,
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
    SomethingElse(String), /* Debug */
}

use nom::IResult;
use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use OutOfBandRecordSink;

pub fn process_output<T: Read, S: OutOfBandRecordSink>(
    output: T,
    result_pipe: Sender<ResultRecord>,
    out_of_band_pipe: S,
    is_running: Arc<AtomicBool>,
) {
    let mut reader = BufReader::new(output);

    loop {
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                return;
            }
            Ok(_) => {
                info!("{}", buffer.trim_end());

                let parse_result = match Output::parse(&buffer) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("PARSING ERROR: {}", e);
                        continue;
                    }
                };
                match parse_result {
                    Output::Result(record) => {
                        match record.class {
                            ResultClass::Running => is_running.store(true, Ordering::SeqCst),
                            //Apparently sometimes gdb first claims to be running, only to then stop again (without notifying the user)...
                            ResultClass::Error => is_running.store(false, Ordering::SeqCst),
                            _ => {}
                        }
                        result_pipe.send(record).expect("send result to pipe");
                    }
                    Output::OutOfBand(record) => {
                        if let OutOfBandRecord::AsyncRecord {
                            class: AsyncClass::Stopped,
                            ..
                        } = record
                        {
                            is_running.store(false, Ordering::SeqCst);
                        }
                        out_of_band_pipe.send(record);
                    }
                    Output::GDBLine => {}
                    //Output::SomethingElse(_) => { /*println!("SOMETHING ELSE: {}", str);*/ }
                    Output::SomethingElse(text) => {
                        out_of_band_pipe.send(OutOfBandRecord::StreamRecord {
                            kind: StreamKind::Target,
                            data: text,
                        });
                    }
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

impl Output {
    fn parse(line: &str) -> Result<Self, String> {
        match output(line.as_bytes()) {
            IResult::Done(_, c) => Ok(c),
            IResult::Incomplete(e) => Err(format!("parsing line: incomplete {:?}", e)), //Is it okay to read the next bytes then?
            IResult::Error(e) => Err(format!("parse error: {}", e)),
        }
    }
}

named!(
    result_class<ResultClass>,
    alt!(
        value!(ResultClass::Done, tag!("done"))
            | value!(ResultClass::Running, tag!("running"))
            | value!(ResultClass::Connected, tag!("connected"))
            | value!(ResultClass::Error, tag!("error"))
            | value!(ResultClass::Exit, tag!("exit"))
    )
);

fn non_quote_byte(input: &[u8]) -> IResult<&[u8], u8> {
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
        value!(b'\n', tag!("\\n"))
            | value!(b'\r', tag!("\\r"))
            | value!(b'\t', tag!("\\t"))
            | value!(b'\"', tag!("\\\""))
            | value!(b'\\', tag!("\\\\"))
            | non_quote_byte
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

fn to_map(v: Vec<(String, JsonValue)>) -> Object {
    //TODO: fix this and parse the map directly
    let mut obj = Object::new();
    for (name, value) in v {
        debug_assert!(obj.get(&name).is_none(), "Duplicate object member!");
        obj.insert(&name, value);
    }
    obj
}

fn to_list(v: Vec<(String, JsonValue)>) -> Vec<JsonValue> {
    //The gdbmi-grammar is really weird...
    //TODO: fix this and parse the map directly
    v.into_iter().map(|(_, value)| value).collect()
}

named!(
    value<JsonValue>,
    alt!(
        map!(string, |s| JsonValue::String(s))
            | chain!(tag!("{") ~ results: separated_list!(tag!(","), result) ~ tag!("}"), || JsonValue::Object(to_map(results)))
            | chain!(tag!("[") ~ values: separated_list!(tag!(","), value) ~ tag!("]"), || JsonValue::Array(values))
            | chain!(tag!("[") ~ results: separated_list!(tag!(","), result) ~ tag!("]"), || JsonValue::Array(to_list(results)))
    )
);

// Don't even ask... Against its spec, gdb(mi) sometimes emits multiple values for a single tuple
// in a comma separated list.
named!(
    buggy_gdb_list_in_result<JsonValue>,
    map!(separated_list!(tag!(","), value), |values: Vec<
        JsonValue,
    >| {
        if values.len() == 1 {
            values
                .into_iter()
                .next()
                .expect("len == 1 => first element is guaranteed")
        } else {
            JsonValue::Array(values)
        }
    })
);

named!(
    result<(String, JsonValue)>,
    chain!(
        var: is_not!("={}" /* Do not allow =, {, nor } */) ~
        tag!("=") ~
        val: buggy_gdb_list_in_result,
        || (String::from_utf8_lossy(var).into_owned(), val))
);

named!(
    token<Token>,
    map!(::nom::digit, |values: &[u8]| values
        .iter()
        .fold(0, |acc, &ascii_digit| 10 * acc
            + (ascii_digit - b'0') as u64))
);

named!(
    result_record<Output>,
    chain!(
    t: opt!(token) ~
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
            token: t,
            class: c,
            results: to_map(res),
        })
    }
    )
);

named!(
    async_kind<AsyncKind>,
    alt!(
        value!(AsyncKind::Exec, tag!("*"))
            | value!(AsyncKind::Status, tag!("+"))
            | value!(AsyncKind::Notify, tag!("="))
    )
);

named!(
    async_class<AsyncClass>,
    alt!(
        value!(AsyncClass::Stopped, tag!("stopped"))
            | value!(
                AsyncClass::Thread(ThreadEvent::Created),
                tag!("thread-created")
            )
            | value!(
                AsyncClass::Thread(ThreadEvent::GroupStarted),
                tag!("thread-group-started")
            )
            | value!(
                AsyncClass::Thread(ThreadEvent::Exited),
                tag!("thread-exited")
            )
            | value!(
                AsyncClass::Thread(ThreadEvent::GroupExited),
                tag!("thread-group-exited")
            )
            | value!(
                AsyncClass::Thread(ThreadEvent::Selected),
                tag!("thread-selected")
            )
            | value!(AsyncClass::CmdParamChanged, tag!("cmd-param-changed"))
            | value!(AsyncClass::LibraryLoaded, tag!("library-loaded"))
            | value!(
                AsyncClass::BreakPoint(BreakPointEvent::Created),
                tag!("breakpoint-created")
            )
            | value!(
                AsyncClass::BreakPoint(BreakPointEvent::Deleted),
                tag!("breakpoint-deleted")
            )
            | value!(
                AsyncClass::BreakPoint(BreakPointEvent::Modified),
                tag!("breakpoint-modified")
            )
            | map!(is_not!(","), |msg| AsyncClass::Other(
                String::from_utf8_lossy(msg).into_owned()
            ))
    )
);

named!(
    async_record<OutOfBandRecord>,
    chain!(
    t: opt!(token) ~
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
            token: t,
            kind: kind,
            class: class,
            results: to_map(results),
        }
    )
);

named!(
    stream_kind<StreamKind>,
    alt!(
        value!(StreamKind::Console, tag!("~"))
            | value!(StreamKind::Target, tag!("@"))
            | value!(StreamKind::Log, tag!("&"))
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
    map!(alt!(stream_record | async_record), |record| {
        Output::OutOfBand(record)
    })
);

named!(
    gdb_line<Output>,
    value!(Output::GDBLine, tag!("(gdb) ")) //TODO proper matching
);

fn debug_line(i: &[u8]) -> IResult<&[u8], Output> {
    IResult::Done(
        i,
        Output::SomethingElse(String::from_utf8_lossy(i).into_owned()),
    )
}

// Ends all records, but can probably ignored
named!(nl, alt!(tag!("\n") | tag!("\r\n")));

named!(
    output<Output>,
    chain!(
    output: alt!(
        result_record |
        out_of_band_record |
        gdb_line |
        debug_line
        ) ~
    nl,
    || output
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_output() {
        let _ = Output::parse("=library-loaded,ranges=[{}]\n");
    }
}
