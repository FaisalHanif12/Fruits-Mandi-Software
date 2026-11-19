//! The mirror lines of a linked khata or loan (plan 76 points 5 and 10, PRD FR-16.4, FR-16.6, FR-17.9
//! v1.41; Umar 14-11-2026, U1). Built once, here, for every posting.
//!
//! Money that reaches a partner through a linked account must lower or raise his کل رقم and keep the
//! books balanced and FR-17.9 true. So every line on a linked account has a mirror pair added to its
//! entry, after the normal lines:
//!
//! * the account بنام X: the partner accounts (0605) بنام their parts, the contra account جمع X;
//! * the account جمع X: the contra account بنام X, the partner accounts جمع their parts.
//!
//! The parts of a partner link are one (the whole X). The parts of a shop link are
//! `split_by_bps(X, the shares of the posting day)`, whole rupees, the leftover to the largest share
//! (the partner created first on a tie), a part of 0 getting no line. Each line is split on its own,
//! and the parts always add back to X, so the entry still balances. A reversal is NOT mirrored: it is
//! the exact swap of the stored lines, mirror lines included.
//!
//! Example (PRD plan 76): خاندانی کھاتہ linked to the shop, shares 40 / 30 / 30, paid out 10,000: the
//! khata بنام 10,000 and روکڑ جمع 10,000 as before, then الف 4,000, ب 3,000 and ج 3,000 بنام (their
//! 0605 accounts) and the contra account جمع 10,000. Both sides are 20,000.

use std::collections::HashMap;

use mandi_core::ids::{AccountId, PartnerId};
use mandi_core::money::{Bps, MoneyError, Paisa, split_by_bps};

use super::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};

/// What a linked account passes on to the partners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirror {
    /// Split between the active partners by their shares of the posting day.
    Shop,
    /// The whole of it is this partner's.
    Partner(PartnerId),
}

/// Why a mirror could not be built.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MirrorError {
    /// The entry itself could not be built again.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// A split or a sum failed (the shares do not total 100%, or an overflow).
    #[error(transparent)]
    Money(#[from] MoneyError),
    /// A partner of the link has no 0605 account.
    #[error("partner {0} has no account for linked money")]
    NoPartnerAccount(i64),
}

/// What the mirror needs, all read by the caller.
pub struct MirrorInputs<'a> {
    /// The linked accounts of the entry and what each passes on. An account that is not in this map
    /// is not mirrored.
    pub links: &'a HashMap<AccountId, Mirror>,
    /// The contra account (03070001).
    pub offset: AccountId,
    /// The 0605 account of each partner.
    pub partner_accounts: &'a HashMap<PartnerId, AccountId>,
    /// The shares of the posting day, for a shop link.
    pub shares: &'a [(PartnerId, Bps)],
}

/// Adds the mirror pair of every line on a linked account. An entry with no linked line comes back
/// as it was.
///
/// # Errors
/// [`MirrorError`].
pub fn mirror_linked(
    entry: &BalancedEntry,
    inputs: &MirrorInputs<'_>,
) -> Result<BalancedEntry, MirrorError> {
    let mut lines: Vec<LedgerLine> = entry.lines().to_vec();
    let mut changed = false;
    for line in entry.lines() {
        let Some(mirror) = inputs.links.get(&line.account_id) else {
            continue;
        };
        changed = true;
        let parts: Vec<(PartnerId, Paisa)> = match mirror {
            Mirror::Partner(partner) => vec![(*partner, line.amount)],
            Mirror::Shop => split_by_bps(line.amount, inputs.shares)?,
        };
        let partner_side = line.side;
        let offset_side = match line.side {
            Side::Dr => Side::Cr,
            Side::Cr => Side::Dr,
        };
        // The partners' parts first, then the contra: the order the plan writes them in.
        for (partner, part) in parts {
            if part == Paisa::ZERO {
                continue;
            }
            let account = inputs
                .partner_accounts
                .get(&partner)
                .copied()
                .ok_or(MirrorError::NoPartnerAccount(partner.get()))?;
            lines.push(LedgerLine::new(account, partner_side, part));
        }
        lines.push(LedgerLine::new(inputs.offset, offset_side, line.amount));
    }
    if !changed {
        return Ok(entry.clone());
    }
    Ok(build_entry(lines)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn shares() -> Vec<(PartnerId, Bps)> {
        vec![
            (PartnerId::new(1), Bps::new(4_000).unwrap()),
            (PartnerId::new(2), Bps::new(3_000).unwrap()),
            (PartnerId::new(3), Bps::new(3_000).unwrap()),
        ]
    }

    fn accounts() -> HashMap<PartnerId, AccountId> {
        (1..=3)
            .map(|i| (PartnerId::new(i), AccountId::new(100 + i)))
            .collect()
    }

    fn lines_of(entry: &BalancedEntry) -> Vec<(i64, Side, i64)> {
        entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side, l.amount.as_paisa() / 100))
            .collect()
    }

    #[test]
    fn paid_out_to_a_shop_linked_khata_splits_by_the_shares() {
        // The khata (7) paid out 10,000 from cash (1).
        let entry =
            crate::khata::khata_out_entry(AccountId::new(7), AccountId::new(1), p(10_000)).unwrap();
        let links = HashMap::from([(AccountId::new(7), Mirror::Shop)]);
        let partner_accounts = accounts();
        let shares = shares();
        let mirrored = mirror_linked(
            &entry,
            &MirrorInputs {
                links: &links,
                offset: AccountId::new(9),
                partner_accounts: &partner_accounts,
                shares: &shares,
            },
        )
        .unwrap();
        assert_eq!(
            lines_of(&mirrored),
            vec![
                (7, Side::Dr, 10_000),
                (1, Side::Cr, 10_000),
                (101, Side::Dr, 4_000),
                (102, Side::Dr, 3_000),
                (103, Side::Dr, 3_000),
                (9, Side::Cr, 10_000),
            ]
        );
        assert_eq!(mirrored.total(), p(20_000));
    }

    #[test]
    fn money_back_mirrors_the_other_way() {
        let entry =
            crate::khata::khata_in_entry(AccountId::new(1), AccountId::new(7), p(4_000)).unwrap();
        let links = HashMap::from([(AccountId::new(7), Mirror::Shop)]);
        let (partner_accounts, shares) = (accounts(), shares());
        let mirrored = mirror_linked(
            &entry,
            &MirrorInputs {
                links: &links,
                offset: AccountId::new(9),
                partner_accounts: &partner_accounts,
                shares: &shares,
            },
        )
        .unwrap();
        assert_eq!(
            lines_of(&mirrored),
            vec![
                (1, Side::Dr, 4_000),
                (7, Side::Cr, 4_000),
                (101, Side::Cr, 1_600),
                (102, Side::Cr, 1_200),
                (103, Side::Cr, 1_200),
                (9, Side::Dr, 4_000),
            ]
        );
    }

    #[test]
    fn a_leftover_rupee_goes_to_the_largest_share_and_a_partner_link_is_one_part() {
        // E2E-20: 3,384 -> 1,354 / 1,015 / 1,015.
        let entry =
            crate::khata::khata_out_entry(AccountId::new(7), AccountId::new(1), p(3_384)).unwrap();
        let (partner_accounts, shares) = (accounts(), shares());
        let links = HashMap::from([(AccountId::new(7), Mirror::Shop)]);
        let mirrored = mirror_linked(
            &entry,
            &MirrorInputs {
                links: &links,
                offset: AccountId::new(9),
                partner_accounts: &partner_accounts,
                shares: &shares,
            },
        )
        .unwrap();
        let parts: Vec<i64> = lines_of(&mirrored)[2..5].iter().map(|l| l.2).collect();
        assert_eq!(parts, vec![1_354, 1_015, 1_015]);
        // A partner link: the whole 10,000 on one partner.
        let entry =
            crate::khata::khata_out_entry(AccountId::new(7), AccountId::new(1), p(10_000)).unwrap();
        let links = HashMap::from([(AccountId::new(7), Mirror::Partner(PartnerId::new(1)))]);
        let mirrored = mirror_linked(
            &entry,
            &MirrorInputs {
                links: &links,
                offset: AccountId::new(9),
                partner_accounts: &partner_accounts,
                shares: &[],
            },
        )
        .unwrap();
        assert_eq!(
            lines_of(&mirrored)[2..],
            [(101, Side::Dr, 10_000), (9, Side::Cr, 10_000)]
        );
    }

    #[test]
    fn an_entry_with_no_linked_line_is_left_as_it_is() {
        let entry =
            crate::khata::khata_out_entry(AccountId::new(7), AccountId::new(1), p(500)).unwrap();
        let (links, partner_accounts) = (HashMap::new(), accounts());
        let same = mirror_linked(
            &entry,
            &MirrorInputs {
                links: &links,
                offset: AccountId::new(9),
                partner_accounts: &partner_accounts,
                shares: &[],
            },
        )
        .unwrap();
        assert_eq!(same, entry);
    }
    use proptest::prelude::*;

    /// Shares of 2 to 5 partners that total 10,000 bps: each takes a cut of what is left.
    fn random_shares(cuts: &[u32]) -> Vec<(PartnerId, Bps)> {
        let mut left: i32 = 10_000;
        let mut out = Vec::new();
        for (i, cut) in cuts.iter().enumerate() {
            let last = i + 1 == cuts.len();
            let take = if last {
                left
            } else {
                i32::try_from(*cut).unwrap_or(0).min(left)
            };
            out.push((
                PartnerId::new(i64::try_from(i + 1).unwrap_or(1)),
                Bps::new(take).unwrap(),
            ));
            left -= take;
        }
        out
    }

    proptest! {
        // 10,000 cases (Code Standards 9.3): for any amounts, any shares totalling 100% and either
        // kind of link, every mirror pair balances, the partners' parts add back to the line, the
        // contra is the whole line, and the normal lines are left as they were (plan 76 point 5).
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn the_mirror_always_balances_and_the_parts_add_back(
            amounts in proptest::collection::vec(1_i64..=10_000_000, 1..5),
            cuts in proptest::collection::vec(0_u32..=10_000, 2..6),
            shop in any::<bool>(),
            out in any::<bool>(),
        ) {
            let khata = AccountId::new(7);
            let cash = AccountId::new(1);
            let mut lines = Vec::new();
            for rupees in &amounts {
                let amount = p(*rupees);
                if out {
                    lines.push(LedgerLine::new(khata, Side::Dr, amount));
                    lines.push(LedgerLine::new(cash, Side::Cr, amount));
                } else {
                    lines.push(LedgerLine::new(cash, Side::Dr, amount));
                    lines.push(LedgerLine::new(khata, Side::Cr, amount));
                }
            }
            let entry = build_entry(lines).unwrap();
            let shares = random_shares(&cuts);
            let partner_accounts: HashMap<PartnerId, AccountId> = shares
                .iter()
                .map(|(id, _)| (*id, AccountId::new(100 + id.get())))
                .collect();
            let mirror = if shop {
                Mirror::Shop
            } else {
                Mirror::Partner(shares[0].0)
            };
            let links = HashMap::from([(khata, mirror)]);
            let mirrored = mirror_linked(
                &entry,
                &MirrorInputs {
                    links: &links,
                    offset: AccountId::new(9),
                    partner_accounts: &partner_accounts,
                    shares: &shares,
                },
            )
            .unwrap();
            // The entry still balances (build_entry checked it) and the normal lines are first.
            prop_assert_eq!(&mirrored.lines()[..entry.lines().len()], entry.lines());
            let total: i64 = amounts.iter().sum();
            let on = |account: i64, side: Side| -> i64 {
                mirrored
                    .lines()
                    .iter()
                    .filter(|l| l.account_id.get() == account && l.side == side)
                    .map(|l| l.amount.as_paisa() / 100)
                    .sum()
            };
            let (partner_side, contra_side) = if out { (Side::Dr, Side::Cr) } else { (Side::Cr, Side::Dr) };
            // The contra is the whole of what the khata moved.
            prop_assert_eq!(on(9, contra_side), total);
            // The partners' parts add back to it exactly, never a rupee more or less.
            let parts: i64 = partner_accounts.values().map(|a| on(a.get(), partner_side)).sum();
            prop_assert_eq!(parts, total);
            // Nothing is posted on the other side of a partner account.
            let wrong_side = if out { Side::Cr } else { Side::Dr };
            prop_assert!(partner_accounts.values().all(|a| on(a.get(), wrong_side) == 0));
            if !shop {
                prop_assert_eq!(on(100 + shares[0].0.get(), partner_side), total);
            }
        }
    }
}
