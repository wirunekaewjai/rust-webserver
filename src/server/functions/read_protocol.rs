use crate::server::{enums::{ConnectionState, ProtocolVersion, RequestMethod}, structs::{Connection, Response}};

use super::read_string_buffer;

pub fn read_protocol(connection: &mut Connection) -> Result<(), Response> {
    let mut i = 0;

    while i < connection.buffer.len() {
        if i > 0 && connection.buffer[i] == b'\n' && connection.buffer[i - 1] == b'\r' {
            let text_buffer = connection.buffer.drain(..=i).collect::<Vec<u8>>();
            let text = read_string_buffer(&text_buffer);

            let parts = text
                .split_whitespace()
                .map(|txt| txt.to_string())
                .collect::<Vec<String>>();

            if parts.len() < 3 {
                return Err(Response::bad_request());
            }

            connection.http_method = RequestMethod::from(&parts[0]);
            connection.http_version = ProtocolVersion::from(&parts[2]);
            connection.http_request_uri = parts[1].clone();

            if connection.http_version == ProtocolVersion::Http1_1 {
                connection.http_state = ConnectionState::ReadHeaders;
                break;
            } else {
                return Err(Response::method_not_allowed());
            }
        }

        i += 1;
    }

    Ok(())
}
