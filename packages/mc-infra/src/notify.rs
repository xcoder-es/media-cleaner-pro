use std::sync::Arc;

use mc_core::{NotificationBus, PipelineEvent};

pub struct InMemoryNotifier {
    subscribers: Arc<std::sync::Mutex<Vec<std::sync::mpsc::Sender<PipelineEvent>>>>,
}

impl InMemoryNotifier {
    pub fn new() -> Self {
        InMemoryNotifier {
            subscribers: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> std::sync::mpsc::Receiver<PipelineEvent> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.subscribers.lock().unwrap().push(tx);
        rx
    }
}

impl Default for InMemoryNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationBus for InMemoryNotifier {
    fn broadcast(&self, event: &PipelineEvent) {
        let mut subs = self.subscribers.lock().unwrap();
        subs.retain(|tx| tx.send(event.clone()).is_ok());
    }
}

impl Clone for InMemoryNotifier {
    fn clone(&self) -> Self {
        InMemoryNotifier {
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}
