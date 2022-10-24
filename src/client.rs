
use std::net::TcpStream;
use std::{error, io};
use std::os::unix::raw::uid_t;

use crate::error::*;

fn check_server(host: &str, port: usize) -> bool {
    true
}

#[derive(Debug)]
pub struct AdbClient{
    pub host:        String,
    pub port:        u32,
    pub socket_time: u32,
}

impl AdbClient{
    pub fn new(host: String, port: u32, socket_time: u32) -> AdbClient {
        AdbClient{
            host,
            port,
            socket_time
        }
    }
    pub fn connect(&self) -> AdbConnection {
        let mut adb_connection = AdbConnection::new(self.host.clone(), self.port);
        let conn = adb_connection.safe_connect().expect("get connect error");
        adb_connection.conn = conn;
        adb_connection
    }
}

#[derive(Debug)]
pub struct AdbConnection {
    host: String,
    port: u32,
    conn: TcpStream,
}

impl AdbConnection {

    fn new(host: String, port: u32) {}

    fn safe_connect() -> Result<TcpStream, AdbError> {
        let conn = match Self.create_socket() {
            // TODO start adb server
            Ok(conn) => conn,
            Err(error) => {
                return Err(AdbError::TcpConnectError {
                    source: Box::new(error),
                });
            }
        };
        // match conn.set_read_timeout(self.) {
        //     Ok(_) => {}
        //     Err(error) => {
        //         return Err(AdbError::TcpReadError {
        //             source: Box::new(error),
        //         });
        //     }
        // };
        // match conn.set_write_timeout(connection_info.write_timeout) {
        //     Ok(_) => {}
        //     Err(error) => {
        //         return Err(AdbError::TcpReadError {
        //             source: Box::new(error),
        //         });
        //     }
        // };
        Ok(conn)
    }

    fn create_socket() -> Result<TcpStream, io::Error> {
        let host_port = format!("{}:{}", Self.host, Self.port);
        let conn = TcpStream::connect(host_port.clone());
        conn
    }
}