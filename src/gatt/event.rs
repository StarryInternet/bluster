use std::fmt;

use futures::channel::{mpsc, oneshot};

pub type EventSender = mpsc::Sender<Event>;
pub type ResponseSender = oneshot::Sender<Response>;

pub const APPLICATION_ERR_MIN: u8 = 0x80;
pub const APPLICATION_ERR_MAX: u8 = 0x9f;

#[derive(Debug)]
pub enum Event {
    ReadRequest(ReadRequest),
    WriteRequest(WriteRequest),
    NotifySubscribe(NotifySubscribe),
    NotifyUnsubscribe,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ReadRequest {
    pub offset: u16,
    pub response: ResponseSender,
    pub mtu: u16,
}

#[derive(Debug)]
pub struct WriteRequest {
    pub data: Vec<u8>,
    pub offset: u16,
    pub without_response: bool,
    pub response: ResponseSender,
}

#[derive(Debug, Clone)]
pub struct NotifySubscribe {
    pub notification: mpsc::Sender<Vec<u8>>,
}

/// Error type used when the caller attempts to construct an invalid application-level error via
/// `Response`.
#[derive(Debug, Clone)]
pub struct InvalidApplicationError {
    /// The invalid code that was used.
    pub invalid_code: u8,
}

impl fmt::Display for InvalidApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Application error must be in range \
            [{APPLICATION_ERR_MIN}, {APPLICATION_ERR_MAX}] - got {}",
            self.invalid_code
        )
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Response {
    Success(Vec<u8>),
    InvalidOffset,
    InvalidAttributeLength,
    UnlikelyError,
    ApplicationError(ApplicationError),
}

impl Response {
    /// Constructs a response containing an application level error code. If the code is invalid,
    /// an error will be returned.
    pub fn application_error(code: u8) -> Result<Self, InvalidApplicationError> {
        ApplicationError::new(code).map(Self::ApplicationError)
    }
}

/// Contains an application level error code. The bluetooth core specification version 5.3 defines
/// application errors to be in the range 0x80-0x9f.
///
/// See https://www.bluetooth.com/specifications/specs/core-specification-5-3/.
#[derive(Debug, Clone)]
pub struct ApplicationError {
    code: u8,
}

impl ApplicationError {
    /// Constructs a response containing an application level error code. If the code is invalid,
    /// an error will be returned.
    pub fn new(code: u8) -> Result<Self, InvalidApplicationError> {
        if !(APPLICATION_ERR_MIN..=APPLICATION_ERR_MAX).contains(&code) {
            return Err(InvalidApplicationError { invalid_code: code });
        }
        Ok(Self { code })
    }

    /// Returns the inner error code.
    pub fn code(&self) -> u8 {
        self.code
    }
}
