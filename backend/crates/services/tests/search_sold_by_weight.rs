//! Plan 35 D3: a search hit says whether its item is sold by weight, so a screen asks for the
//! kilograms without a second lookup. True only for an item whose `sold_by` is weight; false for a
//! unit item and for every other hit (accounts, packings and the rest).
//!
//! The test also writes the fixture pair `search_masters_items_sold_by_weight` for the frontend
//! (the request is exactly the `input` of `search_masters`). It compares the answer with the file
//! on disk and fails if they differ; after a deliberate change, write the file again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test search_sold_by_weight`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::search::{SearchHit, SearchMastersInput, SearchScope};
use mandi_services::search;
use serde_json::{Value, json};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

fn hits(w: &support::World, scope: SearchScope) -> Vec<SearchHit> {
    // Empty text gives the whole list (plan 32 R7).
    search::search(&w.env.ctx, &w.owner, scope, "", 50).unwrap()
}

#[test]
fn only_an_item_sold_by_weight_has_sold_by_weight_true() {
    let w = support::world();
    set_today(&w.env, 2026, 10, 10);

    // The items: Grapes is sold by weight; Banana, Apple and Radish by the نگ.
    let items = hits(&w, SearchScope::Item);
    let flag = |name: &str| {
        items
            .iter()
            .find(|h| h.name_en == name)
            .unwrap_or_else(|| panic!("{name} is not offered"))
            .sold_by_weight
    };
    assert!(flag("Grapes"), "a weight item says so");
    for unit in ["Banana", "Apple", "Radish"] {
        assert!(!flag(unit), "{unit} is sold by the nag");
    }
    assert_eq!(items.iter().filter(|h| h.sold_by_weight).count(), 1);

    // Every other kind of hit says false: accounts of every field, and the other lists.
    for scope in [
        SearchScope::Seller,
        SearchScope::Buyer,
        SearchScope::AnyAccount,
        SearchScope::Packing,
        SearchScope::City,
        SearchScope::Adda,
    ] {
        let list = hits(&w, scope);
        assert!(
            scope == SearchScope::City || scope == SearchScope::Adda || !list.is_empty(),
            "{scope:?} has hits to check"
        );
        assert!(
            list.iter().all(|h| !h.sold_by_weight),
            "{scope:?} has a hit that says sold by weight"
        );
    }

    // The fixture for the frontend: the item list, as `search_masters` gives it.
    let request: Value = json!({"scope": "item", "text": "", "limit": 50, "offset": 0});
    let input: SearchMastersInput = serde_json::from_value(request.clone()).unwrap();
    let answer = search::search_page(
        &w.env.ctx,
        &w.owner,
        input.scope,
        &input.text,
        input.offset,
        input.limit,
    )
    .unwrap();
    write_or_check(
        "search_masters_items_sold_by_weight_request.json",
        &(serde_json::to_string_pretty(&request).unwrap() + "\n"),
    );
    write_or_check(
        "search_masters_items_sold_by_weight.json",
        &(serde_json::to_string_pretty(&answer).unwrap() + "\n"),
    );
}
