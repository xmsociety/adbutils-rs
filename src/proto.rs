use crate::client::AdbConnection;

#[derive(Debug)]
pub enum AdbConnectionOrString {
    AdbConnection(AdbConnection),
    String(String)
}

pub struct DeviceEvent {
    pub present: bool,
    pub serial: String,
    pub status: String
}

pub struct  ForWardItem {
    pub serial: String,
    pub local: String,
    pub remote: String,
}

pub struct ReverseItem {
    pub remote: String,
    pub local: String,
}

