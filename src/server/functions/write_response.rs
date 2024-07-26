use std::io::Write;

use crate::server::{enums::{ConnectionState, ProtocolVersion, RequestMethod}, structs::{Connection, Response}};

pub fn write_response(connection: &mut Connection, response: &Response) {
    let status_line = format!(
        "{} {} {}\r\n",
        connection.http_version,
        response.status,
        response.status_text,
    );

    let _ = connection.stream.write_all(status_line.as_bytes());

    if response.headers.len() > 0 {
        for (key, value) in &response.headers {
            let header = format!("{}: {}\r\n", key, value);
            let _ = connection.stream.write_all(header.as_bytes());
        }
    }

    if !response.headers.contains_key("content-length") {
        let body_length = response.body.len();
        let header = format!("content-length: {body_length}\r\n");
        let _ = connection.stream.write_all(header.as_bytes());
    }

    let _ = connection.stream.write_all(b"\r\n");

    if response.body.len() > 0 {
        let _ = connection.stream.write_all(&response.body);
    }

    let _ = connection.stream.flush();

    connection.buffer.clear();
    connection.http_content_length = 0;
    connection.http_headers_length = 0;
    connection.http_headers.clear();
    connection.http_method = RequestMethod::None;
    connection.http_request_uri.clear();
    connection.http_state = ConnectionState::ReadProtocol;
    connection.http_version = ProtocolVersion::None;
}