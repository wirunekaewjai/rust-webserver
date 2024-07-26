use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolVersion {
    None,

    Http1_0,
    Http1_1,
    Http2_0,
}

impl ProtocolVersion {
    pub fn from(value: &str) -> ProtocolVersion {
        if value == "HTTP/1.0" {
            return ProtocolVersion::Http1_0;
        }
        
        else if value == "HTTP/1.1" {
            return ProtocolVersion::Http1_1;
        }

        else if value == "HTTP/2.0" {
            return ProtocolVersion::Http2_0;
        }

        ProtocolVersion::None
    }
}

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &ProtocolVersion::Http1_0 {
            return write!(f, "HTTP/1.0");
        }

        else if self == &ProtocolVersion::Http2_0 {
            return write!(f, "HTTP/2.0");
        }
        
        return write!(f, "HTTP/1.1");
    }
}