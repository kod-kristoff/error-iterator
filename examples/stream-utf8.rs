use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read};

use error_iterator::{
    io::EIteratorIoExt,
    utf8::{DecodeUtf8Error, EIteratorUtf8Ext},
    EIterator, ToEIter,
};

fn main() -> Result<(), MyAppError> {
    let mapper: HashMap<char, char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .zip("ДВСDЁҒGНІЈКLМПОРQЯЅТЦЏШХЧZавсdёfgніјкlмпорqгѕтцѵшхчz".chars())
        .collect();
    let convert_char = |c| *mapper.get(&c).unwrap_or(&c);

    let stdin = io::stdin();
    let stdout = io::stdout();

    stdin
        .lock()
        .bytes()
        .eiter()
        .map_error(MyAppError::IOError)
        .decode_utf8()
        .map(convert_char)
        .encode_utf8()
        .write_to(stdout.lock())?;

    Ok(())
}

#[derive(Debug)]
pub enum MyAppError {
    IOError(std::io::Error),
    DecodeUtf8Error(DecodeUtf8Error),
}
impl From<std::io::Error> for MyAppError {
    fn from(e: std::io::Error) -> MyAppError {
        MyAppError::IOError(e)
    }
}
impl From<DecodeUtf8Error> for MyAppError {
    fn from(e: DecodeUtf8Error) -> MyAppError {
        MyAppError::DecodeUtf8Error(e)
    }
}
impl std::fmt::Display for MyAppError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MyAppError::IOError(e) => e.fmt(fmt),
            MyAppError::DecodeUtf8Error(e) => e.fmt(fmt),
        }
    }
}
impl Error for MyAppError {
    fn description(&self) -> &str {
        "MyAppError"
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyAppError::IOError(e) => Some(e),
            MyAppError::DecodeUtf8Error(e) => Some(e),
        }
    }
}
