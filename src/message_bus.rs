/// These functions uses the selected message bus to
/// communicate with the environment we are enbedded into.

enum MessageType {
    Broadcast,
    Targeted(String),
}

enum BusError {}

pub struct BusMessage {
    msgtype: MessageType,
    source: String,
    data: String,
}

pub struct NotificationMessage {
    source: String,
    data: String,
}

pub mod message_bus {
    pub fn subscribe(
        identifier: &str,
        upstream_channel: Sender<NotificationMessage>,
    ) -> Result<(), BusError> {
    }

    pub fn send(identififer: &str, message: BusMessage) -> Result<(), BusError> {}

    /// Ask the bus for broadcast messages and messages specifically for me, and return them
    pub fn poll(identifier: &str) -> Result<Vec<BusMessage>, BusError> {}
}
