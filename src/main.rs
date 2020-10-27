//! An interactive serial terminal

use std::convert::TryFrom;
use std::path::PathBuf;
use std::{io, str};

use bytes::{BufMut, BytesMut};
use futures::stream::StreamExt;
use serialport::{FlowControl, Parity, StopBits};
use structopt::clap::AppSettings;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames};
use tokio_serial::DataBits;
use tokio_util::codec::{Decoder, Encoder};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

#[derive(Debug, StructOpt)]
#[structopt(about)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Opt {
    /// Baud rate
    #[structopt(short, long, default_value = "921600")]
    baud: u32,
    /// Data bits (5, 6, 7, 8)
    #[structopt(short, long, default_value = "8")]
    data_bits: usize,
    /// End of line transformation (cr, lf, crlf)
    #[structopt(long, default_value = "crlf")]
    eol: Eol,
    /// Flow control (none, software, hardware)
    #[structopt(long, default_value = "none")]
    flow_control: FlowControlOpt,
    /// Lists available serial ports
    #[structopt(short, long)]
    list: bool,
    /// Parity checking (none, odd, even)
    #[structopt(long, default_value = "none")]
    parity: ParityOpt,
    /// Stop bits (1, 2)
    #[structopt(long, default_value = "1")]
    stop_bits: usize,
    /// Path to the serial device
    #[structopt(short, long)]
    tty: Option<PathBuf>,
}

struct DataBitsExt(DataBits);

impl TryFrom<usize> for DataBitsExt {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(Self(DataBits::Five)),
            6 => Ok(Self(DataBits::Six)),
            7 => Ok(Self(DataBits::Seven)),
            8 => Ok(Self(DataBits::Eight)),
            _ => Err(()),
        }
    }
}

/// End of line character options
#[derive(Debug, EnumString, EnumVariantNames, StructOpt)]
#[strum(serialize_all = "snake_case")]
enum Eol {
    /// Carriage return
    Cr,
    /// Carriage return, line feed
    Crlf,
    /// Line feed
    Lf,
}

impl Eol {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Cr => &b"\r"[..],
            Self::Crlf => &b"\r\n"[..],
            Self::Lf => &b"\n"[..],
        }
    }
}

/// Flow control modes
#[derive(Debug, EnumString, EnumVariantNames, StructOpt)]
#[strum(serialize_all = "snake_case")]
enum FlowControlOpt {
    /// No flow control.
    None,
    /// Flow control using XON/XOFF bytes.
    Software,
    /// Flow control using RTS/CTS signals.
    Hardware,
}

impl From<FlowControlOpt> for FlowControl {
    fn from(opt: FlowControlOpt) -> Self {
        match opt {
            FlowControlOpt::None => FlowControl::None,
            FlowControlOpt::Software => FlowControl::Software,
            FlowControlOpt::Hardware => FlowControl::Hardware,
        }
    }
}

#[derive(Debug, EnumString, EnumVariantNames, StructOpt)]
#[strum(serialize_all = "snake_case")]
enum ParityOpt {
    /// No parity bit.
    None,
    /// Parity bit sets odd number of 1 bits.
    Odd,
    /// Parity bit sets even number of 1 bits.
    Even,
}

impl From<ParityOpt> for Parity {
    fn from(opt: ParityOpt) -> Self {
        match opt {
            ParityOpt::None => Parity::None,
            ParityOpt::Odd => Parity::Odd,
            ParityOpt::Even => Parity::Even,
        }
    }
}

struct StopBitsExt(StopBits);

impl TryFrom<usize> for StopBitsExt {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self(StopBits::One)),
            2 => Ok(Self(StopBits::Two)),
            _ => Err(()),
        }
    }
}

struct SerialReadCodec;

impl Decoder for SerialReadCodec {
    type Item = String;
    type Error = LinesCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.trim_end().to_string())),
                Err(_) => Err(LinesCodecError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid String",
                ))),
            };
        }

        Ok(None)
    }
}

struct SerialWriteCodec(Eol);

impl Encoder<String> for SerialWriteCodec {
    type Error = LinesCodecError;

    fn encode(&mut self, line: String, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let eol = self.0.bytes();
        buf.reserve(line.len() + eol.len());
        buf.put(line.as_bytes());
        buf.put(eol);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let ports = serialport::available_ports().unwrap();

    if ports.is_empty() {
        eprintln!("No serial ports found!");
        std::process::exit(1);
    }

    if opt.list {
        for port in ports {
            println!("{:#?}", port);
        }
        return;
    }

    let tty_path = opt
        .tty
        .unwrap_or_else(|| PathBuf::from(&ports.first().unwrap().port_name));

    let settings = tokio_serial::SerialPortSettings {
        baud_rate: opt.baud,
        data_bits: DataBitsExt::try_from(opt.data_bits).unwrap().0,
        flow_control: opt.flow_control.into(),
        parity: opt.parity.into(),
        stop_bits: StopBitsExt::try_from(opt.stop_bits).unwrap().0,
        timeout: std::time::Duration::from_secs(5),
    };

    println!("Opening serial connection to device {:?}", tty_path);
    let serial = tokio_serial::Serial::from_path(tty_path, &settings);
    if serial.is_err() {
        eprintln!("Failed to create serial port: {:?}", serial.err());
        std::process::exit(1);
    }
    #[allow(unused_mut)]
    let mut serial = serial.unwrap();

    #[cfg(unix)]
    serial
        .set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let stdout = tokio::io::stdout();
    let stdin = tokio::io::stdin();
    let framed_stdin = FramedRead::new(stdin, LinesCodec::new());
    let framed_stdout = FramedWrite::new(stdout, LinesCodec::new());

    let (read, write) = tokio::io::split(serial);
    let stream = FramedRead::new(read, SerialReadCodec);
    let sink = FramedWrite::new(write, SerialWriteCodec(opt.eol));

    let input = framed_stdin.forward(sink);
    let output = stream.forward(framed_stdout);
    let result = futures::future::try_join(input, output).await;

    eprintln!("Uh oh: {:?}", result);
}
