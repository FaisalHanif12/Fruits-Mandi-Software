//! The change list of an edit of a بکری (PRD FR-7.70): the saved بکری before and after, compared
//! header by header, lot by lot and line by line. A lot is the same lot by its number. An edit writes
//! every line of its posting again as a new row (PRD FR-12.19), so a line is followed through the
//! rows that replaced it (`replaced_by_sale_id`) and is the same line to the screen. Only lines in
//! force count; a line with no row after it was taken out, and a row nothing replaced was added.
//!
//! Every value is typed (money, weight, names, a choice), so the screen writes "رحیم، لائن 1: ریٹ
//! 600 تھا، اب 610" without working anything out. The commission, لاگا and totals that moved with
//! the rate are in the list too, next to the field that moved them.

use std::collections::HashMap;

use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::bikri::{BikriDetail, SaleState, SavedLot, SavedSale};
use mandi_domain::core::edit::{ChangeValue, EditChange, EditPlace, EditPlaceKind};
use mandi_domain::core::money::Paisa;
use serde::Serialize;

/// One field of a place, as it is compared. A party also carries his record, so two people of the
/// same name are told apart by their account and not only by the name shown (PRD FR-7.70).
struct Field<'a> {
    name: &'static str,
    value: ChangeValue,
    party: Option<&'a RecordRef>,
}

type Fields<'a> = Vec<Field<'a>>;

/// A field that is not a party.
fn plain(name: &'static str, value: ChangeValue) -> Field<'static> {
    Field {
        name,
        value,
        party: None,
    }
}

/// A field that holds a party: shown with his tag after the name, as `displayName` shows him
/// (PRD 10.2), and compared by his account.
fn party<'a>(name: &'static str, record: &'a RecordRef) -> Field<'a> {
    Field {
        name,
        value: tagged(record, false),
        party: Some(record),
    }
}

/// A party in both scripts. The tag follows the name, so اسلم and اسلم BA read as two people. With
/// `code`, his code follows too, for the rare case of two accounts with the same name and tag.
fn tagged(record: &RecordRef, code: bool) -> ChangeValue {
    let show = |name: &str| {
        let mut text = name.to_owned();
        if let Some(tag) = record.tag.as_deref().filter(|t| !t.is_empty()) {
            text.push(' ');
            text.push_str(tag);
        }
        if code && !record.code.is_empty() {
            text.push_str(" (");
            text.push_str(&record.code);
            text.push(')');
        }
        text
    };
    ChangeValue::Names {
        ur: show(&record.name_ur),
        en: show(&record.name_en),
    }
}

/// A saved بکری with the rows that replaced its lines, as the rehearsal reads it.
pub(crate) struct Snapshot {
    /// The saved بکری.
    pub detail: BikriDetail,
    /// Each replaced sale row and the row that took its place.
    pub replaced_by: HashMap<i64, i64>,
}

/// The row a line ended on, following the rows that replaced it.
fn last_row(start: i64, replaced_by: &HashMap<i64, i64>) -> i64 {
    let mut id = start;
    // A chain is short; the bound stops a loop that cannot happen.
    for _ in 0..replaced_by.len().saturating_add(1) {
        match replaced_by.get(&id) {
            Some(next) => id = *next,
            None => break,
        }
    }
    id
}

fn money(paisa: Paisa) -> ChangeValue {
    ChangeValue::Money { paisa }
}

fn opt_money(paisa: Option<Paisa>) -> ChangeValue {
    paisa.map_or(ChangeValue::Empty, money)
}

fn number(value: i64) -> ChangeValue {
    ChangeValue::Number { value }
}

fn text(value: Option<&String>) -> ChangeValue {
    match value {
        Some(text) if !text.is_empty() => ChangeValue::Text { text: text.clone() },
        _ => ChangeValue::Empty,
    }
}

fn names(record: &RecordRef) -> ChangeValue {
    ChangeValue::Names {
        ur: record.name_ur.clone(),
        en: record.name_en.clone(),
    }
}

/// An enum as the fixed word it is written with (`credit`, `firm`, `per_unit`).
fn word<T: Serialize>(value: &T) -> ChangeValue {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(word)) => ChangeValue::Choice { word },
        _ => ChangeValue::Empty,
    }
}

fn header_fields(d: &BikriDetail) -> Fields<'_> {
    vec![
        plain("book", text(d.book.as_ref())),
        plain(
            "bill_date",
            ChangeValue::Date {
                iso: d.bill_date.clone(),
            },
        ),
        plain("truck_no", text(d.truck_no.as_ref())),
        d.zamindar.as_ref().map_or_else(
            || plain("zamindar", ChangeValue::Empty),
            |zamindar| party("zamindar", zamindar),
        ),
        plain("driver_name", text(d.driver_name.as_ref())),
        plain("driver_phone", text(d.driver_phone.as_ref())),
        plain("total_freight", opt_money(d.total_freight)),
        plain("note", text(d.note.as_ref())),
    ]
}

fn lot_fields(l: &SavedLot) -> Fields<'_> {
    vec![
        party("seller", &l.seller),
        plain("item", names(&l.item)),
        plain("packing", names(&l.packing)),
        plain("marka", text(l.marka_text.as_ref())),
        plain("qty_received", number(l.qty_received)),
        plain(
            "weight_received",
            l.weight_received_grams
                .map_or(ChangeValue::Empty, |grams| ChangeValue::Weight { grams }),
        ),
        plain("freight", money(l.freight)),
        plain("freight_paid_by", word(&l.freight_paid_by)),
        plain("labour", money(l.labour.amount)),
        plain("labour_method", word(&l.labour.method)),
        plain("munshiana", money(l.munshiana.amount)),
        plain("munshiana_method", word(&l.munshiana.method)),
        plain("phone", money(l.phone.amount)),
        plain("phone_method", word(&l.phone.method)),
        plain("cash_paid", money(l.cash_paid)),
        plain("store_rent", money(l.store_rent)),
        plain(
            "adjust_advance",
            ChangeValue::Flag {
                on: l.adjust_advance,
            },
        ),
        plain("seller_comm", money(l.seller_comm)),
        plain("gross", money(l.gross)),
        plain("net", money(l.net)),
    ]
}

fn line_fields(s: &SavedSale) -> Fields<'_> {
    vec![
        party("buyer", &s.buyer),
        plain(
            "chukta",
            s.chukta
                .as_ref()
                .map_or(ChangeValue::Empty, |p| ChangeValue::Names {
                    ur: p.name_ur.clone(),
                    en: p.name_en.clone(),
                }),
        ),
        plain("qty", number(s.qty)),
        plain(
            "weight",
            s.weight_grams
                .map_or(ChangeValue::Empty, |grams| ChangeValue::Weight { grams }),
        ),
        plain("seller_rate", money(s.seller_rate)),
        plain("seller_amount", money(s.seller_amount)),
        plain("seller_comm", money(s.seller_comm.amount)),
        plain("buyer_rate", money(s.buyer_rate)),
        plain("buyer_value", money(s.buyer_value)),
        plain("buyer_comm", money(s.buyer_comm.amount)),
        plain("laga", money(s.laga.amount)),
        plain("buyer_total", money(s.buyer_total)),
        plain("rate_diff", money(s.rate_diff)),
        plain("paid", money(s.paid_paisa)),
        plain("note", text(s.note.as_ref())),
    ]
}

/// The changes between two sets of fields of one place. A side that is not there is empty. A party
/// changed when his account changed, even if the name shown is the same; then the code is shown too.
fn compare(
    place: &EditPlace,
    before: Option<Fields<'_>>,
    after: Option<Fields<'_>>,
    out: &mut Vec<EditChange>,
) {
    let template = after.as_ref().or(before.as_ref());
    let Some(template) = template else {
        return;
    };
    for (index, field) in template.iter().enumerate() {
        let b = before.as_ref().and_then(|f| f.get(index));
        let a = after.as_ref().and_then(|f| f.get(index));
        let mut before_value = b.map_or(ChangeValue::Empty, |f| f.value.clone());
        let mut after_value = a.map_or(ChangeValue::Empty, |f| f.value.clone());
        let other_account = match (b.and_then(|f| f.party), a.and_then(|f| f.party)) {
            (Some(x), Some(y)) if x.id != y.id => Some((x, y)),
            _ => None,
        };
        if let Some((x, y)) = other_account
            && before_value == after_value
        {
            before_value = tagged(x, true);
            after_value = tagged(y, true);
        }
        if before_value != after_value {
            out.push(EditChange {
                place: place.clone(),
                field: field.name.to_owned(),
                before: before_value,
                after: after_value,
            });
        }
    }
}

fn lot_place(lot: &SavedLot) -> EditPlace {
    EditPlace {
        kind: EditPlaceKind::Lot,
        lot_no: Some(lot.lot_no),
        line_no: None,
        label_ur: Some(lot.seller.name_ur.clone()),
        label_en: Some(lot.seller.name_en.clone()),
    }
}

/// The place of a line: the ordinal of the line among the lines of its lot as the screen counts
/// them, from 1 (the number inside the stored row is a running number of the whole بکری).
fn line_place(lot: &SavedLot, sale: &SavedSale, ordinal: usize) -> EditPlace {
    EditPlace {
        kind: EditPlaceKind::Line,
        lot_no: Some(lot.lot_no),
        line_no: i64::try_from(ordinal.saturating_add(1)).ok(),
        label_ur: Some(sale.buyer.name_ur.clone()),
        label_en: Some(sale.buyer.name_en.clone()),
    }
}

fn in_force(lot: &SavedLot) -> Vec<&SavedSale> {
    lot.sales
        .iter()
        .filter(|s| s.state == SaleState::InForce)
        .collect()
}

/// The changes between a بکری before and after an edit.
pub(crate) fn changes(before: &Snapshot, after_snapshot: &Snapshot) -> Vec<EditChange> {
    let replaced_by = &after_snapshot.replaced_by;
    let (before, after) = (&before.detail, &after_snapshot.detail);
    let mut out = Vec::new();
    let header = EditPlace {
        kind: EditPlaceKind::Header,
        lot_no: None,
        line_no: None,
        label_ur: None,
        label_en: None,
    };
    compare(
        &header,
        Some(header_fields(before)),
        Some(header_fields(after)),
        &mut out,
    );
    let mut lot_numbers: Vec<i64> = before
        .lots
        .iter()
        .chain(after.lots.iter())
        .map(|l| l.lot_no)
        .collect();
    lot_numbers.sort_unstable();
    lot_numbers.dedup();
    for lot_no in lot_numbers {
        let b = before.lots.iter().find(|l| l.lot_no == lot_no);
        let a = after.lots.iter().find(|l| l.lot_no == lot_no);
        let Some(shown) = a.or(b) else {
            continue;
        };
        compare(
            &lot_place(shown),
            b.map(lot_fields),
            a.map(lot_fields),
            &mut out,
        );
        let b_lines = b.map(in_force).unwrap_or_default();
        let a_lines = a.map(in_force).unwrap_or_default();
        let mut matched = vec![false; a_lines.len()];
        for (index, old) in b_lines.iter().enumerate() {
            let target = last_row(old.id, replaced_by);
            let position = a_lines.iter().position(|s| s.id == target);
            if let Some(slot) = position.and_then(|p| matched.get_mut(p)) {
                *slot = true;
            }
            let new = position.and_then(|p| a_lines.get(p)).copied();
            let ordinal = position.unwrap_or(index);
            compare(
                &line_place(shown, new.unwrap_or(old), ordinal),
                Some(line_fields(old)),
                new.map(line_fields),
                &mut out,
            );
        }
        for (index, new) in a_lines.iter().enumerate() {
            if !matched.get(index).copied().unwrap_or(true) {
                compare(
                    &line_place(shown, new, index),
                    None,
                    Some(line_fields(new)),
                    &mut out,
                );
            }
        }
    }
    out
}
