//! What the assistant keeps while the app runs: the place of the key and the conversations (plan
//! 48). One [`Runtime`] is made at start-up and shared by the commands.
//!
//! The key is not kept here: only the store that holds it (Code Standards Q8, NFR-SEC-15).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use mandi_services::Context;

use crate::conversation::Conversation;
use crate::keys::{KeyStore, KeyringStore};

/// The assistant's memory while the app runs.
pub struct Runtime {
    keys: Arc<dyn KeyStore>,
    conversations: AtomicU64,
    memory: Mutex<HashMap<String, Conversation>>,
    busy: AtomicBool,
}

/// Held while a question runs: only one question runs at a time (`ERR_ASSISTANT_BUSY`). Dropping it
/// frees the place, whatever way the question ended.
#[derive(Debug)]
pub struct BusyGuard<'a> {
    flag: &'a AtomicBool,
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").finish_non_exhaustive()
    }
}

impl Runtime {
    /// A runtime over a key store.
    pub fn new(keys: Arc<dyn KeyStore>) -> Self {
        Self {
            keys,
            conversations: AtomicU64::new(0),
            memory: Mutex::new(HashMap::new()),
            busy: AtomicBool::new(false),
        }
    }

    /// The real runtime: the key in Windows Credential Manager.
    pub fn real() -> Self {
        Self::new(Arc::new(KeyringStore))
    }

    /// The place of the key.
    pub fn keys(&self) -> &dyn KeyStore {
        self.keys.as_ref()
    }

    /// A new conversation id, unique in this run of the app and across runs (it holds the time).
    pub fn new_conversation_id(&self, ctx: &Context) -> String {
        let n = self.conversations.fetch_add(1, Ordering::Relaxed);
        let stamp: String = ctx
            .now_text()
            .chars()
            .filter(char::is_ascii_digit)
            .collect();
        format!("c-{stamp}-{n}")
    }

    /// Takes the place to run a question, or `None` when one is running.
    pub fn try_begin(&self) -> Option<BusyGuard<'_>> {
        self.busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .ok()
            .map(|_| BusyGuard { flag: &self.busy })
    }

    /// Takes a conversation out of memory to work on it (put it back with [`Runtime::keep`]).
    pub fn take(&self, id: &str) -> Option<Conversation> {
        self.memory
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id)
    }

    /// Drops a conversation's memory at once (the panel started a new one). `true` when it was
    /// there; an unknown id drops nothing.
    pub fn forget(&self, id: &str) -> bool {
        self.memory
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id)
            .is_some()
    }

    /// Puts a conversation back. A conversation with no turns is not kept.
    pub fn keep(&self, conversation: Conversation) {
        if conversation.turns.is_empty() {
            return;
        }
        self.memory
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(conversation.id.clone(), conversation);
    }
}
