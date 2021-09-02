use ::Event;
use serial_port::prelude::*;
use serial_port::{CharSize, FlowControl, Parity, StopBits};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::Duration;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct SerialOptions {
    pub device: Option<PathBuf>,
    pub baud_rate: usize,
    pub data_bits: CharSize,
    pub parity_bit: Parity,
    pub stop_bits: StopBits,
    pub flow_control: FlowControl,
}

pub fn parse_data_bits(inp: &str) -> Result<CharSize, Box<dyn std::error::Error>> {
    match inp {
        "5" => Ok(CharSize::Bits5),
        "6" => Ok(CharSize::Bits6),
        "7" => Ok(CharSize::Bits7),
        "8" => Ok(CharSize::Bits8),
        _ => Err("Valid values for data bits are: 5, 6, 7, and 8.".into()),
    }
}

pub fn parse_parity_bit(inp: &str) -> Result<Parity, Box<dyn std::error::Error>> {
    match inp {
        "none" => Ok(Parity::ParityNone),
        "even" => Ok(Parity::ParityEven),
        "odd" => Ok(Parity::ParityOdd),
        _ => Err("Valid values for parity bit are: \"none\", \"even\", and \"odd\".".into()),
    }
}

pub fn parse_stop_bits(inp: &str) -> Result<StopBits, Box<dyn std::error::Error>> {
    match inp {
        "1" => Ok(StopBits::Stop1),
        "2" => Ok(StopBits::Stop2),
        _ => Err("Valid values for stop bits are: 1 and 2.".into()),
    }
}

pub fn parse_flow_control(inp: &str) -> Result<FlowControl, Box<dyn std::error::Error>> {
    match inp {
        "none" => Ok(FlowControl::FlowNone),
        "software" => Ok(FlowControl::FlowSoftware),
        "hardware" => Ok(FlowControl::FlowHardware),
        _ => Err("Valid values for flow control are: \"none\", \"software\" (for XON/XOFF), and \"hardware\" (for RTS/CTS)".into()),
    }
}

fn open_serial_port(options: &SerialOptions) -> Result<serial::SystemPort, serial::Error> {
    let mut port = serial::open(options.device.as_ref().unwrap())?;
    port.reconfigure(&|settings| {
        let baud_rate = serial::BaudRate::from_speed(options.baud_rate);
        settings.set_baud_rate(baud_rate)?;
        settings.set_char_size(options.data_bits);
        settings.set_parity(options.parity_bit);
        settings.set_stop_bits(options.stop_bits);
        settings.set_flow_control(options.flow_control);
        Ok(())
    })?;

    port.set_timeout(Duration::from_secs(1))?;

    Ok(port)
}

pub fn output_to_pty(options: SerialOptions, output: &OsStr, sink: Sender<Event>) {
    let mut pty = match OpenOptions::new().write(true).open(output) {
        Ok(pty) => pty,
        Err(e) => {
            sink.send(Event::Log(format!("Could not open PTY: {}", e)))
                .expect("send");
            return;
        }
    };

    let device = options.device.as_ref().unwrap();
    let mut buf = vec![0u8; 4096];
    let mut port = match open_serial_port(&options) {
        Ok(port) => port,
        Err(e) => {
            if let Err(e) =
                pty.write_all(format!("Could not open terminal {:?}: {}\n", device, e).as_bytes())
            {
                sink.send(Event::Log(format!("Could not write to PTY: {}", e)))
                    .expect("send");
            }
            return;
        }
    };

    loop {
        if let Ok(bytes) = port.read(&mut buf) {
            if let Err(e) = pty.write_all(&buf[..bytes]) {
                sink.send(Event::Log(format!("Could not write to PTY: {}", e)))
                    .expect("send");
            }
        }
    }
}
