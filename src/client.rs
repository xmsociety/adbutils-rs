use std::borrow::{Borrow, BorrowMut};
use std::env;
use std::fmt::{Debug};
use std::net::{Shutdown, TcpStream};
use std::io;
use std::io::{Error, Read, Write};
use std::path::{Path, PathBuf};
use path_absolutize::*;

use std::process::Command;
use std::time;
use log::{error, log, log_enabled};

use crate::error::*;
use crate::device::{AdbDevice, ShellMixin};
use crate::proto::AdbConnectionOrString;

const OKAY: &str = "OKAY";
const FAIL: &str = "FAIL";
const DENT: &str = "DENT";
const DONE :&str = "DONE";

const WINDOWS: &str = "windows";
const MAC: &str = "macos";
const LINUX: &str = "linux";

fn check_server(host: &str, port: usize) -> bool {
    true
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

    pub fn _connect(&self) -> AdbConnection {
        let mut adb_connection = AdbConnection {
            host: self.host.clone(),
            port: self.port,
            conn: None,
        };
        let conn = adb_connection.safe_connect().expect("get _connect error");
        adb_connection.conn = Some(conn);
        adb_connection
    }

    // Done ✅
    pub fn server_version(&self) -> i32 {
        let mut conn = self._connect();
        conn.send_command("host:version").unwrap();
        conn.check_oky().unwrap();
        let res = conn.read_string_block().unwrap();
        res.parse::<i32>().unwrap() + 16
    }

    pub fn server_kill(&self) {
        unimplemented!()
    }

    pub fn wait_for(&self) {
    unimplemented!()
    }

    pub fn connect(&self, addr: &str) -> String {
        let mut conn = self._connect();
        conn.send_command(&format!("host:connect:{}", addr)).unwrap();
        conn.read_string_block().unwrap()
    }

    pub fn dis_connect(self, addr: &str, raise_err: bool) -> String {
        let mut conn = self._connect();
        conn.send_command(&format!("host:disconnect:{}", addr)).unwrap();
        conn.read_string_block().unwrap()
    }

    pub fn shell(&self, serial: &str, commad: &str, stream: bool) -> AdbConnectionOrString {
        let sn_tid = SerialNTransportID{
            serial: serial.to_string(),
            transport_id: 0,
        };
        return self.device(sn_tid).shell(commad, stream, self.socket_time);
    }

    pub fn devices_list(&self) -> Vec<AdbDevice>{
        let mut res: Vec<AdbDevice> =Vec::new();
        let mut c = self._connect();
        c.send_command("host:devices").unwrap();
        c.check_oky().unwrap();
        let out_put = c.read_string_block().unwrap();
        let out_puts: Vec<&str> = out_put.split("\n").collect();
        for line in out_puts.into_iter() {
            let parts: Vec<&str> = line.split("\t").collect();
            if parts.len() == 2 && parts[1] == "device"{
                res.push(AdbDevice{shell_mixin: ShellMixin::new(AdbClient{
                    host: self.host.clone(),
                    port: self.port.clone(),
                    socket_time: self.socket_time.clone()
                }, parts[0].to_string(), 0, None)})
            }
        }
        res
    }

    pub fn device(&self, sn_tid: SerialNTransportID) -> AdbDevice {
        if sn_tid.serial != "" || sn_tid.transport_id != 0 {
            return AdbDevice { shell_mixin: ShellMixin::new(AdbClient{
                host: self.host.clone(),
                port: self.port.clone(),
                socket_time: self.socket_time.clone()
            }, sn_tid.serial, sn_tid.transport_id, None) }
        }
        let serial = env::var("ANDROID_SERIAL").unwrap();
        if serial != "" {
            let ds = self.devices_list();
            if ds.len() == 0 {
                log::info!("Error: Can't find any android device/emulator")
            } else if ds.len() > 1 {
                log::info!("more than one device/emulator, please specify the serial number")
            } else {
                return AdbDevice{shell_mixin: ShellMixin::new(AdbClient{
                    host: self.host.clone(),
                    port: self.port.clone(),
                    socket_time: self.socket_time.clone()
                }, ds[0].get_serial_no(), 0, None)}
            }
        }
        return AdbDevice{shell_mixin: ShellMixin::new(AdbClient{
            host: self.host.clone(),
            port: self.port.clone(),
            socket_time: self.socket_time.clone()
        }, sn_tid.serial, sn_tid.transport_id, None)}
    }
}

struct SerialNTransportID {
    serial: String,
    transport_id: i32,
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
                return match error.kind() {
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
                                Err(AdbError::ResponseStatusError {
                                    content: String::from(error.clone()),
                                })
                            }
                            Err(error) => {
                                Err(AdbError::StartAdbFailed {
                                    source: Box::new(error),
                                })
                            }
                        }
                    }
                    _ => {
                        Err(AdbError::UnknownError { source: Box::new(error) })
                    }
                }
        };
        Ok(conn)
    }

    pub fn set_timeout(&self, time_out: time::Duration) -> Result<(), AdbError> {
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

    fn read(&mut self, n: usize) -> Result<String, AdbError>  {
        self.read_full(n)
    }

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
                    Ok(content_string) => {
                        println!("read: => {}", content_string);
                        return Ok(content_string)
                    }
                    Err(error) => {
                        return Err(AdbError::ParseResponseError {
                            source: Box::new(error),
                        });
                    }
                }
            },
            None => {
                println!("{:?}", "NoneNoneNoneNoneNone");
                Ok("".to_string())
            }
        }

    }

    fn add_command_length_prefix(&self, command_body: String) -> String {
        let trim_command = command_body.trim();
        let trim_command_length = format!("{:04X}", trim_command.len());
        trim_command_length + trim_command
    }

    pub fn send_command(&mut self, cmd: &str) -> Result<(), AdbError> {
        let msg = self.add_command_length_prefix(cmd.to_string());
        match &mut self.conn {
            Some(conn) => {
                match conn.write_all(msg.as_ref()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(AdbError::TcpWriteError {
                        source: Box::new(error),
                    }),
                }
            },
            None => {Ok(()) }
        }
    }

    fn read_string(&mut self, n: usize) -> String {
        let res = match self.read(n) {
            Ok(res) => res,
            Err(error ) => {
                println!("{:?}", error);
                String::new()
            }
        };
        res
    }

    pub fn read_string_block(&mut self) -> Result<String, AdbError> {
        let res = self.read_string(4);
        if res.len() == 0 {
            return Err(AdbError::ResponseStatusError {content: String::from("receive data error connection closed")})
        }
        let size = res.parse::<usize>().unwrap();
        let res = self.read_string(size);
        Ok(res)
    }

    pub fn read_until_close(&mut self) -> Result<String, AdbError> {
        let mut res = String::new();
        loop {
            let mut origin_buffer = self.read(4096 );
            let buffer = match origin_buffer {
                Ok(r) => { res += &*r; r}
                Err(error) => {String::new()}
            };
            if buffer.len() == 0 {
                break
            }
        }
        Ok(res.to_string())
    }

    pub fn check_oky(&mut self) -> Result<(), AdbError>{
        let data = self.read_string(4);
        if data == FAIL {
            log::debug!("receive data: {} connection closed", data)
        } else if data == OKAY {
            return Ok(())
        }
        Ok(())
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
        // let client = adb._connect();
        println!("adb version: {:?}", adb.server_version())
    }
    #[test]
    fn test_devices() {
        let adb = AdbClient::new(String::from("localhost"), 5037, time::Duration::new(10, 0));
        println!("{:?}", adb.devices_list());
        println!("{:?}", adb.devices_list()[0].shell("ls -a ", false, time::Duration::new(0, 0)))
    }

}