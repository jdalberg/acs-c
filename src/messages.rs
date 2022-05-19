use std::fmt;

#[derive(Clone, Debug)]
pub enum MessageType {
    Inform,
    InformResponse,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
