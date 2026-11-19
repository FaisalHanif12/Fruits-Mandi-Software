//! The collector boxes (lead's request of 06-11-2026, Umar): the search scope `collector` offers
//! only ACTIVE staff whose job is `collector` (PRD FR-15.2, FR-7.61). A munshi, a labourer and an
//! inactive collector are never offered. The scope is read for each call, so a change of a person's
//! job or of his active flag shows in the very next search. It only reads (Code Standards B29).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{MasterInput, MasterKind, StaffInput};
use mandi_services::core::search::{SearchHit, SearchScope};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, auth, masters, search};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// Saves a staff member with this job, confirming whatever spelling the check asks about.
fn staff(
    env: &TestEnv,
    owner: &auth::Actor,
    ur: &str,
    en: &str,
    job: &str,
    id: Option<i64>,
) -> i64 {
    let mut input = StaffInput {
        id,
        version: id.map(|_| 1),
        name_ur: ur.into(),
        name_en: en.into(),
        code: String::new(),
        job: job.into(),
        phone: String::new(),
        confirmed_rules_words: Vec::new(),
    };
    loop {
        match masters::save(&env.ctx, owner, &MasterInput::Staff(input.clone())) {
            Ok(row) => return row.id(),
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        input
                            .confirmed_rules_words
                            .extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn names(hits: &[SearchHit]) -> Vec<String> {
    hits.iter().map(|h| h.name_en.clone()).collect()
}

fn find(env: &TestEnv, owner: &auth::Actor, scope: SearchScope, text: &str) -> Vec<SearchHit> {
    search::search_page(&env.ctx, owner, scope, text, 0, 50).unwrap()
}

#[test]
fn the_collector_box_lists_only_active_staff_whose_job_is_collector() {
    let (_dir, env, owner) = ready();
    let _munshi = staff(&env, &owner, "آصف", "Asif", "munshi", None);
    let bilal = staff(&env, &owner, "بلال", "Bilal", "collector", None);
    let _labour = staff(&env, &owner, "جمیل", "Jamil", "labour", None);
    let danish = staff(&env, &owner, "دانش", "Danish", "collector", None);

    // The staff box still lists all four; the collector box only the two collectors.
    assert_eq!(find(&env, &owner, SearchScope::Staff, "").len(), 4);
    let box_ = find(&env, &owner, SearchScope::Collector, "");
    assert_eq!(names(&box_), ["Bilal", "Danish"]);
    assert!(
        box_.iter()
            .all(|h| h.kind == SearchScope::Collector && h.is_active)
    );
    // The codes are the staff codes, so a short code works the same way.
    assert!(box_.iter().all(|h| h.code.starts_with("006")), "{box_:?}");

    // Typed text, in both scripts, finds a collector and never a munshi.
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "bilal")),
        ["Bilal"]
    );
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "دانش")),
        ["Danish"]
    );
    assert!(find(&env, &owner, SearchScope::Collector, "asif").is_empty());
    assert!(find(&env, &owner, SearchScope::Collector, "jamil").is_empty());

    // An inactive collector disappears from the box at once, and comes back when active again.
    masters::set_active(&env.ctx, &owner, MasterKind::Staff, danish, false).unwrap();
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "")),
        ["Bilal"]
    );
    masters::set_active(&env.ctx, &owner, MasterKind::Staff, danish, true).unwrap();
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "")),
        ["Bilal", "Danish"]
    );

    // A change of job shows in the next search: Bilal becomes a munshi.
    staff(&env, &owner, "بلال", "Bilal", "munshi", Some(bilal));
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "")),
        ["Danish"]
    );
}

#[test]
fn the_collector_box_remembers_what_was_chosen_and_refuses_a_person_who_is_not_a_collector() {
    let (_dir, env, owner) = ready();
    let munshi = staff(&env, &owner, "آصف", "Asif", "munshi", None);
    let bilal = staff(&env, &owner, "بلال", "Bilal", "collector", None);
    let danish = staff(&env, &owner, "دانش", "Danish", "collector", None);

    // The collector chosen last comes first (PRD FR-7.41).
    search::touch(&env.ctx, &owner, SearchScope::Collector, danish).unwrap();
    assert_eq!(
        names(&find(&env, &owner, SearchScope::Collector, "")),
        ["Danish", "Bilal"]
    );
    assert_eq!(
        names(&search::recent(&env.ctx, &owner, SearchScope::Collector).unwrap()),
        ["Danish", "Bilal"]
    );
    // A munshi is not a record of this box: choosing him is `ERR_NOT_FOUND`.
    let refused = search::touch(&env.ctx, &owner, SearchScope::Collector, munshi).unwrap_err();
    assert!(
        matches!(&refused, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrNotFound)),
        "{refused:?}"
    );
    // The choice does not touch the staff box's own memory.
    assert_eq!(
        names(&search::recent(&env.ctx, &owner, SearchScope::Staff).unwrap())
            .first()
            .map(String::as_str),
        Some("Asif")
    );
    let _ = bilal;
}
