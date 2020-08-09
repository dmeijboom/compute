use std::io;
use std::fmt;
use std::error;
use std::result;
use std::str::Utf8Error;

use tokio::task::JoinError;
use rusoto_core::RusotoError;
use rusoto_s3::GetObjectError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    IOError(io::Error),
    Utf8Error(Utf8Error),
    TokioError(JoinError),
    TeraError(tera::Error),
    Json5Error(json5::Error),
    ReqwestError(reqwest::Error),
    RusotoError(RusotoError<GetObjectError>),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(message) => write!(f, "unexpected error: {}", message),
            Self::IOError(e) => write!(f, "IO error: {}", e),
            Self::Utf8Error(e) => match e.error_len() {
                Some(index) => write!(f, "non UTF-8 byte found in string at {}", index),
                None => write!(f, "unexpected EOF for UTF-8 string"),
            },
            Self::TokioError(e) => if e.is_cancelled() {
                write!(f, "failed to complete task: received cancellation")
            } else if e.is_panic() {
                write!(f, "failed to complete task: unexpected panic")
            } else {
                write!(f, "failed to complete task: {}", e)
            },
            Self::TeraError(e) => match &e.kind {
                tera::ErrorKind::Msg(msg) => write!(f, "failed to compile template: {}", msg),
                tera::ErrorKind::CircularExtend { .. } => write!(f, "failed to compile template: circular extend"),
                tera::ErrorKind::MissingParent { .. } => write!(f, "failed to compile template: missing parenthesis"),
                tera::ErrorKind::TemplateNotFound(name) => write!(f, "failed to compile template: template {} does not exist", name),
                tera::ErrorKind::FilterNotFound(name) => write!(f, "failed to compile template: filter {} does not exist", name),
                tera::ErrorKind::TestNotFound(name) => write!(f, "failed to compile template: test {} does not exist", name),
                tera::ErrorKind::FunctionNotFound(msg) => write!(f, "failed to compile template: function {} does not exist", msg),
                tera::ErrorKind::InvalidMacroDefinition(name) => write!(f, "failed to compile template: invalid macro {}", name),
                tera::ErrorKind::Json(e) => write!(f, "failed to compile template: {}", e),
                tera::ErrorKind::CallFunction(name) => write!(f, "failed to compile template: error while calling {}()", name),
                tera::ErrorKind::CallFilter(name) => write!(f, "failed to compile template: error while calling filter {}", name),
                tera::ErrorKind::CallTest(name) => write!(f, "failed to compile template: error while calling test {}", name),
                _ => panic!("invalid tera error"),
            },
            Self::Json5Error(e) => match e {
                json5::Error::Message(msg) => write!(f, "json5 error: {}", msg),
            },
            Self::ReqwestError(e) => match e.status() {
                Some(code) => write!(f, "invalid status code {} from response: {}", code.as_str(), e),
                None => write!(f, "reqwest error: {}", e),
            },
            Self::RusotoError(e) => write!(f, "failed to download file from S3: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Self {
        Error::TeraError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<json5::Error> for Error {
    fn from(err: json5::Error) -> Self {
        Error::Json5Error(err)
    }
}

impl From<RusotoError<GetObjectError>> for Error {
    fn from(err: RusotoError<GetObjectError>) -> Self {
        Error::RusotoError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8Error(err)
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Error::TokioError(err)
    }
}
