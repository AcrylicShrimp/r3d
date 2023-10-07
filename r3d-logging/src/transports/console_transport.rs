use super::format_timestamp;
use crate::{Log, LogLevel, Transport};
use colored::Colorize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConsoleTransport {
    id: Uuid,
}

impl ConsoleTransport {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

impl<L: LogLevel> Transport<L> for ConsoleTransport {
    fn id(&self) -> Uuid {
        self.id
    }

    fn forward(&self, log: &Log<L>) {
        let timestamp = format_timestamp(log.timestamp);
        let level = <_ as Colorize>::color(format!("{}", log.level).as_str(), log.level.color());
        let message = log
            .message
            .split('\n')
            .collect::<Vec<_>>()
            .join("\n\t")
            .color(log.level.color());

        println!("[{}] {} {}", timestamp, level, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn it_should_log() {
        let mut logger = Logger::new();
        let transport = Arc::new(ConsoleTransport::new());

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
