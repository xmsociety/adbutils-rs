
use std::net::{Shutdown, TcpStream};
use std::io;
use std::env;
use std::ffi::CString;
use std::io::{Error, Read};
use std::ops::Neg;
use std::path::{Path, PathBuf};
use path_absolutize::*;

use std::process::Command;
use std::slice::RSplit;
use std::time;
use log::{error, log};

use crate::error::*;


const WINDOWS: &str = "windows";
const MAC: &str = "macos";
const LINUX: &str = "linux";

fn check_server(host: &str, port: usize) -> bool {
    true
}

#[derive(Debug)]
pub struct AdbClient {
    pub host: String,
    pub port: u32,
    pub socket_time: time::Duration,
}

impl AdbClient {

    pub fn new(host: String, port: u32, socket_time: time::Duration) -> AdbClient {
        AdbClient {
            host,
            port,
            socket_time,
        }
    }

    fn _connect(&self) -> AdbConnection {
        let mut adb_connection = AdbConnection {
            host: self.host.clone(),
            port: self.port,
            conn: None,
        };
        let conn = adb_connection.safe_connect().expect("get _connect error");
        adb_connection.conn = Some(conn);
        adb_connection
    }

    pub fn server_version(&self) -> i32 {
        let conn = self._connect();
        // conn.
        1
    }
    pub fn server_kill(&self) {
        unimplemented!()
    }

    pub fn wait_for(&self) {
    unimplemented!()
    }

    pub fn connect(&self, addr: &str) -> String {
        unimplemented!()
    }

    pub fn dis_connect(self, addr: &str, raise_err: bool) -> String {
        unimplemented!()
    }

    pub fn shell(&self, commadn: &str, stream: bool) {
        unimplemented!()
    }

    pub fn devices_list(&self) {
        unimplemented!()
    }

    pub fn device(&self) {
        unimplemented!()
    }
}


fn adb_path() -> String {
    let cwd = env::current_dir().unwrap();
    // let cwd_parent = cwd.parent().unwrap();
    let os = env::consts::OS;
    let mut adb_path = Path::join(&cwd, Path::new("binaries/mac/adb"));
    if os == LINUX {
        adb_path = Path::join(&cwd, Path::new("binaries/linux/adb"))
    } else if os == WINDOWS {
        adb_path = Path::join(&cwd, Path::new("binaries/win/adb.exe"))
    }
    adb_path.to_str().unwrap().to_string()
}

#[derive(Debug)]
pub struct AdbConnection {
    host: String,
    port: u32,
    conn: Option<TcpStream>,
}

impl AdbConnection {

    fn safe_connect(&self) -> Result<TcpStream, AdbError> {
        let conn = match self.create_socket() {
            Ok(conn) => conn,
            Err(error) =>
                match error.kind() {
                    io::ErrorKind::ConnectionRefused => {
                        match Command::new(&adb_path())
                            .arg("start-server")
                            .output()
                        {
                            Ok(response) => {
                                if response.status.success() {
                                    let content = String::from_utf8_lossy(&response.stdout);
                                    log::info!("start-server done! {}",  content);
                                    let conn = match self.create_socket() {
                                        Ok(conn) => conn,
                                        _ => { return Err(AdbError::UnknownError { source: Box::new(error) }); }
                                    };
                                    return Ok(conn);
                                }
                                let error = String::from_utf8_lossy(&response.stderr);
                                return Err(AdbError::ResponseStatusError {
                                    content: String::from(error.clone()),
                                });
                            }
                            Err(error) => {
                                return Err(AdbError::StartAdbFailed {
                                    source: Box::new(error),
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(AdbError::UnknownError { source: Box::new(error) });
                    }
                }
        };
        Ok(conn)
    }

    fn set_timeout(&self, time_out: time::Duration) -> Result<(), AdbError> {
        if !time_out.is_zero() {
            match &self.conn {
                Some(conn) => {
                    match conn.set_read_timeout(Some(time_out)) {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(AdbError::TcpReadError {
                                source: Box::new(error),
                            });
                        }
                    };
                    match conn.set_write_timeout(Some(time_out)) {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(AdbError::TcpReadError {
                                source: Box::new(error),
                            });
                        }
                    };
                }
                None => {
                    panic!("time_out with nil conn!");
                }
            }

        }
        return Ok(())
    }

    fn create_socket(&self) -> Result<TcpStream, io::Error> {
        let host_port = format!("{}:{}", self.host, self.port);
        let conn = TcpStream::connect(host_port.clone());
        conn
    }

    fn close(&self) {
        match &self.conn {
            Some(conn) => { conn.shutdown(Shutdown::Both).expect("close adb server Error!");},
            None => {}
        }
    }

    fn read(&self, n: usize) {}

    fn read_full(&mut self, n: usize) -> Result<String, AdbError> {
        let mut buff = vec![0; n];
        match &mut self.conn {
            Some(conn) => {
                match conn.read_exact(&mut buff) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(AdbError::TcpReadError {
                            source: Box::new(error),
                        });
                    }
                };

                match String::from_utf8(Vec::from(buff)) {
                    Ok(content_string) => Ok(content_string),
                    Err(error) => {
                        return Err(AdbError::ParseResponseError {
                            source: Box::new(error),
                        });
                    }
                }
            },
            None => {}
        }

    }

    fn send_command(&self) -> Result<String, AdbError> {
        unimplemented!()
    }

    fn read_string(&self) -> Result<String, AdbError> {
        unimplemented!()
    }

    fn read_string_block(&self) -> Result<String, AdbError> {
        unimplemented!()
    }

    fn read_until_close(&self) -> Result<String, AdbError> {
        unimplemented!()
    }

    fn check_oky(&mut self) -> Result<String, AdbError>{
        let mut is_ok_buffer = [0; 4];
        match &mut self.conn {
            Some(conn) => {
                match conn.read_exact(&mut is_ok_buffer) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(AdbError::TcpReadError {
                            source: Box::new(error),
                        });
                    }
                }
                match String::from_utf8(Vec::from(is_ok_buffer)) {
                    Ok(response_status) => Ok(response_status),
                    Err(error) => Err(AdbError::ParseResponseError {
                        source: Box::new(error),
                    }),
                }
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::time;
    use super::AdbClient;

    #[test]
    fn test_path() {
        println!("{:?}", super::adb_path())
    }

    #[test]
    fn test_connect() {
        let adb = AdbClient::new(String::from("localhost"), 5037, time::Duration::new(10, 0));
        let client = adb._connect();
        println!("{:?}", client)
    }
}