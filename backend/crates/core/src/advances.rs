//! Advances to sellers in the partners' totals (PRD FR-17.13 v1.61, C28, C22; plan 78 B4, plan
//! 100).
//!
//! The partners carry only the part of a seller's open advance that the seller's own credit does not
//! cover (the covered-advance rule of PRD FR-9.6): `uncovered = min(open advance, what his balance
//! says he owes)`. It is worked out after each of his live journal entries:
//!
//! * the uncovered part is shared over his advances NEWEST first, so his credit covers the oldest
//!   first, as matching takes the oldest (PRD FR-15.8);
//! * each advance has its full parts, split by the shares of its own day (`split_by_bps`, C10);
//! * when an advance's uncovered part grows (it goes out, or a payment makes it uncovered again),
//!   the growth is split over the parts not out (`split_by_open_parts`, C28), so a full uncovered
//!   advance is out exactly by its full parts;
//! * when it shrinks, what comes back is split over the parts out (C28). It is `Back` when the entry
//!   settles that advance (a ticked بکری credit or the seller's وصولی, "پیشگی واپس") and `Covered`
//!   when his credit comes to cover it without a settlement ("بقایا میں سے کٹ گئی", a back event
//!   with `covered` set).
//!
//! This module is pure: the caller reads the advance lines, their shares, the active advance matching
//! rows (`advance_allocations`) and each seller's live entries, and gets every event back. Nothing is
//! stored, so a re-match after an edit or a cancel changes the events with it.
//!
//! * An event is dated on the day of the entry that made it.
//! * A settlement counts from the entry of its credit, or from the advance's entry when the credit
//!   is older (an unmatched credit carried forward, PRD FR-15.5). A settlement whose credit entry is
//!   not live is not counted.
//! * An advance with no shares ever set has no parts, but it still takes its place in the sharing.
//! * Money is `Paisa`, whole rupees, all maths checked.
//!
//! Example (PRD FR-17.13): فیصل's advance of 100,000 at 50 / 50 is out 50,000 / 50,000. His بکری
//! credit of 117,040 (tick off) covers it: 50,000 / 50,000 come back as `Covered`. A payment of
//! 117,040 uncovers it and it goes out again.

use std::collections::{BTreeMap, HashMap};

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

/// One live journal entry of a seller with an advance, in the order of his ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellerStepIn {
    /// The seller.
    pub seller_account_id: i64,
    /// The journal entry.
    pub entry_id: i64,
    /// Its day, `YYYY-MM-DD`.
    pub date: String,
    /// Σ credits less Σ debits of the seller's lines of the entry (signed).
    pub net_credit: Paisa,
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
    /// The line of the credit inside its entry.
    pub credit_line_no: i64,
    /// What it settles, more than 0.
    pub amount: Paisa,
}

/// Out or back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceEventKind {
    /// The advance went out, or came to be uncovered again: the partner's part comes off his کل رقم.
    Out,
    /// A settlement brought it back, or the seller's credit came to cover it: the partner's part
    /// is added back.
    Back,
}

/// One partner's part of an advance event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceEvent {
    /// The partner.
    pub partner_id: i64,
    /// Out or back.
    pub kind: AdvanceEventKind,
    /// True for a back event with no settlement: the seller's credit came to cover it.
    pub covered: bool,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The journal entry that made it.
    pub entry_id: i64,
    /// The advance line.
    pub line_id: i64,
    /// The seller.
    pub seller_account_id: i64,
    /// The first matching row of a back event; none otherwise.
    pub allocation_id: Option<i64>,
    /// What of the advance went out or came back with this event.
    pub amount: Paisa,
    /// The partner's part of it, more than 0.
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
    /// An advance line whose entry is not one of its seller's live entries (a bug in the reading).
    NoStep {
        /// The advance line.
        line_id: i64,
    },
}

impl From<MoneyError> for AdvanceError {
    fn from(error: MoneyError) -> Self {
        AdvanceError::Money(error)
    }
}

/// One advance while its seller's entries are walked.
struct Walked<'a> {
    advance: &'a AdvanceLineIn,
    /// The parts above 0 of the whole advance, by the shares of its day.
    full: Vec<(i64, Paisa)>,
    /// What of each of those parts is out now, in the same order.
    open: Vec<(i64, Paisa)>,
    /// The advance less its settlements so far.
    remaining: Paisa,
}

impl<'a> Walked<'a> {
    fn new(advance: &'a AdvanceLineIn) -> Result<Self, MoneyError> {
        let full: Vec<(i64, Paisa)> = if advance.shares.is_empty() {
            Vec::new()
        } else {
            split_by_bps(advance.amount, &advance.shares)?
                .into_iter()
                .filter(|(_, part)| *part > Paisa::ZERO)
                .collect()
        };
        let open = full.iter().map(|(id, _)| (*id, Paisa::ZERO)).collect();
        Ok(Self {
            advance,
            full,
            open,
            remaining: Paisa::ZERO,
        })
    }

    fn out(&self) -> Result<Paisa, MoneyError> {
        total(self.open.iter().map(|(_, part)| *part))
    }

    /// Moves what is out to `uncovered`, giving the events of this entry.
    fn move_to(
        &mut self,
        uncovered: Paisa,
        step: &SellerStepIn,
        settled_by: Option<i64>,
        events: &mut Vec<AdvanceEvent>,
    ) -> Result<(), MoneyError> {
        if self.full.is_empty() {
            return Ok(());
        }
        let now = self.out()?;
        let (kind, covered, amount, parts) = if uncovered > now {
            let more = uncovered.checked_sub(now)?;
            let mut not_out = Vec::with_capacity(self.full.len());
            for ((id, full), (_, open)) in self.full.iter().zip(&self.open) {
                not_out.push((*id, full.checked_sub(*open)?));
            }
            let parts = split_by_open_parts(more, &not_out)?;
            (AdvanceEventKind::Out, false, more, parts)
        } else if uncovered < now {
            let less = now.checked_sub(uncovered)?;
            let parts = split_by_open_parts(less, &self.open)?;
            (AdvanceEventKind::Back, settled_by.is_none(), less, parts)
        } else {
            return Ok(());
        };
        for ((partner_id, part), (_, open)) in parts.iter().zip(self.open.iter_mut()) {
            if *part == Paisa::ZERO {
                continue;
            }
            *open = match kind {
                AdvanceEventKind::Out => open.checked_add(*part)?,
                AdvanceEventKind::Back => open.checked_sub(*part)?,
            };
            let share_bps = self
                .advance
                .shares
                .iter()
                .find(|(id, _)| id == partner_id)
                .map_or(0, |(_, bps)| bps.get());
            events.push(AdvanceEvent {
                partner_id: *partner_id,
                kind,
                covered,
                date: step.date.clone(),
                entry_id: step.entry_id,
                line_id: self.advance.line_id,
                seller_account_id: self.advance.seller_account_id,
                allocation_id: match kind {
                    AdvanceEventKind::Back => settled_by,
                    AdvanceEventKind::Out => None,
                },
                amount,
                part: *part,
                share_bps,
            });
        }
        Ok(())
    }
}

fn total(parts: impl Iterator<Item = Paisa>) -> Result<Paisa, MoneyError> {
    let mut sum = Paisa::ZERO;
    for part in parts {
        sum = sum.checked_add(part)?;
    }
    Ok(sum)
}

/// Walks one seller's live entries. `advances` are his advance lines in the order of his ledger.
fn walk_seller(
    advances: &[&AdvanceLineIn],
    settlements: &[&SettlementIn],
    steps: &[&SellerStepIn],
    events: &mut Vec<AdvanceEvent>,
) -> Result<(), AdvanceError> {
    let at: HashMap<i64, usize> = steps
        .iter()
        .enumerate()
        .map(|(index, step)| (step.entry_id, index))
        .collect();
    let mut walked = Vec::with_capacity(advances.len());
    let mut starts: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut position_of: HashMap<i64, (usize, usize)> = HashMap::new();
    for (index, advance) in advances.iter().enumerate() {
        let Some(position) = at.get(&advance.entry_id).copied() else {
            return Err(AdvanceError::NoStep {
                line_id: advance.line_id,
            });
        };
        walked.push(Walked::new(advance)?);
        starts.entry(position).or_default().push(index);
        position_of.insert(advance.line_id, (index, position));
    }
    // The settlements that count from each step, in the order of their credits.
    let mut settled_at: HashMap<usize, Vec<(usize, usize, &SettlementIn)>> = HashMap::new();
    for settlement in settlements {
        let Some((index, advance_step)) = position_of.get(&settlement.advance_line_id).copied()
        else {
            continue;
        };
        // A credit that is not live has no step: it settles nothing.
        let Some(credit_step) = at.get(&settlement.credit_entry_id).copied() else {
            continue;
        };
        settled_at
            .entry(credit_step.max(advance_step))
            .or_default()
            .push((credit_step, index, settlement));
    }
    for list in settled_at.values_mut() {
        list.sort_by_key(|(credit_step, _, s)| (*credit_step, s.credit_line_no, s.allocation_id));
    }

    let mut balance = Paisa::ZERO;
    // The advances with something left or out, oldest first.
    let mut live: Vec<usize> = Vec::new();
    for (k, step) in steps.iter().enumerate() {
        balance = balance
            .checked_add(step.net_credit)
            .map_err(AdvanceError::Money)?;
        for index in starts.get(&k).map(Vec::as_slice).unwrap_or_default() {
            if let Some(w) = walked.get_mut(*index) {
                w.remaining = w.advance.amount;
                live.push(*index);
            }
        }
        let mut settled_now: HashMap<usize, i64> = HashMap::new();
        for (_, index, settlement) in settled_at.get(&k).map(Vec::as_slice).unwrap_or_default() {
            if let Some(w) = walked.get_mut(*index) {
                w.remaining = w.remaining.checked_sub(settlement.amount)?;
                if w.remaining.is_negative() {
                    return Err(AdvanceError::OverSettled {
                        line_id: w.advance.line_id,
                    });
                }
                settled_now
                    .entry(*index)
                    .or_insert(settlement.allocation_id);
            }
        }
        if live.is_empty() {
            continue;
        }
        let open_advance = total(
            live.iter()
                .filter_map(|i| walked.get(*i).map(|w| w.remaining)),
        )?;
        let owed = if balance.is_negative() {
            balance.checked_neg()?
        } else {
            Paisa::ZERO
        };
        let mut left = open_advance.min(owed);
        // Newest first: his credit covers the oldest advances first.
        for index in live.iter().rev() {
            if let Some(w) = walked.get_mut(*index) {
                let uncovered = w.remaining.min(left);
                left = left.checked_sub(uncovered)?;
                w.move_to(uncovered, step, settled_now.get(index).copied(), events)?;
            }
        }
        let mut keep = Vec::with_capacity(live.len());
        for index in live {
            if let Some(w) = walked.get(index)
                && (w.remaining > Paisa::ZERO || w.out()? > Paisa::ZERO)
            {
                keep.push(index);
            }
        }
        live = keep;
    }
    Ok(())
}

/// Every event of the advances, seller by seller and in the order of his entries. `advances` and
/// `steps` are given in the order of each seller's ledger.
///
/// # Errors
/// `Money(SharesNot100)` when the shares of an advance do not total 100%; `OverSettled` when its
/// matching rows add up to more than the advance; `NoStep` when an advance's entry is not among
/// its seller's steps; `Money(Overflow)`.
pub fn replay(
    advances: &[AdvanceLineIn],
    settlements: &[SettlementIn],
    steps: &[SellerStepIn],
) -> Result<Vec<AdvanceEvent>, AdvanceError> {
    let mut by_seller: BTreeMap<i64, Vec<&AdvanceLineIn>> = BTreeMap::new();
    let mut seller_of_line: HashMap<i64, i64> = HashMap::new();
    for advance in advances {
        by_seller
            .entry(advance.seller_account_id)
            .or_default()
            .push(advance);
        seller_of_line.insert(advance.line_id, advance.seller_account_id);
    }
    let mut settlements_of: HashMap<i64, Vec<&SettlementIn>> = HashMap::new();
    for settlement in settlements {
        if let Some(seller) = seller_of_line.get(&settlement.advance_line_id) {
            settlements_of.entry(*seller).or_default().push(settlement);
        }
    }
    let mut steps_of: HashMap<i64, Vec<&SellerStepIn>> = HashMap::new();
    for step in steps {
        steps_of
            .entry(step.seller_account_id)
            .or_default()
            .push(step);
    }
    let mut events = Vec::new();
    for (seller, lines) in by_seller {
        walk_seller(
            &lines,
            settlements_of
                .get(&seller)
                .map(Vec::as_slice)
                .unwrap_or_default(),
            steps_of.get(&seller).map(Vec::as_slice).unwrap_or_default(),
            &mut events,
        )?;
    }
    Ok(events)
}

/// What a partner's parts of the advances still have out, from the events dated from `from` (or
/// the start) to `to` (or now), both included: Σ out less Σ back. It is taken off his
/// کل رقم (C22). Negative cannot happen for a whole history (nothing comes back that did not go out)
/// but can for a window that holds a back event and not its out event.
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
    use proptest::prelude::*;

    use super::*;

    fn rs(r: i64) -> Paisa {
        Paisa::from_rupees(r).unwrap()
    }

    fn shares(list: &[(i64, i32)]) -> Vec<(i64, Bps)> {
        list.iter()
            .map(|&(id, bps)| (id, Bps::new(bps).unwrap()))
            .collect()
    }

    const THREE: [(i64, i32); 3] = [(1, 4_000), (2, 3_000), (3, 3_000)];

    fn line(line_id: i64, entry: i64, amount: i64, split: &[(i64, i32)]) -> AdvanceLineIn {
        AdvanceLineIn {
            line_id,
            entry_id: entry,
            date: format!("2026-09-{entry:02}"),
            seller_account_id: 7,
            amount: rs(amount),
            shares: shares(split),
        }
    }

    /// رحیم's advance of 15,000 at 40 / 30 / 30, in entry 1.
    fn rahim(amount: i64) -> AdvanceLineIn {
        line(10, 1, amount, &THREE)
    }

    /// An entry of the seller that moves his ledger by `net_credit` rupees.
    fn step(entry: i64, net_credit: i64) -> SellerStepIn {
        let magnitude = rs(net_credit.abs());
        SellerStepIn {
            seller_account_id: 7,
            entry_id: entry,
            date: format!("2026-09-{entry:02}"),
            net_credit: if net_credit < 0 {
                magnitude.checked_neg().unwrap()
            } else {
                magnitude
            },
        }
    }

    fn settle(allocation: i64, advance: i64, entry: i64, amount: i64) -> SettlementIn {
        SettlementIn {
            allocation_id: allocation,
            advance_line_id: advance,
            credit_entry_id: entry,
            credit_line_no: 1,
            amount: rs(amount),
        }
    }

    /// What happened, as the statement words it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Told {
        Out,
        Back,
        Covered,
    }
    use Told::{Back, Covered, Out};

    fn told(e: &AdvanceEvent) -> Told {
        match (e.kind, e.covered) {
            (AdvanceEventKind::Out, _) => Out,
            (AdvanceEventKind::Back, false) => Back,
            (AdvanceEventKind::Back, true) => Covered,
        }
    }

    fn parts_of(events: &[AdvanceEvent], kind: Told) -> Vec<(i64, i64)> {
        events
            .iter()
            .filter(|e| told(e) == kind)
            .map(|e| (e.partner_id, e.part.rupees()))
            .collect()
    }

    fn of_entry(events: &[AdvanceEvent], entry: i64) -> Vec<(Told, i64, i64, i64)> {
        events
            .iter()
            .filter(|e| e.entry_id == entry)
            .map(|e| (told(e), e.partner_id, e.part.rupees(), e.amount.rupees()))
            .collect()
    }

    #[test]
    fn e2e_31_out_then_back_in_two_steps() {
        // The receipt of 10,000, then بکری 1 (24,316 less نقد 5,000) settling the other 5,000.
        let events = replay(
            &[rahim(15_000)],
            &[settle(1, 10, 5, 10_000), settle(2, 10, 6, 5_000)],
            &[step(1, -15_000), step(5, 10_000), step(6, 19_316)],
        )
        .unwrap();
        assert_eq!(
            parts_of(&events, Out),
            vec![(1, 6_000), (2, 4_500), (3, 4_500)]
        );
        assert_eq!(
            parts_of(&events, Back),
            vec![
                (1, 4_000),
                (2, 3_000),
                (3, 3_000),
                (1, 2_000),
                (2, 1_500),
                (3, 1_500)
            ]
        );
        assert!(parts_of(&events, Covered).is_empty());
        for id in [1, 2, 3] {
            assert_eq!(partner_net(&events, id, None, None).unwrap(), Paisa::ZERO);
        }
        assert_eq!(
            partner_net(&events, 1, None, Some("2026-09-05")).unwrap(),
            rs(2_000)
        );
        assert_eq!(
            partner_net(&events, 1, None, Some("2026-08-31")).unwrap(),
            Paisa::ZERO
        );
    }

    #[test]
    fn a_part_settlement_of_14_999_follows_c28() {
        let events = replay(
            &[rahim(15_000)],
            &[settle(1, 10, 5, 14_999)],
            &[step(1, -15_000), step(5, 14_999)],
        )
        .unwrap();
        assert_eq!(
            parts_of(&events, Back),
            vec![(1, 6_000), (2, 4_500), (3, 4_499)]
        );
        assert_eq!(partner_net(&events, 3, None, None).unwrap(), rs(1));
    }

    #[test]
    fn e2e_28_the_credit_covers_the_advance_and_payments_take_it_out_again() {
        // Tick off: بکری 1 (24,316 less نقد 5,000) covers the whole advance; a payment of 10,000
        // leaves him owing 5,684; the other 9,316 makes the whole advance uncovered again.
        let events = replay(
            &[rahim(15_000)],
            &[],
            &[
                step(1, -15_000),
                step(2, 19_316),
                step(3, -10_000),
                step(4, -9_316),
            ],
        )
        .unwrap();
        assert_eq!(
            of_entry(&events, 2),
            vec![
                (Covered, 1, 6_000, 15_000),
                (Covered, 2, 4_500, 15_000),
                (Covered, 3, 4_500, 15_000)
            ]
        );
        // C28 of 5,684 over 6,000 / 4,500 / 4,500.
        assert_eq!(
            of_entry(&events, 3),
            vec![
                (Out, 1, 2_274, 5_684),
                (Out, 2, 1_705, 5_684),
                (Out, 3, 1_705, 5_684)
            ]
        );
        // What was not out goes out: each part is whole again.
        assert_eq!(
            of_entry(&events, 4),
            vec![
                (Out, 1, 3_726, 9_316),
                (Out, 2, 2_795, 9_316),
                (Out, 3, 2_795, 9_316)
            ]
        );
        for (id, part) in [(1, 6_000), (2, 4_500), (3, 4_500)] {
            assert_eq!(partner_net(&events, id, None, None).unwrap(), rs(part));
        }
        assert!(events.iter().all(|e| e.allocation_id.is_none()));
    }

    #[test]
    fn a_credit_standing_before_an_advance_makes_no_line() {
        // An unticked credit of 20,000 first: the advance of 15,000 is covered from the start.
        let unticked = replay(
            &[line(10, 2, 15_000, &THREE)],
            &[],
            &[step(1, 20_000), step(2, -15_000)],
        )
        .unwrap();
        assert!(unticked.is_empty());
        // The same credit carried forward onto the advance (PRD FR-15.5) counts from its entry.
        let carried = replay(
            &[line(10, 2, 15_000, &THREE)],
            &[settle(1, 10, 1, 15_000)],
            &[step(1, 20_000), step(2, -15_000)],
        )
        .unwrap();
        assert!(carried.is_empty());
    }

    #[test]
    fn cover_takes_the_oldest_advance_first_and_a_payment_sends_it_out_again() {
        let two = [(1, 5_000), (2, 5_000)];
        let other = [(1, 6_000), (2, 4_000)];
        let events = replay(
            &[line(10, 1, 10_000, &two), line(11, 2, 10_000, &other)],
            &[],
            &[
                step(1, -10_000),
                step(2, -10_000),
                step(3, 10_000),
                step(4, -10_000),
            ],
        )
        .unwrap();
        // The newest stays out 6,000 / 4,000; the oldest comes back 5,000 / 5,000 and goes again.
        assert_eq!(
            of_entry(&events, 3),
            vec![(Covered, 1, 5_000, 10_000), (Covered, 2, 5_000, 10_000)]
        );
        assert_eq!(
            of_entry(&events, 4),
            vec![(Out, 1, 5_000, 10_000), (Out, 2, 5_000, 10_000)]
        );
        assert!(
            events
                .iter()
                .filter(|e| e.entry_id > 2)
                .all(|e| e.line_id == 10)
        );
    }

    #[test]
    fn what_comes_back_is_what_was_out_not_the_whole_settlement() {
        // 4,000 of the 10,000 is covered first; the ticked بکری then settles all 10,000 and only
        // the 6,000 still out comes back.
        let two = [(1, 5_000), (2, 5_000)];
        let events = replay(
            &[line(10, 1, 10_000, &two)],
            &[settle(9, 10, 3, 10_000)],
            &[step(1, -10_000), step(2, 4_000), step(3, 12_000)],
        )
        .unwrap();
        assert_eq!(
            of_entry(&events, 3),
            vec![(Back, 1, 3_000, 6_000), (Back, 2, 3_000, 6_000)]
        );
        assert!(of_entry(&events, 3).len() == 2);
        assert!(
            events
                .iter()
                .filter(|e| told(e) == Back)
                .all(|e| e.allocation_id == Some(9))
        );
    }

    #[test]
    fn a_settlement_whose_credit_is_not_live_is_not_counted() {
        let events = replay(
            &[rahim(15_000)],
            &[settle(1, 10, 99, 15_000)],
            &[step(1, -15_000)],
        )
        .unwrap();
        assert_eq!(
            parts_of(&events, Out),
            vec![(1, 6_000), (2, 4_500), (3, 4_500)]
        );
        assert!(parts_of(&events, Back).is_empty());
    }

    #[test]
    fn an_advance_with_no_shares_has_no_parts_but_takes_its_place() {
        let two = [(1, 5_000), (2, 5_000)];
        let events = replay(
            &[line(10, 1, 5_000, &[]), line(11, 2, 5_000, &two)],
            &[],
            &[step(1, -5_000), step(2, -5_000), step(3, 5_000)],
        )
        .unwrap();
        // The credit covers the older one, which has no parts: nothing comes back to anyone.
        assert!(parts_of(&events, Covered).is_empty());
        assert_eq!(parts_of(&events, Out), vec![(1, 2_500), (2, 2_500)]);
    }

    #[test]
    fn a_later_share_change_does_not_change_the_back_split() {
        let events = replay(
            &[rahim(15_000)],
            &[settle(1, 10, 5, 15_000)],
            &[step(1, -15_000), step(5, 15_000)],
        )
        .unwrap();
        assert_eq!(
            parts_of(&events, Back),
            vec![(1, 6_000), (2, 4_500), (3, 4_500)]
        );
    }

    #[test]
    fn a_partner_with_no_share_has_no_event() {
        let split = [(1, 5_000), (2, 5_000), (3, 0)];
        let events = replay(
            &[line(10, 1, 1_001, &split)],
            &[settle(1, 10, 5, 1_001)],
            &[step(1, -1_001), step(5, 1_001)],
        )
        .unwrap();
        assert!(events.iter().all(|e| e.partner_id != 3));
    }

    #[test]
    fn bad_shares_too_much_settled_and_a_missing_entry_are_errors() {
        let bad = line(10, 1, 15_000, &[(1, 4_000), (2, 3_000)]);
        assert_eq!(
            replay(&[bad], &[], &[step(1, -15_000)]),
            Err(AdvanceError::Money(MoneyError::SharesNot100))
        );
        assert_eq!(
            replay(
                &[rahim(1_000)],
                &[settle(1, 10, 5, 600), settle(2, 10, 6, 600)],
                &[step(1, -1_000), step(5, 600), step(6, 600)],
            ),
            Err(AdvanceError::OverSettled { line_id: 10 })
        );
        assert_eq!(
            replay(&[rahim(1_000)], &[], &[]),
            Err(AdvanceError::NoStep { line_id: 10 })
        );
    }

    /// One thing that happens to a seller, in the random runs.
    #[derive(Debug, Clone)]
    enum Op {
        Advance(i64, bool),
        /// A sale credit; with the tick it settles his open advances, oldest first.
        Credit(i64, bool),
        Payment(i64),
        /// His وصولی: it settles advances, at most what is open.
        Receipt(i64),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (1_i64..20_000, any::<bool>()).prop_map(|(a, s)| Op::Advance(a, s)),
            (1_i64..20_000, any::<bool>()).prop_map(|(a, t)| Op::Credit(a, t)),
            (1_i64..20_000).prop_map(Op::Payment),
            (1_i64..20_000).prop_map(Op::Receipt),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// After every entry: what the partners have out is the uncovered advance of the card,
        /// `min(open advance, what he owes)`, and no part is below 0 or above its whole.
        #[test]
        fn the_parts_out_are_always_the_uncovered_advance(
            ops in proptest::collection::vec(op(), 1..10),
        ) {
            let two_sets = [shares(&THREE), shares(&[(1, 5_000), (2, 5_000), (3, 0)])];
            let mut lines: Vec<AdvanceLineIn> = Vec::new();
            let mut left: Vec<i64> = Vec::new();
            let mut settlements: Vec<SettlementIn> = Vec::new();
            let mut steps: Vec<SellerStepIn> = Vec::new();
            let mut balance = 0_i64;
            for (index, op) in ops.iter().enumerate() {
                let entry = i64::try_from(index).unwrap() + 1;
                // What this entry settles of the open advances, oldest first.
                let mut settle_up_to = |mut amount: i64, left: &mut Vec<i64>| -> i64 {
                    let mut done = 0;
                    for (advance, open) in lines.iter().zip(left.iter_mut()) {
                        let take = amount.min(*open);
                        if take > 0 {
                            *open -= take;
                            amount -= take;
                            done += take;
                            let id = i64::try_from(settlements.len()).unwrap() + 1;
                            settlements.push(settle(id, advance.line_id, entry, take));
                        }
                    }
                    done
                };
                let net = match op {
                    Op::Advance(amount, _) => -amount,
                    Op::Credit(amount, tick) => {
                        if *tick {
                            settle_up_to(*amount, &mut left);
                        }
                        *amount
                    }
                    Op::Payment(amount) => -amount,
                    Op::Receipt(amount) => settle_up_to(*amount, &mut left),
                };
                if let Op::Advance(amount, second) = op {
                    let mut advance = line(entry + 100, entry, *amount, &THREE);
                    advance.shares = two_sets[usize::from(*second)].clone();
                    lines.push(advance);
                    left.push(*amount);
                }
                balance += net;
                steps.push(step(entry, net));

                let events = replay(&lines, &settlements, &steps).unwrap();
                let open: i64 = left.iter().sum();
                let uncovered = open.min((-balance).max(0));
                let mut out = 0;
                for partner in [1, 2, 3] {
                    let net = partner_net(&events, partner, None, None).unwrap();
                    prop_assert!(!net.is_negative());
                    out += net.rupees();
                }
                prop_assert_eq!(out, uncovered);
                // Each part of each advance stays from 0 to its whole.
                for advance in &lines {
                    let whole = split_by_bps(advance.amount, &advance.shares).unwrap();
                    for (partner, part) in whole {
                        let mut running = Paisa::ZERO;
                        for e in events
                            .iter()
                            .filter(|e| e.line_id == advance.line_id && e.partner_id == partner)
                        {
                            running = match e.kind {
                                AdvanceEventKind::Out => running.checked_add(e.part).unwrap(),
                                AdvanceEventKind::Back => running.checked_sub(e.part).unwrap(),
                            };
                            prop_assert!(!running.is_negative() && running <= part);
                        }
                    }
                }
            }
        }
    }
}
