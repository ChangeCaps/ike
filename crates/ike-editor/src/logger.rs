use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use ike::logger::{LogEntry, Logger};

pub struct LogReceiver {
    sender: SyncSender<LogEntry>,
    receiver: Receiver<LogEntry>,
}

impl LogReceiver {
    #[inline]
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel(512);

        Self { sender, receiver }
    }

    #[inline]
    pub fn logger(&self) -> Logger {
        Logger::new(self.sender.clone())
    }

    #[inline]
    pub fn entries(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.receiver.try_iter()
    }
}
