//! Wire format for WebSocket messages (also used by fuzz targets).
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::BoardElement;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientWsMessage {
    AddElement {
        element_type: String,
        payload: serde_json::Value,
        #[serde(default)]
        z_index: i32,
    },
    RemoveElement {
        id: Uuid,
    },
    Clear,
}

pub fn parse_client_ws_message(input: &str) -> Result<ClientWsMessage, serde_json::Error> {
    serde_json::from_str(input)
}

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ServerWsMessage {
    Snapshot {
        elements: Vec<BoardElement>,
    },
    ElementAdded {
        element: BoardElement,
    },
    ElementRemoved {
        id: Uuid,
    },
    Cleared,
    Error {
        message: String,
    },
}
