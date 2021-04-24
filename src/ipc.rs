use unix_socket::{UnixListener, UnixStream};

use json;

use gdb::BreakpointOperationError;
use gdbmi::commands::{BreakPointLocation, MiCommand};
use gdbmi::ExecuteError;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;

struct IPCError {
    reason: &'static str,
    details: String,
}
impl IPCError {
    fn new<S: Into<String>>(reason: &'static str, details: S) -> Self {
        IPCError {
            reason: reason,
            details: details.into(),
        }
    }
    fn into_json(self) -> json::JsonValue {
        object! {
            "type" => "error",
            "reason" => self.reason,
            "details" => self.details
        }
    }
}

#[derive(Debug)]
pub struct IPCRequest {
    raw_request: Vec<u8>,
    response_channel: UnixStream,
}

impl IPCRequest {
    pub fn respond(mut self, p: ::UpdateParameters) {
        let reply = match Self::handle(p, self.raw_request) {
            Ok(reply_success) => reply_success,
            Err(reply_fail) => reply_fail.into_json(),
        };
        // Client may just close the channel, so we ignore any errors.
        // If they mess up it's on them.
        let _ = write_ipc_response(&mut self.response_channel, reply.dump().as_bytes());
    }

    fn handle(p: ::UpdateParameters, raw_request: Vec<u8>) -> Result<json::JsonValue, IPCError> {
        let str_request = ::std::str::from_utf8(raw_request.as_slice())
            .map_err(|_| IPCError::new("Malformed utf8.", ""))?;
        let json_request =
            json::parse(str_request).map_err(|_| IPCError::new("Malformed json.", str_request))?;

        let (function_name, parameters) = match json_request {
            json::JsonValue::Object(ref obj) => {
                let function_name = obj
                    .get("function")
                    .and_then(|o| o.as_str())
                    .ok_or(IPCError::new("Missing function name", json_request.dump()))?;

                let parameters = &obj["parameters"];

                (function_name, parameters)
            }
            _ => {
                return Err(IPCError::new(
                    "Malformed (non-object) request",
                    json_request.dump(),
                ));
            }
        };
        let result = Self::dispatch(function_name)?(p, parameters)?;

        Ok(object! {
            "type" => "success",
            "result" => result
        })
    }

    fn dispatch(
        function_name: &str,
    ) -> Result<
        fn(p: ::UpdateParameters, &json::JsonValue) -> Result<json::JsonValue, IPCError>,
        IPCError,
    > {
        match function_name {
            "set_breakpoint" => Ok(Self::set_breakpoint),
            "get_instance_info" => Ok(Self::get_instance_info),
            _ => Err(IPCError::new("unknown function", function_name)),
        }
    }

    fn set_breakpoint(
        p: ::UpdateParameters,
        parameters: &json::JsonValue,
    ) -> Result<json::JsonValue, IPCError> {
        let parameters_obj = if let &json::JsonValue::Object(ref parameters_obj) = parameters {
            parameters_obj
        } else {
            return Err(IPCError::new(
                "Parameters is not an object",
                parameters.dump(),
            ));
        };
        let file = parameters_obj
            .get("file")
            .and_then(|o| o.as_str())
            .ok_or(IPCError::new("Missing file name", parameters.dump()))?;
        let line = parameters_obj
            .get("line")
            .and_then(|o| o.as_u32())
            .ok_or(IPCError::new(
                "Missing integer line number",
                parameters.dump(),
            ))?;
        match p
            .gdb
            .insert_breakpoint(BreakPointLocation::Line(Path::new(file), line as usize))
        {
            Ok(()) => Ok(json::JsonValue::String(format!(
                "Inserted breakpoint at {}:{}",
                file, line
            ))),
            Err(BreakpointOperationError::Busy) => {
                //TODO: we may want to investigate if we can interrupt execution, insert
                //breakpoint, and resume execution thereafter.
                Err(IPCError::new("Could not insert breakpoint", "GDB is busy"))
            }
            Err(BreakpointOperationError::ExecutionError(msg)) => {
                //TODO: we may want to investigate if we can interrupt execution, insert
                //breakpoint, and resume execution thereafter.
                Err(IPCError::new("Could not insert breakpoint:", msg))
            }
        }
    }

    fn get_instance_info(
        p: ::UpdateParameters,
        _: &json::JsonValue,
    ) -> Result<json::JsonValue, IPCError> {
        let result = p
            .gdb
            .mi
            .execute(MiCommand::environment_pwd())
            .map_err(|e| match e {
                ExecuteError::Busy => {
                    //TODO: we may want to investigate if we can interrupt execution, get information
                    //and resume execution thereafter.
                    IPCError::new("Could not get working directory", "GDB is busy")
                }
                ExecuteError::Quit => IPCError::new("Could not get working directory", "GDB quit"),
            })?;
        let working_directory = result.results["cwd"].as_str().ok_or_else(|| {
            IPCError::new("Could not get working directory", "Malformed GDB response")
        })?;
        Ok(object! {
            "working_directory" => working_directory
        })
    }
}

const FALLBACK_RUNTIME_DIR: &'static str = "/tmp/";
const RUNTIME_SUBDIR: &'static str = "ugdb";
const SOCKET_IDENTIFIER_LENGTH: usize = 64;
const IPC_MSG_IDENTIFIER: &'static [u8] = b"ugdb-ipc";
const HEADER_LENGTH: usize = 12;

pub struct IPC {
    socket_path: PathBuf,
}

fn write_ipc_header<W: Write>(w: &mut W, msg_len: u32) -> ::std::io::Result<()> {
    let msg_len = msg_len.to_le();
    let msg_len_buf = [
        msg_len as u8,
        (msg_len >> 8) as u8,
        (msg_len >> 16) as u8,
        (msg_len >> 24) as u8,
    ];
    w.write_all(IPC_MSG_IDENTIFIER)?;
    w.write_all(&msg_len_buf)?;
    Ok(())
}

fn write_ipc_response<W: Write>(w: &mut W, msg: &[u8]) -> ::std::io::Result<()> {
    write_ipc_header(w, msg.len() as u32)?;
    w.write_all(msg)?;
    Ok(())
}

fn try_read_ipc_header<R: Read>(r: &mut R) -> Result<u32, ()> {
    // The header has to look like this with four bytes (****) for the message length in little endian: "ugdb-ipc****"
    let mut buf = vec![0u8; HEADER_LENGTH];
    r.read_exact(&mut buf).map_err(|_| {})?;
    if &buf[0..8] == IPC_MSG_IDENTIFIER {
        let mut len = 0;
        len += buf[8] as u32;
        len += (buf[9] as u32) << 8;
        len += (buf[10] as u32) << 16;
        len += (buf[11] as u32) << 24;
        Ok(u32::from_le(len))
    } else {
        Err(())
    }
}

fn try_read_ipc_request(connection: &mut UnixStream) -> Result<IPCRequest, ()> {
    let msg_len = try_read_ipc_header(connection)?;

    let mut msg_buf = vec![0u8; msg_len as usize];
    connection.read_exact(&mut msg_buf).map_err(|_| {})?;
    Ok(IPCRequest {
        raw_request: msg_buf,
        response_channel: connection.try_clone().expect("clone handle"),
    })
}

fn start_connection(mut connection: UnixStream, request_sink: std::sync::mpsc::Sender<::Event>) {
    let _ = thread::Builder::new()
        .name("IPC Connection".to_owned())
        .spawn(move || {
            connection.set_nonblocking(false).expect("set blocking");

            loop {
                match try_read_ipc_request(&mut connection) {
                    Ok(request) => {
                        request_sink.send(::Event::Ipc(request)).unwrap();
                    }
                    Err(_) => {
                        // If you don't play nicely, we don't want to talk:
                        break;
                    }
                }
            }
        });
}

impl IPC {
    pub fn setup(request_sink: std::sync::mpsc::Sender<::Event>) -> ::std::io::Result<Self> {
        let runtime_dir =
            ::std::env::var_os("XDG_RUNTIME_DIR").unwrap_or(OsString::from(FALLBACK_RUNTIME_DIR));
        let ugdb_dir = Path::join(runtime_dir.as_ref(), RUNTIME_SUBDIR);
        let _ = fs::create_dir(&ugdb_dir); //Ignore error if dir exists, we check if we can access it soon.

        use rand::Rng;
        let socket_name = ::rand::thread_rng()
            .gen_ascii_chars()
            .take(SOCKET_IDENTIFIER_LENGTH)
            .collect::<String>();
        let socket_path = ugdb_dir.join(socket_name);

        let listener = UnixListener::bind(&socket_path)?;

        let _ = thread::Builder::new()
            .name("IPC Connection Listener".to_owned())
            .spawn(move || {
                for connection in listener.incoming() {
                    if let Ok(connection) = connection {
                        start_connection(connection, request_sink.clone());
                    }
                }
            });

        Ok(IPC {
            socket_path: socket_path,
        })
    }
}

impl ::std::ops::Drop for IPC {
    fn drop(&mut self) {
        // We at least try to remove the socket. If it fails we cannot really do about it here.
        let _ = fs::remove_file(&self.socket_path);
    }
}
