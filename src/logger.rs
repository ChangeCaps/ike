use std::sync::mpsc::SyncSender;

use log::{Level, Log};

pub struct LogEntry {
    pub msg: String,
    pub level: Level,
}

pub struct Logger {
    sender: SyncSender<LogEntry>,
}

impl Logger {
    pub fn new(sender: SyncSender<LogEntry>) -> Self {
        Self { sender }
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let msg = format!("{}", record.args());

        let _ = self.sender.send(LogEntry {
            msg,
            level: record.level(),
        });
    }

    fn flush(&self) {}
}
