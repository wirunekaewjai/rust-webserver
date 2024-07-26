use crate::server::{enums::ConnectionState, structs::{Connection, Response}};

pub fn read_body(connection: &mut Connection) -> Result<(), Response> {
    let content_buffer_length = connection.buffer.len();

    if content_buffer_length > connection.http_content_length {
        return Err(Response::content_too_large());
    }
    
    else if content_buffer_length == connection.http_content_length {
        connection.http_state = ConnectionState::HandleRequest;
    }

    Ok(())
}
