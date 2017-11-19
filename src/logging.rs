#[derive(Copy, Clone, Debug)]
pub enum LogMsgType {
    Message,
    Debug,
}
pub struct Logger {
    messages: Vec<(LogMsgType, String)>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            messages: Vec::new(),
        }
    }
    pub fn log<S: Into<String>>(&mut self, msg_type: LogMsgType, msg: S) {
        self.messages.push((msg_type, msg.into()));
    }
    pub fn drain_messages(&mut self) -> Vec<(LogMsgType, String)> {
        let mut alt_buffer = Vec::new();
        ::std::mem::swap(&mut self.messages, &mut alt_buffer);
        alt_buffer
    }
}
