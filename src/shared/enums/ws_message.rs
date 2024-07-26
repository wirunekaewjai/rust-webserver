use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "action", content = "payload")]
pub enum WsMessage {
    SUBSCRIBE {
        room: String,
    },

    SUBSCRIBE_REJECTED {
        room: String,
    },

    UNSUBSCRIBE {
        room: String,
    },

    NOTIFY {
        room: String,
        text: String,
    },

    SEND {
        room: String,
        text: String,
    },
}