use crate::{Log, LogLevel, Transport};
use std::sync::Arc;
use uuid::Uuid;

pub struct FilterTransport<L: LogLevel> {
    id: Uuid,
    levels: Vec<L>,
    transports: Vec<Arc<dyn Transport<L>>>,
}

impl<L: LogLevel> FilterTransport<L> {
    pub fn new(levels: Vec<L>) -> Self {
        Self {
            id: Uuid::new_v4(),
            levels,
            transports: Vec::new(),
        }
    }

    pub fn levels(&self) -> &[L] {
        &self.levels
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
}

impl<L: LogLevel> Transport<L> for FilterTransport<L> {
    fn id(&self) -> Uuid {
        self.id
    }

    fn forward(&self, log: &Log<L>) {
        if !self.levels.contains(&log.level) {
            return;
        }

        for transport in &self.transports {
            transport.forward(log);
        }
    }
}
