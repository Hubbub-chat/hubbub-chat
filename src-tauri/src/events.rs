use async_trait::async_trait;
use hubbub_domain::errors::DomainError;
use hubbub_domain::events::RunEvent;
use hubbub_domain::ports::event_sink::EventSink;
use tauri::{AppHandle, Emitter};

pub struct TauriEventSink {
    app_handle: AppHandle,
}

impl TauriEventSink {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl EventSink for TauriEventSink {
    async fn emit(&self, event: RunEvent) -> Result<(), DomainError> {
        self.app_handle
            .emit("run_event", event)
            .map_err(|e| DomainError::Internal(format!("Failed to emit Tauri event: {e}")))?;
        Ok(())
    }
}
