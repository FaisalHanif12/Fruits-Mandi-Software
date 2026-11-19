//! Shared set-up of the assistant tests: the same test environment, first run and sign-in as the
//! services tests (one source of truth), with a key store in memory so no test touches the real
//! Credential Manager.
#![allow(dead_code)]

#[path = "../../../services/tests/common/mod.rs"]
pub mod common;

use std::sync::Arc;

use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::runtime::Runtime;
use mandi_services::auth::{self, Actor};
use mandi_services::testing::TestEnv;
use tempfile::TempDir;

pub use common::{OWNER_PASSWORD, assert_chain_ok, audit_count, raw, set_today};

/// A set-up firm with the Owner signed in, and a runtime whose key store is in memory.
pub fn ready() -> (TempDir, TestEnv, Actor, Runtime) {
    let (dir, env) = common::env();
    common::set_up(&env);
    common::login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
    (dir, env, owner, runtime)
}
