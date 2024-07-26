use std::collections::HashMap;

pub struct Response {
    pub status: u16,
    pub status_text: String,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, buffer: &[u8]) -> Self {
        self.body.extend_from_slice(&buffer);
        self
    }

    pub fn new(status: u16, status_text: &str) -> Response {
        Response {
            status,
            status_text: status_text.to_string(),

            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn switch_protocols() -> Response {
        Response::new(101, "Switching Protocols")
    }

    pub fn ok() -> Response {
        Response::new(200, "OK")
    }

    pub fn created() -> Response {
        Response::new(201, "Created")
    }

    pub fn no_content() -> Response {
        Response::new(204, "No Content")
    }

    pub fn bad_request() -> Response {
        Response::new(400, "Bad Request")
    }

    pub fn not_found() -> Response {
        Response::new(404, "Not Found")
    }

    pub fn method_not_allowed() -> Response {
        Response::new(405, "Method Not Allowed")
    }

    pub fn not_acceptable() -> Response {
        Response::new(406, "Not Acceptable")
    }

    pub fn content_too_large() -> Response {
        Response::new(413, "Content Too Large")
    }

    pub fn request_header_fields_too_large() -> Response {
        Response::new(431, "Request Header Fields Too Large")
    }
}
