use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl<'a> From<&std::panic::Location<'a>> for Location {
    #[inline]
    fn from(location: &std::panic::Location<'a>) -> Self {
        Location {
            file: String::from(location.file()),
            line: location.line(),
            column: location.column(),
        }
    }
}

impl std::fmt::Display for Location {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

pub struct Panic {
    pub message: Option<String>,
    pub location: Option<Location>,
}

pub struct PanicSink {
    sender: SyncSender<Panic>,
}

impl PanicSink {
    pub fn panic(&self, info: &std::panic::PanicInfo<'_>) {
        let payload = info.payload();

        let message = if let Some(msg) = payload.downcast_ref::<&str>() {
            Some(String::from(*msg))
        } else if let Some(msg) = payload.downcast_ref::<String>() {
            Some(msg.clone())
        } else {
            None
        };

        let location = info.location().map(|loc| Location::from(loc));

        let _ = self.sender.send(Panic { message, location });
    }
}

pub type PanicHook = Box<dyn Fn(&std::panic::PanicInfo<'_>) + Send + Sync>;

pub struct Panics {
    sender: SyncSender<Panic>,
    receiver: Receiver<Panic>,
}

impl Default for Panics {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Panics {
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel(16);

        Self { sender, receiver }
    }

    pub fn sink(&self) -> PanicSink {
        PanicSink {
            sender: self.sender.clone(),
        }
    }

    pub fn hook(&self) -> PanicHook {
        let sink = self.sink();

        Box::new(move |info| {
            sink.panic(&info);
        })
    }

    pub fn panics(&self) -> impl Iterator<Item = Panic> + '_ {
        self.receiver.try_iter()
    }
}
