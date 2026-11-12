//! The counter of each lot's name (PRD FR-12.45, plan 71 L1 to L5).
//!
//! A lot's name is its seller, its item and a counter: احسان، آم-1, then احسان، آم-2 for his next
//! mango lot. The counter belongs to one seller and one item and runs for the seller's whole life:
//! it never resets and a number is never reused. [`assign`] is the one function that works out the
//! counter of every lot of a draft, for the preview (it shows the number the lot will get) and for
//! the save (it writes that number), so the screen and the books cannot differ (Code Standards B5).
//!
//! The rules, for one lot of the draft:
//!
//! * A lot that is already saved and keeps its seller and its item keeps its counter (a later sale,
//!   an edit that changes something else).
//! * A new lot, or a saved lot whose seller or item an edit changed (L5), gets the next number of
//!   its seller and item: the biggest number ever given to that pair (read from `lot_counters`, so
//!   a cancelled lot and a lot that moved away still count) plus one, plus one for each lot above it
//!   in the same draft that also takes a number of that pair.
//! * A lot with no seller or no item yet has no number.
//!
//! Example (E2E-70): Ehsan has no lot yet. A draft with lots typed آم, کیلا, آم shows آم-1, کیلا-1,
//! آم-2. After it is saved, his next بکری's mango lot shows آم-3.

use std::collections::HashMap;

use mandi_core::bikri::PreparedDraft;

use super::lookups::Lookups;
use super::stored::StoredPosting;

/// The counter of each lot of the draft, in the order of the draft. `None` for a lot with no
/// seller or no item picked yet.
pub(crate) fn assign(
    draft: &PreparedDraft,
    lookups: &Lookups,
    stored: &StoredPosting,
) -> Vec<Option<i64>> {
    // How many numbers this draft has already given to each (seller, item).
    let mut given: HashMap<(i64, i64), i64> = HashMap::new();
    draft
        .lots
        .iter()
        .map(|lot| {
            let pair = lot.seller_account_id.zip(lot.item_id)?;
            if let Some(kept) = kept_counter(lot.lot_id, pair, stored) {
                return Some(kept);
            }
            let before = lookups.lot_counters.get(&pair).copied().unwrap_or(0);
            let taken = given.entry(pair).or_insert(0);
            *taken = taken.saturating_add(1);
            Some(before.saturating_add(*taken))
        })
        .collect()
}

/// The stored counter of a saved lot that keeps its seller and its item. A later sale does not say
/// who the lot belongs to (`identity` is `None`), which counts as unchanged.
fn kept_counter(lot_id: Option<i64>, pair: (i64, i64), stored: &StoredPosting) -> Option<i64> {
    let saved = stored.lots.get(&lot_id?)?;
    let counter = saved.lot_counter?;
    match &saved.identity {
        Some(identity) if (identity.seller_account_id, identity.item_id) != pair => None,
        _ => Some(counter),
    }
}
