#[derive(Debug, PartialEq, Eq)]
pub enum RequestMethod {
    None,

    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
}

impl RequestMethod {
    pub fn from(value: &str) -> RequestMethod {
        if value.ends_with("GET") {
            return RequestMethod::Get;
        }
        
        else if value.ends_with("POST") {
            return RequestMethod::Post;
        }
        
        else if value.ends_with("PUT") {
            return RequestMethod::Put;
        }
        
        else if value.ends_with("PATCH") {
            return RequestMethod::Patch;
        }
        
        else if value.ends_with("DELETE") {
            return RequestMethod::Delete;
        }
        
        else if value.ends_with("HEAD") {
            return RequestMethod::Head;
        }
        
        else if value.ends_with("OPTIONS") {
            return RequestMethod::Options;
        }
        
        RequestMethod::None
    }
}