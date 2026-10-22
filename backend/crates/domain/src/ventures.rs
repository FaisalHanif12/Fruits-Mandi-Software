//! Money of a market trip (PRD 12.10, FR-12.29 to FR-12.33, C21, C23, C24, C25, plan 16b) as pure
//! functions. No database, no clock. The same functions are used by the preview and by the save
//! (Code Standards B5), and every split goes through `split_by_bps` or `split_by_qty` and nothing
//! else (money agent Section 5.5).
//!
//! - [`payment_parts`] (C23): a payment of a trip, split by the trip's shares. A part from the shop's
//!   cash posts his 0604 account بنام; a part from his own pocket posts nothing.
//! - [`payment_entry`]: the journal entry of those parts: every shop part بنام on his 0604 account and
//!   one روکڑ جمع for the shop parts together. `None` when no part is from the shop's cash, because
//!   a payment from pockets only has no journal entry (PRD FR-12.30).
//! - [`credit_parts`] (C25): money that comes to the trip's partners, split by the same shares.
//! - [`receipt_entry`]: روکڑ بنام the money, each partner's 0604 account جمع his part (PRD FR-12.32).
//! - [`spread_cost`] (C24): a trip cost spread over the purchases that still have goods left, by the
//!   نگ bought.
//!
//! Example (PRD E2E-41): a purchase of 96,000 with الف 50% from the shop and ب 50% from his pocket:
//! the parts are 48,000 and 48,000, and the entry is الف's 0604 بنام 48,000 against روکڑ جمع 48,000.
//!
//! ```
//! use mandi_domain::core::ids::{AccountId, PartnerId};
//! use mandi_domain::core::money::{Bps, Paisa};
//! use mandi_domain::core::ventures::MoneySource;
//! use mandi_domain::ventures::{VenturePartner, payment_entry, payment_parts};
//! let partners = [
//!     VenturePartner {
//!         partner_id: PartnerId::new(1),
//!         account_id: AccountId::new(11),
//!         share: Bps::new(5_000).unwrap(),
//!         source: MoneySource::Shop,
//!     },
//!     VenturePartner {
//!         partner_id: PartnerId::new(2),
//!         account_id: AccountId::new(12),
//!         share: Bps::new(5_000).unwrap(),
//!         source: MoneySource::Pocket,
//!     },
//! ];
//! let parts = payment_parts(Paisa::from_rupees(96_000).unwrap(), &partners).unwrap();
//! let rupees: Vec<i64> = parts.iter().map(|p| p.amount.rupees()).collect();
//! assert_eq!(rupees, vec![48_000, 48_000]);
//! let entry = payment_entry(&parts, AccountId::new(1)).unwrap().unwrap();
//! assert_eq!(entry.total().rupees(), 48_000);
//! ```

use mandi_core::ids::{AccountId, PartnerId};
use mandi_core::money::{Bps, MoneyError, Paisa, Qty, split_by_bps, split_by_qty};
use mandi_core::rules::{Working, WorkingArg, WorkingValue};
use mandi_core::ventures::MoneySource;

use crate::ledger::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};

/// One partner of a trip as the money functions need him.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VenturePartner {
    /// The partner. A lower id means created first (PRD C10 tie rule).
    pub partner_id: PartnerId,
    /// His gardens, trips and fruit account (0604).
    pub account_id: AccountId,
    /// His share of the trip.
    pub share: Bps,
    /// Where his part of this payment comes from (ignored for money that comes in).
    pub source: MoneySource,
}

/// One partner's part of a payment (C23).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentPart {
    /// The partner.
    pub partner_id: PartnerId,
    /// His 0604 account.
    pub account_id: AccountId,
    /// His share.
    pub share: Bps,
    /// Where his part comes from.
    pub source: MoneySource,
    /// His part. The parts add up to the payment.
    pub amount: Paisa,
    /// How the part was worked out.
    pub working: Working,
}

/// One partner's part of money that comes to the partners of a trip (C25).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditPart {
    /// The partner.
    pub partner_id: PartnerId,
    /// His 0604 account.
    pub account_id: AccountId,
    /// His share.
    pub share: Bps,
    /// His part, signed like the money. The parts add up to the money.
    pub amount: Paisa,
    /// How the part was worked out.
    pub working: Working,
}

/// One purchase's part of a trip cost (C24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadPart {
    /// The purchase.
    pub purchase_id: i64,
    /// نگ bought: the weight of this purchase.
    pub qty: i64,
    /// Its part. The parts add up to the cost.
    pub amount: Paisa,
    /// How the part was worked out.
    pub working: Working,
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

/// How a stored part of a payment was worked out, built again from the stored amounts for the
/// screens that read a saved entry (the same message as [`payment_parts`] writes).
pub fn payment_working(amount: Paisa, bps: i32, part: Paisa) -> Working {
    working(
        "working-venture-part",
        vec![
            money_arg("amount", amount),
            count_arg("bps", i64::from(bps)),
            money_arg("result", part),
        ],
    )
}

/// How a stored part of money that came in was worked out (the message of [`credit_parts`]).
pub fn credit_working(amount: Paisa, bps: i32, part: Paisa) -> Working {
    working(
        "working-venture-credit",
        vec![
            money_arg("amount", amount),
            count_arg("bps", i64::from(bps)),
            money_arg("result", part),
        ],
    )
}

/// How a trip cost's part of one purchase was worked out (the message of [`spread_cost`]).
pub fn spread_working(cost: Paisa, qty: i64, total_qty: i64, part: Paisa) -> Working {
    working(
        "working-trip-cost-spread",
        vec![
            money_arg("cost", cost),
            count_arg("qty", qty),
            count_arg("total_qty", total_qty),
            money_arg("result", part),
        ],
    )
}

/// The parts of `amount` by the shares of the trip's partners, in the order of `partners` (the
/// caller gives them by partner id). Shares must total exactly 100%: `MoneyError::SharesNot100`
/// otherwise.
fn by_shares(
    amount: Paisa,
    partners: &[VenturePartner],
) -> Result<Vec<(PartnerId, Paisa)>, MoneyError> {
    let shares: Vec<(PartnerId, Bps)> = partners.iter().map(|p| (p.partner_id, p.share)).collect();
    split_by_bps(amount, &shares)
}

fn part_of(parts: &[(PartnerId, Paisa)], partner_id: PartnerId) -> Result<Paisa, MoneyError> {
    parts
        .iter()
        .find(|(id, _)| *id == partner_id)
        .map(|(_, part)| *part)
        .ok_or(MoneyError::DuplicateId)
}

/// C23: a payment of a trip split by the trip's shares, with the source of each part. The rounding
/// is `split_by_bps`: each part is rounded down to whole rupees and the leftover rupees go to the
/// partner with the largest share (a tie: the partner created first).
///
/// Example (E2E-34): 10,400 at 40 / 30 / 30 gives 4,160 / 3,120 / 3,120.
///
/// # Errors
/// `MoneyError::SharesNot100` if the shares do not total 100%; `MoneyError::Overflow`.
pub fn payment_parts(
    amount: Paisa,
    partners: &[VenturePartner],
) -> Result<Vec<PaymentPart>, MoneyError> {
    let split = by_shares(amount, partners)?;
    partners
        .iter()
        .map(|p| {
            let part = part_of(&split, p.partner_id)?;
            Ok(PaymentPart {
                partner_id: p.partner_id,
                account_id: p.account_id,
                share: p.share,
                source: p.source,
                amount: part,
                working: payment_working(amount, p.share.get(), part),
            })
        })
        .collect()
}

/// The part of a payment that leaves روکڑ: the sum of the parts paid from the shop's cash.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn shop_total(parts: &[PaymentPart]) -> Result<Paisa, MoneyError> {
    parts
        .iter()
        .filter(|p| p.source == MoneySource::Shop)
        .try_fold(Paisa::ZERO, |sum, p| sum.checked_add(p.amount))
}

/// The part of a payment that comes from pockets: recorded only.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn pocket_total(parts: &[PaymentPart]) -> Result<Paisa, MoneyError> {
    parts
        .iter()
        .filter(|p| p.source == MoneySource::Pocket)
        .try_fold(Paisa::ZERO, |sum, p| sum.checked_add(p.amount))
}

/// The journal entry of a payment (PRD FR-12.30): each shop part بنام on his 0604 account, and one
/// روکڑ جمع for all the shop parts. A part of 0 makes no line. `None` when there is nothing to post
/// (every part is from a pocket, or the shop parts are all 0).
///
/// # Errors
/// The errors of building an entry (`LedgerError`); it balances by construction.
pub fn payment_entry(
    parts: &[PaymentPart],
    cash: AccountId,
) -> Result<Option<BalancedEntry>, LedgerError> {
    let total = shop_total(parts)?;
    if total == Paisa::ZERO {
        return Ok(None);
    }
    let mut lines: Vec<LedgerLine> = parts
        .iter()
        .filter(|p| p.source == MoneySource::Shop && p.amount > Paisa::ZERO)
        .map(|p| LedgerLine::new(p.account_id, Side::Dr, p.amount))
        .collect();
    lines.push(LedgerLine::new(cash, Side::Cr, total));
    build_entry(lines).map(Some)
}

/// C25: money that comes to the trip's partners, split by the trip's shares. A negative amount is
/// split as positive and the parts are made negative (PRD C11), by `split_by_bps`.
///
/// Example (E2E-34): 8,460 at 40 / 30 / 30 gives 3,384 / 2,538 / 2,538.
///
/// # Errors
/// `MoneyError::SharesNot100`; `MoneyError::Overflow`.
pub fn credit_parts(
    amount: Paisa,
    partners: &[VenturePartner],
) -> Result<Vec<CreditPart>, MoneyError> {
    let split = by_shares(amount, partners)?;
    partners
        .iter()
        .map(|p| {
            let part = part_of(&split, p.partner_id)?;
            Ok(CreditPart {
                partner_id: p.partner_id,
                account_id: p.account_id,
                share: p.share,
                amount: part,
                working: credit_working(amount, p.share.get(), part),
            })
        })
        .collect()
}

/// The journal entry of money that comes in from a sale in another shop (PRD FR-12.32): روکڑ بنام
/// the money, each trip partner's 0604 account جمع his part. A part of 0 makes no line.
///
/// # Errors
/// `LedgerError::Empty` if the money is 0; the other errors of building an entry.
pub fn receipt_entry(
    amount: Paisa,
    credits: &[CreditPart],
    cash: AccountId,
) -> Result<BalancedEntry, LedgerError> {
    let mut lines = vec![LedgerLine::new(cash, Side::Dr, amount)];
    lines.extend(
        credits
            .iter()
            .filter(|c| c.amount > Paisa::ZERO)
            .map(|c| LedgerLine::new(c.account_id, Side::Cr, c.amount)),
    );
    build_entry(lines)
}

/// C24: a trip cost spread over the purchases that still have goods left, by the نگ bought of each
/// (`split_by_qty`): each part is rounded down and the leftover rupees go to the purchase with the
/// most نگ bought (a tie: the earlier purchase, the lower id). `purchases` are `(id, qty bought)`.
///
/// Example: 1,000 over purchases of 30 and 20 نگ gives 600 and 400.
///
/// # Errors
/// `MoneyError::DivisionByZero` if there is no purchase; `MoneyError::Overflow`.
pub fn spread_cost(cost: Paisa, purchases: &[(i64, i64)]) -> Result<Vec<SpreadPart>, MoneyError> {
    let weights = purchases
        .iter()
        .map(|&(id, qty)| Qty::new(qty).map(|q| (id, q)))
        .collect::<Result<Vec<_>, _>>()?;
    let total_qty = weights
        .iter()
        .try_fold(0_i64, |sum, (_, q)| sum.checked_add(q.get()))
        .ok_or(MoneyError::Overflow)?;
    let split = split_by_qty(cost, &weights)?;
    purchases
        .iter()
        .map(|&(id, qty)| {
            let part = split
                .iter()
                .find(|(pid, _)| *pid == id)
                .map(|(_, part)| *part)
                .ok_or(MoneyError::DuplicateId)?;
            Ok(SpreadPart {
                purchase_id: id,
                qty,
                amount: part,
                working: spread_working(cost, qty, total_qty, part),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn partner(id: i64, bps: i32, source: MoneySource) -> VenturePartner {
        VenturePartner {
            partner_id: PartnerId::new(id),
            account_id: AccountId::new(100 + id),
            share: Bps::new(bps).unwrap(),
            source,
        }
    }

    fn trip(source: [MoneySource; 3]) -> Vec<VenturePartner> {
        vec![
            partner(1, 4_000, source[0]),
            partner(2, 3_000, source[1]),
            partner(3, 3_000, source[2]),
        ]
    }

    fn rupees(parts: &[PaymentPart]) -> Vec<i64> {
        parts.iter().map(|part| part.amount.rupees()).collect()
    }

    #[test]
    fn e2e_34_the_purchase_is_split_4160_3120_3120_and_posts_10400() {
        // PRD E2E-34: 20 x 500 + 400 freight = 10,400, all three from the shop's cash.
        let partners = trip([MoneySource::Shop; 3]);
        let parts = payment_parts(p(10_400), &partners).unwrap();
        assert_eq!(rupees(&parts), vec![4_160, 3_120, 3_120]);
        assert_eq!(shop_total(&parts).unwrap(), p(10_400));
        assert_eq!(pocket_total(&parts).unwrap(), Paisa::ZERO);
        let entry = payment_entry(&parts, AccountId::new(1)).unwrap().unwrap();
        assert_eq!(entry.total(), p(10_400));
        let debits: Vec<i64> = entry
            .lines()
            .iter()
            .filter(|l| l.side == Side::Dr)
            .map(|l| l.amount.rupees())
            .collect();
        assert_eq!(debits, vec![4_160, 3_120, 3_120]);
        let cash_lines: Vec<_> = entry
            .lines()
            .iter()
            .filter(|l| l.account_id == AccountId::new(1))
            .collect();
        assert_eq!(cash_lines.len(), 1);
        assert_eq!(cash_lines[0].side, Side::Cr);
    }

    #[test]
    fn e2e_41_a_pocket_part_posts_nothing() {
        // PRD E2E-41: 96,000 at 50 / 50, الف from the shop and ب from his pocket.
        let partners = vec![
            partner(1, 5_000, MoneySource::Shop),
            partner(2, 5_000, MoneySource::Pocket),
        ];
        let purchase = payment_parts(p(96_000), &partners).unwrap();
        assert_eq!(rupees(&purchase), vec![48_000, 48_000]);
        let entry = payment_entry(&purchase, AccountId::new(1))
            .unwrap()
            .unwrap();
        assert_eq!(entry.total(), p(48_000));
        assert_eq!(entry.lines().len(), 2);
        // The truck: 4,000 over the one purchase of 80 نگ, split 2,000 / 2,000.
        let truck = payment_parts(p(4_000), &partners).unwrap();
        assert_eq!(rupees(&truck), vec![2_000, 2_000]);
        assert_eq!(shop_total(&truck).unwrap(), p(2_000));
        assert_eq!(pocket_total(&truck).unwrap(), p(2_000));
        // Cash 50,000 - 48,000 - 2,000 = 0 (the shop parts).
        assert_eq!(
            p(50_000)
                .checked_sub(shop_total(&purchase).unwrap())
                .unwrap()
                .checked_sub(shop_total(&truck).unwrap())
                .unwrap(),
            Paisa::ZERO
        );
    }

    #[test]
    fn a_payment_from_pockets_only_has_no_entry() {
        let partners = trip([MoneySource::Pocket; 3]);
        let parts = payment_parts(p(10_400), &partners).unwrap();
        assert_eq!(rupees(&parts), vec![4_160, 3_120, 3_120]);
        assert_eq!(payment_entry(&parts, AccountId::new(1)).unwrap(), None);
    }

    #[test]
    fn a_shop_part_of_zero_makes_no_line_and_the_entry_balances() {
        // 1 rupee at 40 / 30 / 30: the leftover goes to the largest share, the others are 0.
        let partners = trip([MoneySource::Shop; 3]);
        let parts = payment_parts(p(1), &partners).unwrap();
        assert_eq!(rupees(&parts), vec![1, 0, 0]);
        let entry = payment_entry(&parts, AccountId::new(1)).unwrap().unwrap();
        assert_eq!(entry.lines().len(), 2);
        assert_eq!(entry.total(), p(1));
        // Only the zero part is from the shop: nothing to post.
        let zero_only = vec![
            partner(1, 4_000, MoneySource::Pocket),
            partner(2, 3_000, MoneySource::Shop),
            partner(3, 3_000, MoneySource::Shop),
        ];
        let parts = payment_parts(p(1), &zero_only).unwrap();
        assert_eq!(rupees(&parts), vec![1, 0, 0]);
        assert_eq!(payment_entry(&parts, AccountId::new(1)).unwrap(), None);
    }

    #[test]
    fn shares_that_do_not_total_100_are_refused() {
        let partners = vec![
            partner(1, 4_000, MoneySource::Shop),
            partner(2, 3_000, MoneySource::Shop),
        ];
        assert_eq!(
            payment_parts(p(100), &partners),
            Err(MoneyError::SharesNot100)
        );
    }

    #[test]
    fn e2e_34_the_seller_credit_goes_3384_2538_2538() {
        // PRD C25: 9,000 - 540 = 8,460 at 40 / 30 / 30.
        let partners = trip([MoneySource::Shop; 3]);
        let credits = credit_parts(p(8_460), &partners).unwrap();
        let amounts: Vec<i64> = credits.iter().map(|c| c.amount.rupees()).collect();
        assert_eq!(amounts, vec![3_384, 2_538, 2_538]);
        let entry = receipt_entry(p(8_460), &credits, AccountId::new(1)).unwrap();
        assert_eq!(entry.total(), p(8_460));
        assert_eq!(entry.lines().len(), 4);
    }

    #[test]
    fn e2e_41_the_trips_money_goes_52640_52640() {
        // PRD E2E-41: the seller credit 105,280 to الف and ب at 50 / 50.
        let partners = vec![
            partner(1, 5_000, MoneySource::Shop),
            partner(2, 5_000, MoneySource::Pocket),
        ];
        let credits = credit_parts(p(105_280), &partners).unwrap();
        let amounts: Vec<i64> = credits.iter().map(|c| c.amount.rupees()).collect();
        assert_eq!(amounts, vec![52_640, 52_640]);
    }

    #[test]
    fn a_negative_credit_is_split_as_positive_and_made_negative() {
        let partners = trip([MoneySource::Shop; 3]);
        let credits = credit_parts(p(-1_001), &partners).unwrap();
        let amounts: Vec<i64> = credits.iter().map(|c| c.amount.rupees()).collect();
        // 1,001 at 40 / 30 / 30: 400 + 1 leftover, 300, 300.
        assert_eq!(amounts, vec![-401, -300, -300]);
        let total = credits
            .iter()
            .try_fold(Paisa::ZERO, |s, c| s.checked_add(c.amount))
            .unwrap();
        assert_eq!(total, p(-1_001));
    }

    #[test]
    fn c24_a_trip_cost_is_spread_by_the_nag_bought() {
        // PRD C24: 1,000 over purchases of 30 and 20 نگ is 600 and 400.
        let parts = spread_cost(p(1_000), &[(7, 30), (9, 20)]).unwrap();
        let amounts: Vec<i64> = parts.iter().map(|s| s.amount.rupees()).collect();
        assert_eq!(amounts, vec![600, 400]);
        // E2E-41: 4,000 over the one purchase of 80 نگ is all 4,000.
        let one = spread_cost(p(4_000), &[(1, 80)]).unwrap();
        assert_eq!(one[0].amount, p(4_000));
        // 1,000 over three purchases of 1 نگ: the leftover goes to the lowest id (334 / 333 / 333).
        let three = spread_cost(p(1_000), &[(1, 1), (2, 1), (3, 1)]).unwrap();
        let amounts: Vec<i64> = three.iter().map(|s| s.amount.rupees()).collect();
        assert_eq!(amounts, vec![334, 333, 333]);
    }

    #[test]
    fn a_trip_cost_with_no_purchase_to_carry_it_is_an_error() {
        assert!(spread_cost(p(1_000), &[]).is_err());
    }

    use proptest::prelude::*;

    /// Shares that total exactly 10,000 bps, from a list of weights.
    fn shares_of(weights: &[u32]) -> Vec<i32> {
        let total: u32 = weights.iter().sum();
        let mut out: Vec<i32> = weights
            .iter()
            .map(|w| i32::try_from(u64::from(*w) * 10_000 / u64::from(total)).unwrap())
            .collect();
        let used: i32 = out.iter().sum();
        out[0] += 10_000 - used;
        out
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// Code Standards 9.3, PRD C23: the parts of a payment add up to the payment, the shop parts
        /// plus the pocket parts are the payment, and the entry of the shop parts always balances.
        #[test]
        fn the_parts_of_a_payment_add_up_and_the_entry_balances(
            amount in 0_i64..5_000_000,
            weights in proptest::collection::vec(1_u32..1_000, 1..8),
            sources in proptest::collection::vec(any::<bool>(), 8),
        ) {
            let shares = shares_of(&weights);
            let partners: Vec<VenturePartner> = shares
                .iter()
                .enumerate()
                .map(|(i, bps)| {
                    partner(
                        i64::try_from(i).unwrap() + 1,
                        *bps,
                        if sources[i] { MoneySource::Shop } else { MoneySource::Pocket },
                    )
                })
                .filter(|p| p.share.get() > 0)
                .collect();
            let total_bps: i32 = partners.iter().map(|p| p.share.get()).sum();
            prop_assume!(total_bps == 10_000);
            let parts = payment_parts(p(amount), &partners).unwrap();
            let sum = parts.iter().try_fold(Paisa::ZERO, |s, x| s.checked_add(x.amount)).unwrap();
            prop_assert_eq!(sum, p(amount));
            let shop = shop_total(&parts).unwrap();
            let pocket = pocket_total(&parts).unwrap();
            prop_assert_eq!(shop.checked_add(pocket).unwrap(), p(amount));
            prop_assert!(parts.iter().all(|x| x.amount >= Paisa::ZERO));
            match payment_entry(&parts, AccountId::new(1)).unwrap() {
                Some(entry) => prop_assert_eq!(entry.total(), shop),
                None => prop_assert_eq!(shop, Paisa::ZERO),
            }
        }

        /// PRD C25: money that comes in, split by the same shares, adds up to the money, and the
        /// entry balances.
        #[test]
        fn the_credits_add_up_and_the_receipt_balances(
            amount in 1_i64..5_000_000,
            weights in proptest::collection::vec(1_u32..1_000, 1..8),
        ) {
            let shares = shares_of(&weights);
            let partners: Vec<VenturePartner> = shares
                .iter()
                .enumerate()
                .map(|(i, bps)| partner(i64::try_from(i).unwrap() + 1, *bps, MoneySource::Shop))
                .filter(|p| p.share.get() > 0)
                .collect();
            let total_bps: i32 = partners.iter().map(|p| p.share.get()).sum();
            prop_assume!(total_bps == 10_000);
            let credits = credit_parts(p(amount), &partners).unwrap();
            let sum = credits.iter().try_fold(Paisa::ZERO, |s, x| s.checked_add(x.amount)).unwrap();
            prop_assert_eq!(sum, p(amount));
            let entry = receipt_entry(p(amount), &credits, AccountId::new(1)).unwrap();
            prop_assert_eq!(entry.total(), p(amount));
        }

        /// PRD C24: a trip cost spread over purchases adds up to the cost, and no part is negative.
        #[test]
        fn a_spread_cost_adds_up(
            cost in 0_i64..5_000_000,
            qtys in proptest::collection::vec(1_i64..500, 1..10),
        ) {
            let purchases: Vec<(i64, i64)> = qtys
                .iter()
                .enumerate()
                .map(|(i, q)| (i64::try_from(i).unwrap() + 1, *q))
                .collect();
            let parts = spread_cost(p(cost), &purchases).unwrap();
            let sum = parts.iter().try_fold(Paisa::ZERO, |s, x| s.checked_add(x.amount)).unwrap();
            prop_assert_eq!(sum, p(cost));
            prop_assert!(parts.iter().all(|x| x.amount >= Paisa::ZERO));
        }
    }
}
