//! Advances to sellers in the partners' totals (PRD FR-17.13, C28, C22; plan 78 B4).
//!
//! An advance to a seller (a پیشگی, or an opening debit marked as an advance) is money of the
//! partners that is out with the seller. It lowers each partner's کل رقم by his part, split by the
//! shares of the day of the advance (`split_by_bps`, C10). When the advance is settled, by a بکری
//! credit with the tick on or by the seller's وصولی, the settled amount comes back to the partners
//! by the parts of THAT advance still open (`split_by_open_parts`, C28).
//!
//! This module is pure: the caller reads the advance lines, their shares and the active advance
//! matching rows (`advance_allocations`) and gets every event back. Nothing is stored, so a re-match
//! after an edit or a cancel changes the events with it.
//!
//! * An event is dated on the day of its entry. A settlement whose credit is older than its advance
//!   (an unmatched credit carried forward, PRD FR-15.5) is dated on the day of the advance, so a back
//!   event is never before its out event.
//! * The settlements of one advance are replayed in the order of the credit entries (day, entry,
//!   line), the matching row id only breaks a tie.
//! * Money is `Paisa`, whole rupees, all maths checked.
//!
//! Example (PRD FR-17.13, E2E-31): رحیم's advance of 15,000 at 40 / 30 / 30 is out 6,000 / 4,500 /
//! 4,500. 10,000 settled comes back 4,000 / 3,000 / 3,000; then 5,000 settled comes back 2,000 / 1,500
//! / 1,500. Nothing is open.

use std::collections::HashMap;

use crate::money::{Bps, MoneyError, Paisa, split_by_bps, split_by_open_parts};

/// One advance line: a seller debit that counts as an advance, with the shares that split it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceLineIn {
    /// The journal line.
    pub line_id: i64,
    /// Its journal entry.
    pub entry_id: i64,
    /// The day of the advance, `YYYY-MM-DD`.
    pub date: String,
    /// The seller.
    pub seller_account_id: i64,
    /// The advance, more than 0.
    pub amount: Paisa,
    /// The shares of the day of the advance, by partner id. Empty when no share was ever set: the
    /// advance then has no parts. Otherwise they total 100% (`SharesNot100` if not).
    pub shares: Vec<(i64, Bps)>,
}

/// One active matching row: an amount of an advance that a credit settled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementIn {
    /// The matching row (`advance_allocations.id`).
    pub allocation_id: i64,
    /// The advance line it settles.
    pub advance_line_id: i64,
    /// The journal entry of the credit that settles it.
    pub credit_entry_id: i64,
    /// The day of that entry, `YYYY-MM-DD`.
    pub credit_date: String,
    /// The line of the credit inside its entry.
    pub credit_line_no: i64,
    /// What it settles, more than 0.
    pub amount: Paisa,
}

/// Out or back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceEventKind {
    /// The advance went out: the partner's part comes off his کل رقم.
    Out,
    /// A settlement came back: the partner's part is added back.
    Back,
}

/// One partner's part of an advance event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceEvent {
    /// The partner.
    pub partner_id: i64,
    /// Out or back.
    pub kind: AdvanceEventKind,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The journal entry it belongs to: the advance, or the credit that settled it.
    pub entry_id: i64,
    /// The advance line.
    pub line_id: i64,
    /// The seller.
    pub seller_account_id: i64,
    /// The matching row of a back event; none for an out event.
    pub allocation_id: Option<i64>,
    /// The whole advance (out), or what was settled (back).
    pub amount: Paisa,
    /// The partner's part, more than 0.
    pub part: Paisa,
    /// His share of the advance on its day, bps.
    pub share_bps: i32,
}

/// Why the events could not be worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvanceError {
    /// A money error: the shares do not total 100%, or a sum does not fit.
    Money(MoneyError),
    /// The matching rows of an advance settle more than the advance (a bug in the matching).
    OverSettled {
        /// The advance line.
        line_id: i64,
    },
}

impl From<MoneyError> for AdvanceError {
    fn from(error: MoneyError) -> Self {
        AdvanceError::Money(error)
    }
}

/// Every event of the advances: for each one its out parts, then its back parts in the order of the
/// settlements. The list is in the order of the advance lines given, so it is deterministic.
///
/// # Errors
/// `Money(SharesNot100)` when the shares of an advance do not total 100%; `OverSettled` when its
/// matching rows add up to more than the advance; `Money(Overflow)`.
pub fn replay(
    advances: &[AdvanceLineIn],
    settlements: &[SettlementIn],
) -> Result<Vec<AdvanceEvent>, AdvanceError> {
    let mut by_line: HashMap<i64, Vec<&SettlementIn>> = HashMap::new();
    for settlement in settlements {
        by_line
            .entry(settlement.advance_line_id)
            .or_default()
            .push(settlement);
    }
    let mut events = Vec::new();
    for advance in advances {
        if advance.shares.is_empty() {
            // No share was ever set: nothing is taken off any partner, so nothing comes back.
            continue;
        }
        let parts = split_by_bps(advance.amount, &advance.shares)?;
        let share_of = |partner_id: i64| -> i32 {
            advance
                .shares
                .iter()
                .find(|(id, _)| *id == partner_id)
                .map_or(0, |(_, bps)| bps.get())
        };
        // Only the partners with a part are in it; the others are never given a rupee back.
        let mut open: Vec<(i64, Paisa)> = Vec::with_capacity(parts.len());
        for (partner_id, part) in &parts {
            if *part > Paisa::ZERO {
                open.push((*partner_id, *part));
                events.push(AdvanceEvent {
                    partner_id: *partner_id,
                    kind: AdvanceEventKind::Out,
                    date: advance.date.clone(),
                    entry_id: advance.entry_id,
                    line_id: advance.line_id,
                    seller_account_id: advance.seller_account_id,
                    allocation_id: None,
                    amount: advance.amount,
                    part: *part,
                    share_bps: share_of(*partner_id),
                });
            }
        }
        let mut mine = by_line.remove(&advance.line_id).unwrap_or_default();
        // A credit older than its advance is dated on the advance (it cannot come back before it
        // went out).
        let day_of = |s: &SettlementIn| -> String {
            if s.credit_date.as_str() < advance.date.as_str() {
                advance.date.clone()
            } else {
                s.credit_date.clone()
            }
        };
        mine.sort_by(|a, b| {
            (
                day_of(a),
                a.credit_entry_id,
                a.credit_line_no,
                a.allocation_id,
            )
                .cmp(&(
                    day_of(b),
                    b.credit_entry_id,
                    b.credit_line_no,
                    b.allocation_id,
                ))
        });
        let mut settled = Paisa::ZERO;
        for settlement in mine {
            settled = settled.checked_add(settlement.amount)?;
            if settled > advance.amount {
                return Err(AdvanceError::OverSettled {
                    line_id: advance.line_id,
                });
            }
            let back = split_by_open_parts(settlement.amount, &open)?;
            for ((partner_id, part), (_, open_part)) in back.iter().zip(open.iter_mut()) {
                if *part > Paisa::ZERO {
                    *open_part = open_part.checked_sub(*part)?;
                    events.push(AdvanceEvent {
                        partner_id: *partner_id,
                        kind: AdvanceEventKind::Back,
                        date: day_of(settlement),
                        entry_id: settlement.credit_entry_id,
                        line_id: advance.line_id,
                        seller_account_id: advance.seller_account_id,
                        allocation_id: Some(settlement.allocation_id),
                        amount: settlement.amount,
                        part: *part,
                        share_bps: share_of(*partner_id),
                    });
                }
            }
        }
    }
    Ok(events)
}

/// What a partner's parts of the advances still have out, from the events dated from `from` (or
/// the start) to `to` (or now), both included: Σ out less Σ back. It is taken off his کل رقم
/// (C22). Negative cannot happen for a whole history (a back part never exceeds its out part) but
/// can for a window that holds a back event and not its out event.
///
/// # Errors
/// `Overflow`.
pub fn partner_net(
    events: &[AdvanceEvent],
    partner_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Paisa, MoneyError> {
    events
        .iter()
        .filter(|e| e.partner_id == partner_id)
        .filter(|e| from.is_none_or(|d| e.date.as_str() >= d))
        .filter(|e| to.is_none_or(|d| e.date.as_str() <= d))
        .try_fold(Paisa::ZERO, |sum, e| match e.kind {
            AdvanceEventKind::Out => sum.checked_add(e.part),
            AdvanceEventKind::Back => sum.checked_sub(e.part),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs(r: i64) -> Paisa {
        Paisa::from_rupees(r).unwrap()
    }

    fn shares(list: &[(i64, i32)]) -> Vec<(i64, Bps)> {
        list.iter()
            .map(|&(id, bps)| (id, Bps::new(bps).unwrap()))
            .collect()
    }

    fn rahim(amount: i64) -> AdvanceLineIn {
        AdvanceLineIn {
            line_id: 10,
            entry_id: 1,
            date: "2026-09-01".into(),
            seller_account_id: 7,
            amount: rs(amount),
            shares: shares(&[(1, 4_000), (2, 3_000), (3, 3_000)]),
        }
    }

    fn settle(allocation: i64, entry: i64, date: &str, amount: i64) -> SettlementIn {
        SettlementIn {
            allocation_id: allocation,
            advance_line_id: 10,
            credit_entry_id: entry,
            credit_date: date.into(),
            credit_line_no: 1,
            amount: rs(amount),
        }
    }

    fn parts_of(events: &[AdvanceEvent], kind: AdvanceEventKind) -> Vec<(i64, i64)> {
        events
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| (e.partner_id, e.part.rupees()))
            .collect()
    }

    #[test]
    fn e2e_31_out_then_back_in_two_steps() {
        let events = replay(
            &[rahim(15_000)],
            &[
                settle(1, 5, "2026-10-10", 10_000),
                settle(2, 6, "2026-10-11", 5_000),
            ],
        )
        .unwrap();
        assert_eq!(
            parts_of(&events, AdvanceEventKind::Out),
            vec![(1, 6_000), (2, 4_500), (3, 4_500)]
        );
        // 10,000 then 5,000, in that order.
        assert_eq!(
            parts_of(&events, AdvanceEventKind::Back),
            vec![
                (1, 4_000),
                (2, 3_000),
                (3, 3_000),
                (1, 2_000),
                (2, 1_500),
                (3, 1_500)
            ]
        );
        // Nothing is open at the end, and in between the open parts are 2,000 / 1,500 / 1,500.
        for (id, expect) in [(1, 0), (2, 0), (3, 0)] {
            assert_eq!(partner_net(&events, id, None, None).unwrap(), rs(expect));
        }
        assert_eq!(
            partner_net(&events, 1, None, Some("2026-10-10")).unwrap(),
            rs(2_000)
        );
        assert_eq!(
            partner_net(&events, 2, None, Some("2026-10-10")).unwrap(),
            rs(1_500)
        );
        // Before the advance nothing is out.
        assert_eq!(
            partner_net(&events, 1, None, Some("2026-08-31")).unwrap(),
            Paisa::ZERO
        );
    }

    #[test]
    fn a_part_settlement_of_14_999_follows_c28() {
        let events = replay(&[rahim(15_000)], &[settle(1, 5, "2026-10-10", 14_999)]).unwrap();
        assert_eq!(
            parts_of(&events, AdvanceEventKind::Back),
            vec![(1, 6_000), (2, 4_500), (3, 4_499)]
        );
        // One rupee is still out, on the partner who did not get his last rupee.
        assert_eq!(partner_net(&events, 3, None, None).unwrap(), rs(1));
    }

    #[test]
    fn a_credit_older_than_its_advance_is_dated_on_the_advance() {
        let events = replay(&[rahim(15_000)], &[settle(1, 5, "2026-08-01", 15_000)]).unwrap();
        for e in events.iter().filter(|e| e.kind == AdvanceEventKind::Back) {
            assert_eq!(e.date, "2026-09-01");
        }
        // At every day the open part is never below 0.
        for day in ["2026-08-31", "2026-09-01", "2026-09-02"] {
            for id in [1, 2, 3] {
                assert!(
                    !partner_net(&events, id, None, Some(day))
                        .unwrap()
                        .is_negative()
                );
            }
        }
    }

    #[test]
    fn a_later_share_change_does_not_change_the_back_split() {
        // The advance was split 40 / 30 / 30 on its day; the back split uses its own parts.
        let events = replay(&[rahim(15_000)], &[settle(1, 5, "2026-10-10", 15_000)]).unwrap();
        assert_eq!(
            parts_of(&events, AdvanceEventKind::Back),
            vec![(1, 6_000), (2, 4_500), (3, 4_500)]
        );
    }

    #[test]
    fn no_shares_means_no_parts_and_a_bad_total_is_an_error() {
        let mut none = rahim(15_000);
        none.shares = Vec::new();
        assert!(
            replay(&[none], &[settle(1, 5, "2026-10-10", 1_000)])
                .unwrap()
                .is_empty()
        );
        let mut bad = rahim(15_000);
        bad.shares = shares(&[(1, 4_000), (2, 3_000)]);
        assert_eq!(
            replay(&[bad], &[]),
            Err(AdvanceError::Money(MoneyError::SharesNot100))
        );
    }

    #[test]
    fn more_settled_than_the_advance_is_an_error() {
        assert_eq!(
            replay(
                &[rahim(1_000)],
                &[
                    settle(1, 5, "2026-10-10", 600),
                    settle(2, 6, "2026-10-11", 600)
                ]
            ),
            Err(AdvanceError::OverSettled { line_id: 10 })
        );
    }

    #[test]
    fn a_partner_with_no_share_has_no_event() {
        let mut two = rahim(1_001);
        two.shares = shares(&[(1, 5_000), (2, 5_000), (3, 0)]);
        let events = replay(&[two], &[settle(1, 5, "2026-10-10", 1_001)]).unwrap();
        assert!(events.iter().all(|e| e.partner_id != 3));
    }
}
