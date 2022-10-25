use std::collections::HashMap;


struct ShellMixin {
    client: Option<crate::client::AdbClient>,
    serial: String,
    transport_id: i32,
    properties: HashMap<String, String>
}

struct AdbDevice;
