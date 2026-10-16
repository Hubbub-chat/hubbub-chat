//! One run at a time per conversation.
//!
//! Two runs on the same conversation interleave their messages and split tool
//! call/result pairs. The set is only locked briefly and never across `.await`.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default, Clone)]
pub(crate) struct ActiveConversations(Arc<Mutex<HashSet<Uuid>>>);

impl ActiveConversations {
    /// Returns a guard if no run is active on `id`; the slot is freed when the guard drops.
    pub(crate) fn try_acquire(&self, id: Uuid) -> Option<ConversationGuard> {
        let mut set = self.0.lock().unwrap_or_else(|p| p.into_inner());
        set.insert(id).then(|| ConversationGuard {
            set: self.0.clone(),
            id,
        })
    }
}

pub(crate) struct ConversationGuard {
    set: Arc<Mutex<HashSet<Uuid>>>,
    id: Uuid,
}

impl Drop for ConversationGuard {
    fn drop(&mut self) {
        let mut set = self.set.lock().unwrap_or_else(|p| p.into_inner());
        set.remove(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_run_on_same_conversation_is_rejected_until_first_ends() {
        let active = ActiveConversations::default();
        let conv = Uuid::now_v7();

        let first = active.try_acquire(conv);
        assert!(first.is_some());
        assert!(active.try_acquire(conv).is_none());
        assert!(active.try_acquire(Uuid::now_v7()).is_some());

        drop(first);
        assert!(active.try_acquire(conv).is_some());
    }
}
