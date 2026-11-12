//! The name of a lot as the prints and reports write it, and the order a seller's lots are printed
//! in (PRD FR-12.45, FR-12.12, plan 71 L1, L7, L8).
//!
//! A lot's name is its seller, its item and a counter: احسان، آم-2. The seller sits in his own
//! place on a page (the party, the seller column), so the lots of one seller are written as
//! `item-counter`: [`lot_label`] gives "آم-2" and "Mango-2". The text is a record name and a number
//! joined by a hyphen, never a sentence, so it needs no Fluent message. The number is always
//! written with the digits 0 to 9 (PRD FR-7.32).
//!
//! A seller's lots print sorted, whatever order they were typed in: by item (the Urdu alphabetical
//! order of the Urdu item name in an Urdu print, the order of the English item name in an English
//! print), then by counter. [`sort_lots`] is the one function that does it, for the seller bill
//! (R1) and the seller's day slip (R35).
//!
//! Example (E2E-70): lots typed آم-3, کیلا-1, آم-2 print آم-2, آم-3, کیلا-1.

use crate::auth::Language;
use crate::text::UrduCollator;

/// A lot written as `item-counter`: "آم-2", "Mango-2".
///
/// ```
/// use mandi_core::bikri::lot_label;
/// assert_eq!(lot_label("آم", 2), "آم-2");
/// assert_eq!(lot_label("Mango", 12), "Mango-12");
/// ```
pub fn lot_label(item_name: &str, counter: i64) -> String {
    format!("{item_name}-{counter}")
}

/// What decides where a lot goes in a sorted list: the names of its item in both scripts, and its
/// counter.
#[derive(Debug, Clone, Copy)]
pub struct LotOrderKey<'a> {
    /// The Urdu name of the item.
    pub item_ur: &'a str,
    /// The English name of the item.
    pub item_en: &'a str,
    /// The counter of the lot's name.
    pub counter: i64,
}

/// Sorts lots by item name, in the order of `language`, then by counter. The sort is stable, so
/// two lots with the same item and counter (which never happens for the lots of one seller) keep
/// the order they came in.
///
/// ```
/// use mandi_core::auth::Language;
/// use mandi_core::bikri::{LotOrderKey, sort_lots};
/// // (item in Urdu, item in English, counter)
/// let mut lots = [("آم", "Mango", 3), ("کیلا", "Banana", 1), ("آم", "Mango", 2)];
/// sort_lots(&mut lots, Language::Ur, |l| LotOrderKey { item_ur: l.0, item_en: l.1, counter: l.2 });
/// assert_eq!(lots, [("آم", "Mango", 2), ("آم", "Mango", 3), ("کیلا", "Banana", 1)]);
/// // English order: Banana comes before Mango.
/// sort_lots(&mut lots, Language::En, |l| LotOrderKey { item_ur: l.0, item_en: l.1, counter: l.2 });
/// assert_eq!(lots, [("کیلا", "Banana", 1), ("آم", "Mango", 2), ("آم", "Mango", 3)]);
/// ```
pub fn sort_lots<T>(
    items: &mut [T],
    language: Language,
    key: impl for<'a> Fn(&'a T) -> LotOrderKey<'a>,
) {
    match language {
        Language::Ur => {
            let collator = UrduCollator::new().ok();
            items.sort_by_cached_key(|item| {
                let k = key(item);
                let name = collator
                    .as_ref()
                    .map_or_else(|| k.item_ur.as_bytes().to_vec(), |c| c.sort_key(k.item_ur));
                (name, k.counter)
            });
        }
        Language::En => {
            items.sort_by_cached_key(|item| {
                let k = key(item);
                (k.item_en.to_lowercase().into_bytes(), k.counter)
            });
        }
    }
}
