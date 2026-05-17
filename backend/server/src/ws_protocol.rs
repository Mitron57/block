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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn parses_add_element() {
        let msg = parse_client_ws_message(
            r##"{"op":"add_element","element_type":"stroke","payload":{"color":"#000"},"z_index":2}"##,
        )
        .unwrap();
        match msg {
            ClientWsMessage::AddElement {
                element_type,
                z_index,
                ..
            } => {
                assert_eq!(element_type, "stroke");
                assert_eq!(z_index, 2);
            }
            _ => panic!("expected AddElement"),
        }
    }

    #[test]
    fn add_element_default_z_index() {
        let msg = parse_client_ws_message(
            r#"{"op":"add_element","element_type":"line","payload":{}}"#,
        )
        .unwrap();
        match msg {
            ClientWsMessage::AddElement { z_index, .. } => assert_eq!(z_index, 0),
            _ => panic!("expected AddElement"),
        }
    }

    #[test]
    fn parses_remove_element_and_clear() {
        let id = Uuid::new_v4();
        let remove = parse_client_ws_message(
            &format!(r#"{{"op":"remove_element","id":"{id}"}}"#),
        )
        .unwrap();
        assert!(matches!(remove, ClientWsMessage::RemoveElement { .. }));

        let clear = parse_client_ws_message(r#"{"op":"clear"}"#).unwrap();
        assert!(matches!(clear, ClientWsMessage::Clear));
    }

    #[test]
    fn rejects_unknown_op() {
        assert!(parse_client_ws_message(r#"{"op":"noop"}"#).is_err());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_client_ws_message("not-json").is_err());
        assert!(parse_client_ws_message("").is_err());
    }
}
