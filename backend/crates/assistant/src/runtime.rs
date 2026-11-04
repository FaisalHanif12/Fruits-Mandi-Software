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
    /// Whether the app finds the party of a plain lookup question before the first call (plan 52
    /// P52-S1, item 10). OFF in the app until the findings of the money agent's identity review
    /// are fixed (a role-only search can pre-find a near name in the wrong role, the shared پختہ
    /// accounts can be found, one ambiguous word does not stop it). The tests of this step switch
    /// it on.
    prefind: AtomicBool,
    /// Whether the app answers by itself, with no model call, a question it understood completely
    /// and whose answer is already in the app: a print, a how-to, a name nobody has (plan 54 S2,
    /// plan 55 Section 2). ON in the app. The tests of the model path (recorded transcripts of
    /// the model's own help and lookup answers) switch it off; the tests of the local answers
    /// switch it on.
    local: AtomicBool,
    /// Whether the app reads the books before the first call when it understood the question
    /// (plan 54 S3). ON in the app; the recorded transcripts of the model's own tool choices switch
    /// it off.
    read_first: AtomicBool,
    /// Whether a call whose data is already read, and help, think "low" (plan 54 S4). ON in the
    /// app until the bank shows a quality drop; the tests of the recorded requests switch it off.
    low_effort: AtomicBool,
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
            prefind: AtomicBool::new(false),
            local: AtomicBool::new(true),
            read_first: AtomicBool::new(true),
            low_effort: AtomicBool::new(true),
        }
    }

    /// True while the app may find the party of a plain lookup question itself.
    pub fn prefind(&self) -> bool {
        self.prefind.load(Ordering::Relaxed)
    }

    /// Switches the pre-find on or off (a test switch; it is off in the app for now).
    pub fn set_prefind(&self, on: bool) {
        self.prefind.store(on, Ordering::Relaxed);
    }

    /// True while the app may answer a completely understood question by itself.
    pub fn local(&self) -> bool {
        self.local.load(Ordering::Relaxed)
    }

    /// Switches the local answers on or off (a test switch; they are on in the app).
    pub fn set_local(&self, on: bool) {
        self.local.store(on, Ordering::Relaxed);
    }

    /// True while the app may read the books before the first call.
    pub fn read_first(&self) -> bool {
        self.read_first.load(Ordering::Relaxed)
    }

    /// Switches the read before the first call on or off (a test switch).
    pub fn set_read_first(&self, on: bool) {
        self.read_first.store(on, Ordering::Relaxed);
    }

    /// True while a call with its data already read thinks "low".
    pub fn low_effort(&self) -> bool {
        self.low_effort.load(Ordering::Relaxed)
    }

    /// Switches the low effort on or off (a test switch).
    pub fn set_low_effort(&self, on: bool) {
        self.low_effort.store(on, Ordering::Relaxed);
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
