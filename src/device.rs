use std::collections::HashMap;
use std::fmt::{Display, format, Formatter, Pointer, write};
use std::time;
use crate::client::{AdbClient, AdbConnection};
use crate::error::AdbError;
use crate::proto::AdbConnectionOrString;


#[derive(Debug)]
pub struct ShellMixin {
    pub client: AdbClient,
    pub serial: String,
    pub transport_id: i32,
    pub properties: Option<HashMap<String, String>>
}

impl ShellMixin {

    pub fn new(client: AdbClient, serial: String, transport_id: i32, properties: Option<HashMap<String, String>>) -> ShellMixin {
        Self{
            client,
            serial,
            transport_id,
            properties,
        }
    }

    pub fn run(&self, cmd: String) -> AdbConnectionOrString {
        self.client.shell(self.serial.as_str(), cmd.as_str(), false)
    }

    pub fn say_hello(&self) -> String {
        let content = "hello from ".to_string() + &self.serial;
        let res = self.run("echo ".to_string() + content.as_str());
        match res {
            AdbConnectionOrString::String(str) => str,
            _ => String::from("err say_hello")
        }
    }

    pub fn switch_screen(&self, status: bool) {
        let key_map: HashMap<bool, &str> = HashMap::from([(true, "224"), (false, "223")]);
        let cmd = key_map.get(&status).unwrap();

    }

    pub fn switch_air_plane(&self, status: bool) {
        let mut base = "settings put global airplane_mode_on".to_string();
        let mut am = "am broadcast -a android.intent.action.AIRPLANE_MODE --ez state".to_string();
        if status {
            base += "1";
            am += "true"
        } else {
            base += "0";
            am += "false"
        }
        self.run(base.to_string());
        self.run(am.to_string());
    }

    pub fn switch_wifi(&self, status: bool) {
        let cmd_map: HashMap<bool, String> = HashMap::from([(true, "svc wifi enable".to_string()), (false, "svc wifi disable".to_string())]);
        let cmd = cmd_map.get(&status).unwrap();
        self.run(String::from(cmd));
    }
    pub fn key_event(&self, key_code: &str) -> String {
        let res = self.run("input keyevent ".to_string() + key_code);
        match res {
            AdbConnectionOrString::String(str) => str,
            _ => String::from("err key_event")
        }
    }

    pub fn click(&self, x: i32, y: i32) {
        self.run(format!("input tap {} {}", x, y));
    }

    pub fn swipe(&self, x: i32, y: i32, tox: i32, toy: i32, duration: time::Duration) {
        self.run(format!("input swipe {} {} {} {} {}", x, y, tox, toy, duration.as_secs() * 1000));
    }

    pub fn send_keys(&self, text: &str) {
        self.run(format!("input text {}", text));
    }

    pub fn escape_special_characters(&self, text: &str) {}

    pub fn wlan_ip(&self) -> String {
        let res = self.run("ifconfig wlan0".to_string());
        match res {
            AdbConnectionOrString::String(str) => str,
            _ => String::from("err wlan_ip")
        }
    }

    pub fn install(&self) {
        unimplemented!()
    }

    pub fn install_remote(&self) {
        unimplemented!()
    }

    pub fn uninstall(&self, package_name: &str) {
        self.run(format!("pm uninstall {}",  package_name));
    }

    pub fn get_prop(&self, prop: &str) -> String {
        let res = self.run("getprop ".to_string() + prop);
        match res {
            AdbConnectionOrString::String(str) => str,
            _ => String::from("err get_prop")
        }
    }

    pub fn list_packages(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        let out_put = self.run("pm list packages".to_string());
        match out_put {
            AdbConnectionOrString::String(packages_str) => {
                let packages: Vec<&str> = packages_str.split("\n").collect();
                for package in packages {
                    res.push(package.to_string())
                }
                return res
            }
            _ => res
        }
    }

    pub fn package_info(&self) {
        unimplemented!()
    }

    pub fn rotation(&self) {
        unimplemented!()
    }

    pub fn raw_window_sise(&self) {
        unimplemented!()
    }

    pub fn window_size(&self) {
        unimplemented!()
    }

    pub fn app_start(&self, package_name: &str, activity: &str){
        if activity != "" {
            self.run(format!("am start -n {} / {}", package_name, activity));
        }else {
            self.run(format!("monkey -p {} -c android.intent.category.LAUNCHER 1", package_name));
        }
    }

    pub fn app_stop(&self, package_name: &str) {
        self.run("am force-stop ".to_string() + package_name);
    }

    pub fn app_clear(&self, package_name: &str) {
        self.run("pm clear ".to_string() + package_name);
    }

    pub fn is_screen_on(&self) -> bool {
        let res = self.run("dumpsys power".to_string());
        return match res {
            AdbConnectionOrString::String(str) => {
                str.contains("mHoldingDisplaySuspendBlocker=true")
            }
            _ => {
                false
            }
        }
    }

    pub fn open_browser(&self, url: &str) {
        self.run("am start -a android.intent.action.VIEW -d ".to_string() + url);
    }

    pub fn dump_hierarchy(&self) {
        unimplemented!()
    }

    pub fn curren_app(&self) {
        unimplemented!()
    }

    pub fn remove(&self, path: &str) {
        self.run("rm ".to_string() + path);
    }

    fn open_transport(&self, command:&str, time_out: time::Duration) -> AdbConnection {
        let mut conn = self.client._connect();
        conn.set_timeout(time_out).unwrap();
        if command != "" {
            if self.transport_id > 0 {
                conn.send_command(&format!("host-transport-id:{}:{}", self.transport_id, command)).unwrap()
            } else if self.serial != "" {
                conn.send_command(&format!("host-serial:{}:{}", self.serial, command)).unwrap()
            } else {
                panic!("RuntimeError")
            };
            conn.check_oky();
        }
        conn
    }
}

#[derive(Debug)]
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

    pub fn shell(&self, cmd: &str, stream: bool, time_out: time::Duration) -> AdbConnectionOrString {
        let mut conn = self.shell_mixin.open_transport("", time_out);
        conn.send_command(&format!("shell:{}", cmd)).unwrap();
        conn.check_oky().unwrap();
        if stream {
            return AdbConnectionOrString::AdbConnection(conn)
        }
        AdbConnectionOrString::String(conn.read_until_close().unwrap())
    }

    pub fn shell_out_put(&self, cmd: &str) -> String {
        let out_put= self.shell_mixin.client.shell(&self.shell_mixin.serial, cmd, false);
        match out_put {
            AdbConnectionOrString::String(str) => str,
            _ => String::from("get shell_out_put error!")
        }
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