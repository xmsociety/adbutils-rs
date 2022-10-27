use core::panicking::panic;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::time;
use crate::client::{AdbClient, AdbConnection};
use crate::error::AdbError;


pub struct ShellMixin<'a> {
    pub client: &'a AdbClient,
    pub serial: String,
    pub transport_id: i32,
    pub properties: Option<HashMap<String, String>>
}

impl ShellMixin {

    pub fn new(client: &AdbClient, serial: String, transport_id: i32, properties: Option<HashMap<String, String>>) -> Self {
        Self{
            client,
            serial,
            transport_id,
            properties,
        }
    }

    fn run(&self, cmd: &str) -> Result<String, AdbError> {
    unimplemented!()
    }

    fn say_hello(&self) -> String {
        let content = "hello from " + &self.serial;
        let res = self.run("echo " + content);
        return res.unwrap()
    }

    fn open_transport(&self, command:&str, time_out: time::Duration) -> AdbConnection {
        let mut conn = self.client._connect();
        conn.set_timeout(time_out).unwrap();
        if command != "" {
            if self.transport_id > 0 {
                conn.send_command("host-transport-id:" + &format!("{}:{}", self.transport_id, command)).unwrap()
            } else if self.serial != "" {
                conn.send_command("host-serial:" + &format!("{}:{}", self.serial, command)).unwrap()
            } else {
                panic("RuntimeError")
            };
            conn.check_oky()
        }
        conn
    }
}

pub struct AdbDevice {
    pub shell_mixin: ShellMixin
}

impl AdbDevice {
    pub fn get_with_command(&self, cmd: &str) -> String {
        let mut conn = self.shell_mixin.open_transport("", self.shell_mixin.client.socket_time);
        conn.send_command(&format!("host-serial:{}:{}", self.shell_mixin.serial, cmd)).unwrap();
        conn.check_oky().unwrap();
        conn.read_string_block().unwrap()
    }
    pub fn get_state(&self) -> String {
        self.get_with_command("get-state")
    }

    pub fn get_serial_no(&self) -> String {
        self.get_with_command("get-serialno")
    }

    pub fn get_dev_path(&self) -> String {
        self.get_with_command("get-devpath")
    }

    pub fn get_feature(&self) -> String {
        self.get_with_command("features")
    }

    pub fn info(&self) -> HashMap<String, String> {
        let mut res: HashMap<String, String> = HashMap::new();
        res.insert("serialno".to_string(), self.get_serial_no().to_string());
        res.insert("devpath".to_string(), self.get_dev_path().to_string());
        res.insert("state".to_string(), self.get_state().to_string());
        res
    }

    pub fn adb_out(&self, cmd: &str) -> String {
        unimplemented!()
    }

    pub fn shell<T>(&self, cmd: &str, stream: bool, time_out: time::Duration) -> T {
        let mut conn = self.shell_mixin.open_transport("", time_out);
        conn.send_command("shell:" + cmd).unwrap();
        conn.check_oky().unwrap();
        if stream {
            conn
        }
        conn.read_until_close()
    }

    pub fn shell_out_put<T>(&self, cmd: &str) -> T {
        self.shell_mixin.client.shell(&self.shell_mixin.serial, cmd, false)
    }

}

impl Display for AdbDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdbDevice(serial={})", self.shell_mixin.serial)
    }
}

struct Sync {
    adb_client: AdbClient,
    serial:  String,
}