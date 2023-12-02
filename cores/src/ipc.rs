

use serde::{Serialize, Deserialize};

pub const IPC_EXTENSION_ENDPOINT: &str = "0.0.0.0:4000";
pub const IPC_ACCEPT_ACK_TOKEN: &[u8; 3] = b"ACK";

#[derive(Serialize, Deserialize)]
pub struct InvokeMessage {
    pub event_type: String,
    pub body: String,
}
