use crate::server::{enums::{ConnectionState, RequestMethod}, structs::{Connection, Response}};

use super::read_string_buffer;

const HARD_LIMIT_CONTENT_LENGTH: usize = 100 * 1024 * 1024; // 100 MB
const HARD_LIMIT_HEADERS_LENGTH: usize = 32 * 1024; // 32 KB
const HARD_LIMIT_HEADER_LENGTH: usize = 16 * 1024; // 16 KB

pub fn read_headers(connection: &mut Connection) -> Result<(), Response> {
    let mut i = 0;

    while i < connection.buffer.len() {
        if i > 0 && connection.buffer[i] == b'\n' && connection.buffer[i - 1] == b'\r' {
            let text_buffer = connection.buffer.drain(..=i).collect::<Vec<u8>>();

            // blank line (\r\n)
            if text_buffer.len() == 2 {
                if let Some(content_length) = connection.http_headers.get("content-length") {
                    if let Ok(content_length) = content_length.parse::<usize>() {
                        if content_length > 0 {
                            if connection.http_method == RequestMethod::Get ||
                               connection.http_method == RequestMethod::Head {
                                return Err(Response::not_acceptable());
                            } else if content_length > HARD_LIMIT_CONTENT_LENGTH {
                                return Err(Response::content_too_large());
                            }

                            connection.http_content_length = content_length;
                            connection.http_state = ConnectionState::ValidateHeaders;
                            
                            break;
                        }
                    }
                }

                connection.websocket = connection.http_headers.get("upgrade").is_some_and(|v| v == "websocket");
                connection.http_state = ConnectionState::ValidateHeaders;
                break;
            }

            let text = read_string_buffer(&text_buffer);
            let header = text.splitn(2, ": ").collect::<Vec<&str>>();

            if header.len() == 2 {
                let key = header[0].to_lowercase();
                let value = header[1].trim();
                let header_length = key.len() + value.len();

                connection.http_headers_length += header_length;

                if header_length > HARD_LIMIT_HEADER_LENGTH ||
                    connection.http_headers_length > HARD_LIMIT_HEADERS_LENGTH {
                    return Err(Response::request_header_fields_too_large());
                }

                connection.http_headers.insert(key.to_string(), value.to_string());
            }

            i = 0;

            continue;
        }

        i += 1;
    }

    Ok(())
}
