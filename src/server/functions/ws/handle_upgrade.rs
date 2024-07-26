use base64::prelude::*;
use sha1::{Digest, Sha1};

use crate::server::structs::{Connection, Response};

const WEBSOCKET_VERSION: &str = "13";
const RFC6455_CONSTANT: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn handle_upgrade(connection: &Connection) -> Response {
    let websocket_version = connection.http_headers.get("sec-websocket-version");

    if websocket_version.is_none() || websocket_version.is_some_and(|v| v.ne(WEBSOCKET_VERSION)) {
        return Response::not_acceptable();
    }

    let Some(sec_websocket_key) = connection.http_headers.get("sec-websocket-key") else {
        return Response::not_acceptable();
    };

    let accept_key = generate_sec_websocket_accept(&sec_websocket_key);
 
    if let Ok(addr) = connection.stream.peer_addr() {
        println!("> {}: [WS] connected", addr);
    }

    return Response::switch_protocols()
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", &accept_key);
}

fn generate_sec_websocket_accept(sec_websocket_key: &str) -> String {
    let mut hasher = Sha1::new();
                                            
    hasher.update(format!("{}{}", sec_websocket_key, RFC6455_CONSTANT));

    let hash = hasher.finalize();
    let accept_key = BASE64_STANDARD.encode(hash);

    accept_key
}