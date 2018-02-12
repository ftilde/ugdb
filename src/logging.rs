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
    fn log<S: Into<String>>(&mut self, msg_type: LogMsgType, msg: S) {
        self.messages.push((msg_type, msg.into()));
    }
    pub fn log_debug<S: Into<String>>(&mut self, msg: S) {
        self.log(LogMsgType::Debug, msg);
    }
    pub fn log_message<S: Into<String>>(&mut self, msg: S) {
        self.log(LogMsgType::Message, msg);
    }
    pub fn drain_messages(&mut self) -> Vec<(LogMsgType, String)> {
        let mut alt_buffer = Vec::new();
        ::std::mem::swap(&mut self.messages, &mut alt_buffer);
        alt_buffer
    }
}
