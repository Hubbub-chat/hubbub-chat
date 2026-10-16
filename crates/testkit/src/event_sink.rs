use async_trait::async_trait;
use hubbub_domain::errors::DomainError;
use hubbub_domain::events::RunEvent;
use hubbub_domain::ports::event_sink::EventSink;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct InMemoryEventSink {
    events: Arc<Mutex<Vec<RunEvent>>>,
}

impl InMemoryEventSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_events(&self) -> Vec<RunEvent> {
        let lock = self.events.lock().await;
        lock.clone()
    }

    pub async fn get_deltas(&self) -> Vec<String> {
        let lock = self.events.lock().await;
        lock.iter()
            .filter_map(|e| match e {
                RunEvent::MessageDelta { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect()
    }

    pub async fn full_text(&self) -> String {
        self.get_deltas().await.join("")
    }

    pub async fn has_finished_status(&self, expected_status: &str) -> bool {
        let lock = self.events.lock().await;
        lock.iter().any(|e| match e {
            RunEvent::RunFinished { status, .. } => status == expected_status,
            _ => false,
        })
    }
}

#[async_trait]
impl EventSink for InMemoryEventSink {
    async fn emit(&self, event: RunEvent) -> Result<(), DomainError> {
        let mut lock = self.events.lock().await;
        lock.push(event);
        Ok(())
    }
}
