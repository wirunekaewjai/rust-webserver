#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    ReadProtocol,
    ReadHeaders,
    ValidateHeaders,
    ReadBody,
    HandleRequest,
}
