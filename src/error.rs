use std::error::Error;

#[derive(Debug)]
pub enum AdbError {
    StartAdbFailed {
        source: Box<dyn Error>,
    },
    TcpConnectError {
        source: Box<dyn Error>,
    },
    TcpWriteError {
        source: Box<dyn Error>,
    },
    TcpReadError {
        source: Box<dyn Error>,
    },
    ParseResponseError {
        source: Box<dyn Error>,
    },
    ResponseStatusError {
        content: String,
    },
    UnknownError {
        source: Box<dyn Error>,
    }
}