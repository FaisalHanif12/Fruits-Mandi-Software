//! Money of fruit taken (PRD 16.7, FR-16.10, FR-16.11, C1, C27, plan 17d) as pure functions. No
//! database, no clock. The preview and the save use the same functions (Code Standards B5), and
//! every split goes through `split_by_qty` and nothing else (money agent Section 5.5).
//!
//! - [`fruit_value`] (C1): `qty × rate`, or `round(kg × rate per kg)` for an item sold by weight.
//! - [`net_after_deduction`] (PRD FR-16.11 v1.59): fruit from another shop may carry a کل کٹوتی, the
//!   other shop's commission, freight and other charges in one amount. The net, `value - deduction`,
//!   is what the agent is credited and what the partners are charged. 0 deducts nothing.
//! - [`partner_part`]: fruit taken home is worth its whole value to that one partner.
//! - [`guest_parts`] (C27): fruit for the shop's guests is split equally between the partners, never
//!   by the firm's shares. The partners come in the order they were created, and the leftover rupees
//!   all go to the one created first (PRD E68).
//! - [`fruit_entry`]: each partner's 0604 account بنام his part, the source account جمع the value.
//!   No commission, no cash, and no partner share rows (fruit is never a shop expense).
//!
//! Example (PRD E2E-42): 3 پیٹی سیب at 1,000 = 3,000 for the guests of a shop with three partners:
//! 1,000 / 1,000 / 1,000. And 1,000 over three partners: 334 / 333 / 333. Example (PRD FR-16.11 v1.59):
//! 2 پیٹی at 1,400 = 2,800, کل کٹوتی 200: the net is 2,600, and the guests of three partners are
//! charged 868 / 866 / 866 (C27: the leftover 2 rupees go to the partner created first).
//!
//! ```
//! use mandi_domain::core::ids::{AccountId, PartnerId};
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::fruit::{fruit_entry, guest_parts};
//! let partners = [
//!     (PartnerId::new(1), AccountId::new(11)),
//!     (PartnerId::new(2), AccountId::new(12)),
//!     (PartnerId::new(3), AccountId::new(13)),
//! ];
//! let value = Paisa::from_rupees(1_000).unwrap();
//! let parts = guest_parts(value, &partners).unwrap();
//! let rupees: Vec<i64> = parts.parts.iter().map(|p| p.amount.rupees()).collect();
//! assert_eq!(rupees, vec![334, 333, 333]);
//! let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
//! assert_eq!(entry.total(), value);
//! ```

use mandi_core::ids::{AccountId, PartnerId};
use mandi_core::money::{Grams, MoneyError, Paisa, Qty, line_value, split_by_qty, weight_value};
use mandi_core::rules::{Working, WorkingArg, WorkingValue};

use crate::ledger::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};

/// What the entry is worth, with how it was worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitValue {
    /// The value, whole rupees.
    pub value: Paisa,
    /// How it was worked out (CalcInfo).
    pub working: Working,
}

/// One partner's part of the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitPartAmount {
    /// The partner.
    pub partner_id: PartnerId,
    /// His gardens, trips and fruit account (0604).
    pub account_id: AccountId,
    /// His part, whole rupees. His 0604 account is debited this amount.
    pub amount: Paisa,
}

/// The parts of one entry and how they were worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitParts {
    /// One part for each partner, in the order given. The parts add up to the value.
    pub parts: Vec<FruitPartAmount>,
    /// How the guests' parts were worked out (C27). `None` for one partner.
    pub working: Option<Working>,
}

fn money_arg(name: &str, paisa: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa },
    }
}

fn count_arg(name: &str, value: i64) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Count { value },
    }
}

fn working(message_id: &str, args: Vec<WorkingArg>) -> Working {
    Working {
        message_id: message_id.to_owned(),
        args,
        rule_id: None,
    }
}

/// The value of the fruit (PRD C1): `qty × rate` for an item sold by نگ, or `round(kg × rate per kg)`
/// for an item sold by weight (`weight` is given). The rate is the price of one نگ, or of one kg.
///
/// Example (PRD E2E-42): 2 × 580 = 1,160 rupees. 12.345 kg at 100 per kg = 1,234.5 → 1,235.
///
/// # Errors
/// `MoneyError::Overflow`; `MoneyError::NegativeBase` for a negative rate.
pub fn fruit_value(qty: Qty, weight: Option<Grams>, rate: Paisa) -> Result<FruitValue, MoneyError> {
    match weight {
        None => {
            let value = line_value(qty, rate)?;
            Ok(FruitValue {
                value,
                working: working(
                    "working-fruit-value",
                    vec![
                        count_arg("qty", qty.get()),
                        money_arg("rate", rate),
                        money_arg("result", value),
                    ],
                ),
            })
        }
        Some(grams) => {
            let value = weight_value(grams, rate)?;
            Ok(FruitValue {
                value,
                working: working(
                    "working-fruit-value-weight",
                    vec![
                        count_arg("grams", grams.get()),
                        money_arg("rate", rate),
                        money_arg("result", value),
                    ],
                ),
            })
        }
    }
}

/// The net of an entry from another shop, with how it was worked out (PRD FR-16.11 v1.59).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitNet {
    /// `value - deduction`, whole rupees, more than 0.
    pub net: Paisa,
    /// How it was worked out (CalcInfo): message `working-fruit-net`, args `value`, `deduction`,
    /// `result`.
    pub working: Working,
}

/// The most that can be deducted from `value`: one rupee less, so the agent is credited something
/// (PRD FR-16.11 v1.59). `None` when the value is not above one rupee, so nothing can be deducted.
///
/// Example: a value of 2,800 allows a deduction up to 2,799.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn max_deduction(value: Paisa) -> Result<Option<Paisa>, MoneyError> {
    let one = Paisa::from_rupees(1)?;
    let max = value.checked_sub(one)?;
    Ok((max > Paisa::ZERO).then_some(max))
}

/// What the partners are charged and the agent is credited when the other shop takes a کل کٹوتی off
/// the value (PRD FR-16.11 v1.59): `net = value - deduction`, all in whole rupees with checked
/// subtraction. A deduction of 0 gives `Some` with the net equal to the value.
///
/// Returns `None` when the deduction is not less than the value (the agent would be credited nothing
/// or less): the caller refuses it with `ERR_DEDUCTION_NOT_BELOW_VALUE`. The net is therefore always
/// more than 0.
///
/// Example: 2,800 less 200 is 2,600. 2,800 less 2,800 is `None`.
///
/// # Errors
/// `MoneyError::NegativeValue` for a negative deduction or value; `MoneyError::Overflow`.
pub fn net_after_deduction(value: Paisa, deduction: Paisa) -> Result<Option<FruitNet>, MoneyError> {
    if deduction.is_negative() || value.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    if deduction >= value {
        return Ok(None);
    }
    let net = value.checked_sub(deduction)?;
    Ok(Some(FruitNet {
        net,
        working: working(
            "working-fruit-net",
            vec![
                money_arg("value", value),
                money_arg("deduction", deduction),
                money_arg("result", net),
            ],
        ),
    }))
}

/// Fruit taken home by one partner: his part is the whole value (PRD FR-16.10). Nobody else is
/// affected.
#[must_use]
pub fn partner_part(partner: (PartnerId, AccountId), value: Paisa) -> FruitParts {
    FruitParts {
        parts: vec![FruitPartAmount {
            partner_id: partner.0,
            account_id: partner.1,
            amount: value,
        }],
        working: None,
    }
}

/// Fruit for the shop's guests (PRD C27): `split_by_qty(value, [1 for each partner])`. Each part is
/// rounded down and the leftover rupees all go to the partner created first (the lowest id, PRD E68).
/// `partners` are `(partner id, 0604 account)` in the order they were created.
///
/// Example: 1,000 over three partners = 334 / 333 / 333; 1 rupee over three = 1 / 0 / 0;
/// 10 rupees over four = 4 / 2 / 2 / 2.
///
/// # Errors
/// `MoneyError::DivisionByZero` if there is no partner; `MoneyError::Overflow`.
pub fn guest_parts(
    value: Paisa,
    partners: &[(PartnerId, AccountId)],
) -> Result<FruitParts, MoneyError> {
    let weights = partners
        .iter()
        .map(|&(id, _)| Qty::new(1).map(|one| (id, one)))
        .collect::<Result<Vec<_>, _>>()?;
    let split = split_by_qty(value, &weights)?;
    let parts = partners
        .iter()
        .map(|&(partner_id, account_id)| {
            split
                .iter()
                .find(|(id, _)| *id == partner_id)
                .map(|&(_, amount)| FruitPartAmount {
                    partner_id,
                    account_id,
                    amount,
                })
                .ok_or(MoneyError::DuplicateId)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let count = i64::try_from(partners.len()).map_err(|_| MoneyError::Overflow)?;
    let result = parts.first().map_or(Paisa::ZERO, |p| p.amount);
    Ok(FruitParts {
        parts,
        working: Some(working(
            "working-fruit-guests",
            vec![
                money_arg("value", value),
                count_arg("count", count),
                money_arg("result", result),
            ],
        )),
    })
}

/// The journal entry of the entry: each partner's 0604 account بنام his part, the source account
/// جمع the value. A part of 0 makes no line. The entry balances because the parts add up to the value.
///
/// # Errors
/// `LedgerError::Empty` if every part is 0; the other errors of building an entry
/// (`LedgerError::Unbalanced` if the parts do not add up to the value).
pub fn fruit_entry(
    parts: &[FruitPartAmount],
    credit: AccountId,
) -> Result<BalancedEntry, LedgerError> {
    let total = parts
        .iter()
        .try_fold(Paisa::ZERO, |sum, part| sum.checked_add(part.amount))?;
    let mut lines: Vec<LedgerLine> = parts
        .iter()
        .filter(|part| part.amount > Paisa::ZERO)
        .map(|part| LedgerLine::new(part.account_id, Side::Dr, part.amount))
        .collect();
    lines.push(LedgerLine::new(credit, Side::Cr, total));
    build_entry(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn three() -> Vec<(PartnerId, AccountId)> {
        (1..=3)
            .map(|i| (PartnerId::new(i), AccountId::new(10 + i)))
            .collect()
    }

    fn rupees(parts: &FruitParts) -> Vec<i64> {
        parts
            .parts
            .iter()
            .map(|part| part.amount.rupees())
            .collect()
    }

    /// PRD FR-16.11 v1.59: 2 × 1,400 = 2,800, کل کٹوتی 200, net 2,600. The partner's 0604 account is
    /// debited 2,600 and the agent is credited 2,600.
    #[test]
    fn a_deduction_is_subtracted_and_the_partner_and_the_agent_get_the_net() {
        let value = fruit_value(Qty::new(2).unwrap(), None, p(1_400)).unwrap();
        assert_eq!(value.value, p(2_800));
        let net = net_after_deduction(value.value, p(200)).unwrap().unwrap();
        assert_eq!(net.net, p(2_600));
        assert_eq!(net.working.message_id, "working-fruit-net");
        let parts = partner_part((PartnerId::new(1), AccountId::new(11)), net.net);
        assert_eq!(rupees(&parts), vec![2_600]);
        let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
        assert_eq!(entry.total(), p(2_600));
        assert_eq!(entry.lines().len(), 2);
    }

    /// The guests of three partners are charged the net by C27: floors 866 each, and the 2 leftover
    /// rupees all go to the partner created first (E68): 868 / 866 / 866.
    #[test]
    fn guests_share_the_net_by_c27() {
        let net = net_after_deduction(p(2_800), p(200)).unwrap().unwrap().net;
        let parts = guest_parts(net, &three()).unwrap();
        assert_eq!(rupees(&parts), vec![868, 866, 866]);
        let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
        assert_eq!(entry.total(), p(2_600));
        assert_eq!(entry.lines().len(), 4);
    }

    /// A deduction of 0 deducts nothing: the net is the value, so the entry is what it was.
    #[test]
    fn a_deduction_of_zero_posts_exactly_as_before() {
        let net = net_after_deduction(p(2_800), Paisa::ZERO).unwrap().unwrap();
        assert_eq!(net.net, p(2_800));
    }

    /// The deduction must be less than the value; the most is one rupee less.
    #[test]
    fn a_deduction_not_below_the_value_gives_none_and_the_most_is_one_rupee_less() {
        assert!(net_after_deduction(p(2_800), p(2_800)).unwrap().is_none());
        assert!(net_after_deduction(p(2_800), p(3_000)).unwrap().is_none());
        let just_below = net_after_deduction(p(2_800), p(2_799)).unwrap().unwrap();
        assert_eq!(just_below.net, p(1));
        assert_eq!(max_deduction(p(2_800)).unwrap(), Some(p(2_799)));
        // One rupee of value leaves nothing to deduct.
        assert_eq!(max_deduction(p(1)).unwrap(), None);
    }

    /// A negative deduction is refused, never worked out.
    #[test]
    fn a_negative_deduction_is_an_error() {
        assert_eq!(
            net_after_deduction(p(2_800), p(-100)),
            Err(MoneyError::NegativeValue)
        );
    }

    #[test]
    fn e2e_42_home_fruit_is_2_times_580_for_one_partner() {
        let value = fruit_value(Qty::new(2).unwrap(), None, p(580)).unwrap();
        assert_eq!(value.value, p(1_160));
        assert_eq!(value.working.message_id, "working-fruit-value");
        let parts = partner_part((PartnerId::new(1), AccountId::new(11)), value.value);
        assert_eq!(rupees(&parts), vec![1_160]);
        assert!(parts.working.is_none());
        // His 0604 account Dr 1,160, the lot's seller Cr 1,160.
        let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
        assert_eq!(entry.total(), p(1_160));
        assert_eq!(entry.lines().len(), 2);
    }

    #[test]
    fn e2e_42_guest_fruit_3000_over_three_partners_is_1000_each() {
        let value = fruit_value(Qty::new(3).unwrap(), None, p(1_000)).unwrap();
        assert_eq!(value.value, p(3_000));
        let parts = guest_parts(value.value, &three()).unwrap();
        assert_eq!(rupees(&parts), vec![1_000, 1_000, 1_000]);
        let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
        assert_eq!(entry.total(), p(3_000));
        assert_eq!(entry.lines().len(), 4);
    }

    #[test]
    fn c27_the_leftover_goes_to_the_partners_created_first() {
        assert_eq!(
            rupees(&guest_parts(p(1_000), &three()).unwrap()),
            vec![334, 333, 333]
        );
        assert_eq!(rupees(&guest_parts(p(7), &three()).unwrap()), vec![3, 2, 2]);
        assert_eq!(rupees(&guest_parts(p(1), &three()).unwrap()), vec![1, 0, 0]);
        let two = &three()[..2];
        assert_eq!(rupees(&guest_parts(p(1_000), two).unwrap()), vec![500, 500]);
        assert_eq!(rupees(&guest_parts(p(3), two).unwrap()), vec![2, 1]);
        // All the leftover goes to the first created: 10 over four is floor 2 each, leftover 2.
        let four: Vec<(PartnerId, AccountId)> = (1..=4)
            .map(|i| (PartnerId::new(i), AccountId::new(10 + i)))
            .collect();
        assert_eq!(
            rupees(&guest_parts(p(10), &four).unwrap()),
            vec![4, 2, 2, 2]
        );
    }

    #[test]
    fn a_part_of_zero_makes_no_journal_line() {
        let parts = guest_parts(p(1), &three()).unwrap();
        let entry = fruit_entry(&parts.parts, AccountId::new(99)).unwrap();
        // One partner line and the credit.
        assert_eq!(entry.lines().len(), 2);
        assert_eq!(entry.total(), p(1));
    }

    #[test]
    fn an_item_sold_by_weight_is_rounded_once() {
        // 12.345 kg at 100 per kg = 1,234.5 -> 1,235 (PRD C1, E23).
        let value = fruit_value(
            Qty::new(1).unwrap(),
            Some(Grams::new(12_345).unwrap()),
            p(100),
        )
        .unwrap();
        assert_eq!(value.value, p(1_235));
        assert_eq!(value.working.message_id, "working-fruit-value-weight");
    }

    #[test]
    fn no_partner_for_the_guests_is_an_error() {
        assert_eq!(
            guest_parts(p(1_000), &[]).unwrap_err(),
            MoneyError::DivisionByZero
        );
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// Code Standards 9.3, PRD C27: the guests' parts add up to the value, every part is at
        /// least the value divided by the number of partners (rounded down), every partner but the
        /// first created gets exactly that floor (the leftover all goes to the first created, E68),
        /// and the entry balances.
        #[test]
        fn the_guest_parts_add_up_and_the_entry_balances(
            value in 1_i64..5_000_000,
            count in 1_usize..10,
        ) {
            let partners: Vec<(PartnerId, AccountId)> = (0..count)
                .map(|i| {
                    let n = i64::try_from(i).unwrap() + 1;
                    (PartnerId::new(n), AccountId::new(100 + n))
                })
                .collect();
            let parts = guest_parts(p(value), &partners).unwrap();
            let sum = parts
                .parts
                .iter()
                .try_fold(Paisa::ZERO, |s, x| s.checked_add(x.amount))
                .unwrap();
            prop_assert_eq!(sum, p(value));
            let floor = value / i64::try_from(count).unwrap();
            for part in &parts.parts {
                prop_assert!(part.amount.rupees() >= floor);
            }
            for part in parts.parts.iter().skip(1) {
                prop_assert_eq!(part.amount.rupees(), floor);
            }
            let first = parts.parts.first().unwrap().amount.rupees();
            prop_assert_eq!(first, value - floor * (i64::try_from(count).unwrap() - 1));
            let entry = fruit_entry(&parts.parts, AccountId::new(9_999)).unwrap();
            prop_assert_eq!(entry.total(), p(value));
        }

        /// PRD FR-16.11 v1.59: for any value and any deduction below it, the net is a whole rupee of
        /// at least 1, `net + deduction = value`, the guests' parts of the net add up to the net and
        /// the entry balances at the net. A deduction of the value or more has no net.
        #[test]
        fn the_net_is_whole_positive_and_the_entry_balances_at_the_net(
            value in 1_i64..5_000_000,
            cut in 0_i64..5_000_000,
            count in 1_usize..10,
        ) {
            let deduction = cut % value;
            let net = net_after_deduction(p(value), p(deduction)).unwrap().unwrap();
            prop_assert!(net.net >= p(1));
            prop_assert_eq!(net.net.as_paisa() % 100, 0);
            prop_assert_eq!(net.net.checked_add(p(deduction)).unwrap(), p(value));
            let partners: Vec<(PartnerId, AccountId)> = (0..count)
                .map(|i| {
                    let n = i64::try_from(i).unwrap() + 1;
                    (PartnerId::new(n), AccountId::new(100 + n))
                })
                .collect();
            let parts = guest_parts(net.net, &partners).unwrap();
            let entry = fruit_entry(&parts.parts, AccountId::new(9_999)).unwrap();
            prop_assert_eq!(entry.total(), net.net);
            // A deduction of the value or more has no net.
            prop_assert!(net_after_deduction(p(value), p(value + cut)).unwrap().is_none());
            // With no deduction the net is the value: the entry is what it was.
            let same = net_after_deduction(p(value), Paisa::ZERO).unwrap().unwrap();
            prop_assert_eq!(same.net, p(value));
            let max = max_deduction(p(value)).unwrap();
            prop_assert_eq!(max, if value > 1 { Some(p(value - 1)) } else { None });
        }
    }
}
