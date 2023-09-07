use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    PacketTooLong,
    PacketTooShort,
    ValueOutOfRange,
    NotSupported(String),
    InvalidString,
    InvalidSeq,
    WriteFailure,
    ValidationError(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::PacketTooLong => formatter.write_str("packet too long"),
            Error::PacketTooShort => formatter.write_str("packet too short"),
            Error::ValueOutOfRange => formatter.write_str("value out of range"),
            Error::NotSupported(datatype) => {
                formatter.write_str(format!("datatype {} not supported", datatype).as_str())
            }
            Error::InvalidString => formatter.write_str("invalid string (not valid UTF-8)"),
            Error::InvalidSeq => formatter.write_str("invalid sequence (no VecLen used)"),
            Error::WriteFailure => formatter.write_str("failed to write to buffer"),
            Error::ValidationError(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}
