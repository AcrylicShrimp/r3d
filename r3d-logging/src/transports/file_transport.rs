use super::format_timestamp;
use crate::{Log, LogLevel, Transport};
use parking_lot::Mutex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    time::{Duration, Instant},
};
use uuid::Uuid;

/// Controls how often the file is flushed. Note that this is not guaranteed; it will flush the file if internal buffers are full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushMode {
    /// Never flushes the file. It relies on the operating system to flush the file when it is closed. This is useful for low-level logging such as debug messages. Note that it may cause data loss in case of a crash.
    Never,
    /// Flushes the file after a certain amount of time. But the duration is not guaranteed; it will flush the file at the next log after the duration has elapsed. This is useful for medium-level logging such as warnings.
    Interval(Duration),
    /// Flush the file after every log. This is useful for high-level logging such as errors and fatal messages. Note that it will have a significant performance impact.
    Immediate,
}

pub struct FileTransport {
    id: Uuid,
    file: Mutex<BufWriter<File>>,
    flush_mode: FlushMode,
    last_flush: Mutex<Instant>,
}

impl FileTransport {
    pub fn with_file(file: File, flush_mode: FlushMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            file: Mutex::new(BufWriter::new(file)),
            flush_mode,
            last_flush: Mutex::new(Instant::now()),
        }
    }
}

impl<L: LogLevel> Transport<L> for FileTransport {
    fn id(&self) -> Uuid {
        self.id
    }

    fn forward(&self, log: &Log<L>) {
        let timestamp = format_timestamp(log.timestamp);
        let message = log.message.split('\n').collect::<Vec<_>>().join("\n\t");
        let lines = format!("[{}] {} {}\n", timestamp, log.level, message);

        let mut file = self.file.lock();
        file.write_all(lines.as_bytes()).ok();

        let should_flush = match self.flush_mode {
            FlushMode::Never => false,
            FlushMode::Interval(interval) => {
                let now = Instant::now();
                let mut last_flush = self.last_flush.lock();
                let elapsed = now - *last_flush;
                if interval <= elapsed {
                    *last_flush = Instant::now();
                    true
                } else {
                    false
                }
            }
            FlushMode::Immediate => true,
        };

        if should_flush {
            file.flush().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn it_should_log() {
        let mut logger = Logger::new();
        let transport = Arc::new(FileTransport::with_file(
            File::create("test.log").expect("failed to create test.log"),
            FlushMode::Never,
        ));

        logger.wire(transport);

        logger.log(
            StandardLogLevel::Debug,
            "Some debug message\nwith multiple lines",
        );
        logger.log(
            StandardLogLevel::Info,
            "Some info message\nwith multiple lines",
        );
        logger.log(
            StandardLogLevel::Warning,
            "Some warning message\nwith multiple lines",
        );
        logger.log(
            StandardLogLevel::Error,
            "Some error message\nwith multiple lines",
        );
        logger.log(
            StandardLogLevel::Fatal,
            "Some fatal message\nwith multiple lines",
        );
    }
}
