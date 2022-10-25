
use std::net::TcpStream;
mod proto;
mod device;
mod client;
mod error;

// thx https://github.com/WangZemin0816/rust-adb/blob/d415ab988dce9090da987e066695803388b58ea4/src/adb_host/mod.rs
pub trait SyncHostCommand {
    fn execute(&mut self) -> Result<SyncHostResponse, error::AdbError>;
}

pub trait AsyncHostCommand {
    fn execute(&mut self) -> Result<AsyncHostResponse, error::AdbError>;
}

#[derive(Debug)]
pub struct SyncHostResponse {
    pub length: usize,
    pub content: String,
}

pub struct AsyncHostResponse {
    pub tcp_stream: TcpStream,
}
// thx end

#[cfg(test)]
mod tests {
    use crate::client::AdbClient;
    use super::*;
}
