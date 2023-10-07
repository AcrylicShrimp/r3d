use chrono::{DateTime, Utc};
use colored::Color;
use std::{fmt::Display, sync::Arc};
use uuid::Uuid;

pub mod transports;

pub trait LogLevel
where
    Self: 'static + Clone + PartialEq + Eq + Display,
{
    fn color(&self) -> Color;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardLogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl LogLevel for StandardLogLevel {
    fn color(&self) -> Color {
        match self {
            StandardLogLevel::Debug => Color::White,
            StandardLogLevel::Info => Color::BrightWhite,
            StandardLogLevel::Warning => Color::Yellow,
            StandardLogLevel::Error => Color::Red,
            StandardLogLevel::Fatal => Color::BrightRed,
        }
    }
}

impl Display for StandardLogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandardLogLevel::Debug => write!(f, "DEBUG"),
            StandardLogLevel::Info => write!(f, "INFO "),
            StandardLogLevel::Warning => write!(f, "WARN "),
            StandardLogLevel::Error => write!(f, "ERROR"),
            StandardLogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log<L: LogLevel> {
    pub level: L,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

pub struct Logger<L: LogLevel> {
    transports: Vec<Arc<dyn Transport<L>>>,
}

impl<L: LogLevel> Logger<L> {
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
        }
    }

    pub fn wire(&mut self, transport: Arc<dyn Transport<L>>) {
        if self
            .transports
            .iter()
            .any(|item| item.id() == transport.id())
        {
            return;
        }

        self.transports.push(transport);
    }

    pub fn unwire(&mut self, transport: Arc<dyn Transport<L>>) {
        self.transports.retain(|item| item.id() != transport.id());
    }

    pub fn log(&self, level: L, message: impl Into<String>) {
        let log = Log {
            level,
            message: message.into(),
            timestamp: Utc::now(),
        };

        for transport in &self.transports {
            transport.forward(&log);
        }
    }
}

pub trait Transport<L: LogLevel> {
    fn id(&self) -> Uuid;
    fn forward(&self, log: &Log<L>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transports::*;

    #[test]
    fn it_should_log() {
        let mut logger = Logger::new();
        let mut filter =
            FilterTransport::new(vec![StandardLogLevel::Error, StandardLogLevel::Fatal]);
        let transport = Arc::new(ConsoleTransport::new());

        filter.wire(transport);
        logger.wire(Arc::new(filter));

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
