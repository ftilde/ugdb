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
pub struct Result {
    variable: String,
    value: Value,
}

#[derive(Debug)]
pub enum Value {
    Const(String),
    Tuple(Vec<Result>),
    ValueList(Vec<Value>),
    ResultList(Vec<Result>),
}

pub type Token = u64;

#[derive(Debug)]
pub struct ResultRecord {
    token: Option<Token>,
    class: ResultClass,
    results: Vec<Result>,
}

#[derive(Debug)]
pub enum OutOfBandRecord {
    AsyncRecord {
        token: Option<Token>,
        kind: AsyncKind,
        class: AsyncClass,
        results: Vec<Result>
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

use ::nom::{IResult};
use ::std::io::{Read, BufRead, BufReader};
use ::std::sync::mpsc::Sender;
use ::std::sync::Arc;
use ::std::sync::atomic::{AtomicBool, Ordering};

pub fn process_output<T: Read>(output: T, result_pipe: Sender<ResultRecord>, out_of_band_pipe: Sender<OutOfBandRecord>, is_running: Arc<AtomicBool>) {
    let mut reader = BufReader::new(output);
    loop {
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => { return; },
            Ok(_) => { /* TODO */
                //println!("::::: {:?}", buffer);
                match Output::parse(&buffer) {
                    Output::Result(record) => {
                        if let ResultRecord{token: _, class: ResultClass::Running, results: _} = record {
                            is_running.store(true, Ordering::Relaxed /*TODO: maybe something else? */);
                        }
                        result_pipe.send(record).unwrap();
                    },
                    Output::OutOfBand(record) => {
                        if let OutOfBandRecord::AsyncRecord{token: _, kind: _, class: AsyncClass::Stopped, results: _} = record {
                            is_running.store(false, Ordering::Relaxed /*TODO: maybe something else? */);
                        }
                        out_of_band_pipe.send(record).unwrap();
                    },
                    Output::GDBLine => { },
                    //Output::SomethingElse(_) => { /*println!("SOMETHING ELSE: {}", str);*/ }
                    Output::SomethingElse(text) => { out_of_band_pipe.send(OutOfBandRecord::StreamRecord{ kind: StreamKind::Target, data: text}).unwrap(); }
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


named!(
    string<String>,
    chain!(
        tag!("\"") ~
        s: is_not!("\"")~
        tag!("\""),
        || String::from_utf8_lossy(s).into_owned()
        )
    );

named!(value<Value>,
       alt!(
           map!(string, |s| Value::Const(s)) |
           chain!(tag!("{") ~ results: separated_list!(tag!(","), result) ~ tag!("}"), || Value::Tuple(results)) |
           chain!(tag!("[") ~ values: separated_list!(tag!(","), value) ~ tag!("]"), || Value::ValueList(values)) |
           chain!(tag!("[") ~ results: separated_list!(tag!(","), result) ~ tag!("]"), || Value::ResultList(results))
           )
       );

named!(
    result<Result>,
    chain!(
        var: is_not!("=") ~
        tag!("=") ~
        val: value,
        || Result {
            variable: String::from_utf8_lossy(var).into_owned(),
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
                    results: res,
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
                    results: results,
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
        value!(Output::GDBLine, tag!("(gdb) \n")) //TODO proper matching
    );

fn debug_line(i: &[u8]) -> IResult<&[u8], Output> {
    IResult::Done(i, Output::SomethingElse(String::from_utf8_lossy(i).into_owned()))
}

named!(
    output<Output>,
        alt!(
            result_record |
            out_of_band_record |
            gdb_line |
            debug_line
            )
    );
