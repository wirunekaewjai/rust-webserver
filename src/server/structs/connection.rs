use std::{collections::HashMap, net::TcpStream};

use crate::server::enums::{ConnectionState, ProtocolVersion, RequestMethod};

#[derive(Debug)]
pub struct Connection {
    pub id: u128,
    
    pub stream: TcpStream,
    pub buffer: Vec<u8>,
    
    pub websocket: bool,

    pub http_content_length: usize,
    pub http_headers_length: usize,
    pub http_headers: HashMap<String, String>,
    pub http_method: RequestMethod,
    pub http_request_uri: String,
    pub http_state: ConnectionState,
    pub http_version: ProtocolVersion,
}