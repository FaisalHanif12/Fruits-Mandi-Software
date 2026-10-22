//! Worked numbers of PRD 4.4, 4.5 and the money agent's cases for the journal of a بکری.

use super::*;
use crate::ledger::Side::{Cr, Dr};

fn p(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

const CASH: i64 = 1;
const SELLER_COMM: i64 = 2;
const BUYER_COMM: i64 = 3;
const LAGA: i64 = 4;
const PHONE: i64 = 5;
const RATE_DIFF: i64 = 6;
const LABOUR: i64 = 7;
const FREIGHT: i64 = 8;
const STORE_RENT: i64 = 9;
const MUNSHIANA: i64 = 10;
const RAHIM: i64 = 20;
const KARIM: i64 = 21;
const AKRAM: i64 = 30;
const ARSHAD: i64 = 31;
const PUKHTA_BUYER: i64 = 32;

fn accounts() -> SystemAccounts {
    let id = AccountId::new;
    SystemAccounts {
        cash: id(CASH),
        seller_commission: id(SELLER_COMM),
        buyer_commission: id(BUYER_COMM),
        laga: id(LAGA),
        phone: id(PHONE),
        rate_diff: id(RATE_DIFF),
        labour: id(LABOUR),
        freight: id(FREIGHT),
        store_rent: id(STORE_RENT),
        munshiana: id(MUNSHIANA),
    }
}

/// Σ on one side of one account.
fn side_sum(entry: &BalancedEntry, account: i64, side: Side) -> i64 {
    entry
        .lines()
        .iter()
        .filter(|l| l.account_id == AccountId::new(account) && l.side == side)
        .map(|l| l.amount.rupees())
        .sum()
}

#[allow(clippy::too_many_arguments)]
fn sale(
    buyer: i64,
    total: i64,
    payment: PaymentType,
    seller_comm: i64,
    buyer_comm: i64,
    laga: i64,
    rate_diff: i64,
) -> JournalSale {
    JournalSale {
        buyer_account_id: AccountId::new(buyer),
        person_key: None,
        buyer_total: p(total),
        payment_type: payment,
        seller_comm: p(seller_comm),
        buyer_comm: p(buyer_comm),
        laga: p(laga),
        rate_diff: p(rate_diff),
    }
}

fn event1_lot() -> JournalLot {
    JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(24_316),
        cash_paid: p(5_000),
        freight_deducted: p(3_000),
        labour: p(250),
        munshiana: p(50),
        store_rent: Paisa::ZERO,
        phone: p(20),
        sales: vec![
            sale(AKRAM, 12_880, PaymentType::Credit, 720, 720, 160, 0),
            sale(ARSHAD, 18_684, PaymentType::Credit, 1_044, 1_044, 240, 0),
        ],
    }
}

#[test]
fn event_1_whole_entry_is_36_564() {
    // PRD 4.4: sale part 31,564 = 31,564 and the cash part 5,000 = 5,000.
    let entry = build_bikri_entry(
        &JournalSource {
            lots: vec![event1_lot()],
        },
        &accounts(),
    )
    .unwrap();
    assert_eq!(entry.total(), p(36_564));
    assert_eq!(side_sum(&entry, AKRAM, Dr), 12_880);
    assert_eq!(side_sum(&entry, ARSHAD, Dr), 18_684);
    assert_eq!(side_sum(&entry, RAHIM, Cr), 24_316);
    assert_eq!(side_sum(&entry, SELLER_COMM, Cr), 1_764);
    assert_eq!(side_sum(&entry, BUYER_COMM, Cr), 1_764);
    assert_eq!(side_sum(&entry, LAGA, Cr), 400);
    assert_eq!(side_sum(&entry, PHONE, Cr), 20);
    assert_eq!(side_sum(&entry, LABOUR, Cr), 250);
    assert_eq!(side_sum(&entry, FREIGHT, Cr), 3_000);
    assert_eq!(side_sum(&entry, MUNSHIANA, Cr), 50);
    // The cash part: نقد to the seller.
    assert_eq!(side_sum(&entry, RAHIM, Dr), 5_000);
    assert_eq!(side_sum(&entry, CASH, Cr), 5_000);
    assert_eq!(side_sum(&entry, CASH, Dr), 0);
    // 2 buyers + 1 seller + 4 income + 3 payables in the sale part, 2 lines in the cash part.
    // No line for an account with nothing to post (store rent is 0).
    assert_eq!(entry.lines().len(), 2 + 1 + 4 + 3 + 2);
}

#[test]
fn event_2_cash_line_of_a_pukhta_buyer_is_25_600() {
    // PRD 4.5: Karim's 10 apples; the one-time buyer pays 12,800 in cash.
    let lot = JournalLot {
        seller_account_id: AccountId::new(KARIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(10_160),
        cash_paid: Paisa::ZERO,
        freight_deducted: p(1_000),
        labour: p(50),
        munshiana: p(50),
        store_rent: Paisa::ZERO,
        phone: p(20),
        sales: vec![sale(
            PUKHTA_BUYER,
            12_800,
            PaymentType::Cash,
            720,
            720,
            80,
            0,
        )],
    };
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert_eq!(entry.total(), p(25_600));
    // The buyer is debited in the sale part and credited in the cash part, so he nets to 0.
    assert_eq!(side_sum(&entry, PUKHTA_BUYER, Dr), 12_800);
    assert_eq!(side_sum(&entry, PUKHTA_BUYER, Cr), 12_800);
    assert_eq!(side_sum(&entry, CASH, Dr), 12_800);
    // Cash change of the entry is +12,800 (nothing was paid to the seller).
    let change = crate::cash::net_change(&entry, AccountId::new(CASH)).unwrap();
    assert_eq!(change, p(12_800));
}

#[test]
fn event_1_cash_change_is_minus_5_000() {
    let entry = build_bikri_entry(
        &JournalSource {
            lots: vec![event1_lot()],
        },
        &accounts(),
    )
    .unwrap();
    assert_eq!(
        crate::cash::net_change(&entry, AccountId::new(CASH)).unwrap(),
        p(-5_000)
    );
}

#[test]
fn a_negative_seller_credit_is_a_debit_on_the_seller() {
    // Money agent case 5 (E28): 1 received, 1 x 600, freight 3,000 paid by the firm, نقد 5,000.
    // Seller commission 36, buyer commission 36, لاگا 8, buyer total 644. Seller credit
    // 600 - 36 - 3,000 = -2,436.
    let lot = JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(-2_436),
        cash_paid: p(5_000),
        freight_deducted: p(3_000),
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![sale(AKRAM, 644, PaymentType::Credit, 36, 36, 8, 0)],
    };
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    // Sale part 3,080 = 3,080 (644 + 2,436 | 36 + 36 + 8 + 3,000), whole entry 8,080.
    assert_eq!(entry.total(), p(8_080));
    assert_eq!(side_sum(&entry, RAHIM, Dr), 2_436 + 5_000);
    assert_eq!(side_sum(&entry, RAHIM, Cr), 0);
}

#[test]
fn a_seller_credit_of_exactly_zero_makes_no_seller_line() {
    // Money agent case 6: 1 x 1,000, freight 940. Seller commission 60, seller credit 0.
    // Debit 1,068 = credit 60 + 60 + 8 + 940.
    let lot = JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: Paisa::ZERO,
        cash_paid: Paisa::ZERO,
        freight_deducted: p(940),
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![sale(AKRAM, 1_068, PaymentType::Credit, 60, 60, 8, 0)],
    };
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert_eq!(entry.total(), p(1_068));
    assert!(
        entry
            .lines()
            .iter()
            .all(|l| l.account_id != AccountId::new(RAHIM))
    );
}

#[test]
fn a_negative_rate_difference_is_a_debit_on_rate_diff() {
    // Money agent case 7: Akram's rate 590 against 600. Value 11,800, buyer commission 708,
    // buyer total 12,668, rate difference -200. No lot charges. Arshad as in Event 1.
    let lot = JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        // 12,000 + 17,400 - 1,764 = 27,636
        seller_credit: p(27_636),
        cash_paid: Paisa::ZERO,
        freight_deducted: Paisa::ZERO,
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![
            sale(AKRAM, 12_668, PaymentType::Credit, 720, 708, 160, -200),
            sale(ARSHAD, 18_684, PaymentType::Credit, 1_044, 1_044, 240, 0),
        ],
    };
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert_eq!(side_sum(&entry, RATE_DIFF, Dr), 200);
    assert_eq!(side_sum(&entry, RATE_DIFF, Cr), 0);
    // Debit 12,668 + 18,684 + 200 (rate difference) = 31,552 = credit 27,636 + 1,764 + 1,752 + 400.
    assert_eq!(entry.total(), p(31_552));
}

#[test]
fn freight_paid_by_the_seller_makes_no_freight_line() {
    let mut lot = event1_lot();
    // Money agent case 10: the seller paid the driver. Deductions 7,084, seller credit 27,316.
    lot.freight_deducted = Paisa::ZERO;
    lot.seller_credit = p(27_316);
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert!(
        entry
            .lines()
            .iter()
            .all(|l| l.account_id != AccountId::new(FREIGHT))
    );
}

#[test]
fn an_entry_that_does_not_balance_is_refused() {
    let mut lot = event1_lot();
    lot.seller_credit = p(24_317);
    let error = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap_err();
    assert!(matches!(error, LedgerError::Unbalanced { .. }), "{error:?}");
}

#[test]
fn nothing_to_post_is_an_error_and_a_huge_entry_is_too_large() {
    assert_eq!(
        build_bikri_entry(&JournalSource::default(), &accounts()).unwrap_err(),
        LedgerError::Empty
    );
    let huge = crate::ledger::MAX_LINE_PAISA / 100 + 1;
    let lot = JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(huge),
        cash_paid: Paisa::ZERO,
        freight_deducted: Paisa::ZERO,
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![sale(AKRAM, huge, PaymentType::Credit, 0, 0, 0, 0)],
    };
    assert_eq!(
        build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap_err(),
        LedgerError::TooLarge
    );
}

#[test]
fn the_layout_says_where_the_matching_lines_are() {
    // Event 1: buyers at 0 and 1, the seller credit at 2, the cash part starts at 10 with the
    // seller's نقد debit.
    let (entry, layout) = build_bikri_entry_with_layout(
        &JournalSource {
            lots: vec![event1_lot()],
        },
        &accounts(),
    )
    .unwrap();
    assert_eq!(entry.lines().len(), 12);
    let lot = &layout.lots[0];
    assert_eq!(
        lot.sales.iter().map(|s| s.buyer_debit).collect::<Vec<_>>(),
        [Some(0), Some(1)]
    );
    assert!(lot.sales.iter().all(|s| s.cash_credit.is_none()));
    assert_eq!((lot.seller_credit, lot.seller_debit), (Some(2), None));
    assert_eq!(lot.cash_debit, Some(10));
    assert_eq!(layout.cash_part_start, 10);
    // The two parts balance on their own (money agent property).
    let (sale, cash) = entry.lines().split_at(layout.cash_part_start);
    let total = |lines: &[crate::ledger::LedgerLine], side: Side| -> i64 {
        lines
            .iter()
            .filter(|l| l.side == side)
            .map(|l| l.amount.rupees())
            .sum()
    };
    assert_eq!((total(sale, Dr), total(sale, Cr)), (31_564, 31_564));
    assert_eq!((total(cash, Dr), total(cash, Cr)), (5_000, 5_000));
    // The positions point at the lines they name.
    assert_eq!(entry.lines()[2].account_id, AccountId::new(RAHIM));
    assert_eq!(entry.lines()[2].side, Cr);
    assert_eq!(entry.lines()[10].side, Dr);
}

#[test]
fn the_layout_of_a_cash_line_and_of_a_negative_credit() {
    // Event 2: the one-time buyer's debit at 0, his cash settlement credit is the last line.
    let lot = JournalLot {
        seller_account_id: AccountId::new(KARIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(10_160),
        cash_paid: Paisa::ZERO,
        freight_deducted: p(1_000),
        labour: p(50),
        munshiana: p(50),
        store_rent: Paisa::ZERO,
        phone: p(20),
        sales: vec![JournalSale {
            person_key: Some("شاہد|03000000000".into()),
            ..sale(PUKHTA_BUYER, 12_800, PaymentType::Cash, 720, 720, 80, 0)
        }],
    };
    let (entry, layout) =
        build_bikri_entry_with_layout(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert_eq!(entry.lines().len(), 11);
    let row = layout.lots[0].sales[0];
    assert_eq!((row.buyer_debit, row.cash_credit), (Some(0), Some(10)));
    assert_eq!(layout.lots[0].cash_debit, None);
    // Both lines carry the person key.
    assert_eq!(
        entry.lines()[0].person_key.as_deref(),
        Some("شاہد|03000000000")
    );
    assert_eq!(
        entry.lines()[10].person_key.as_deref(),
        Some("شاہد|03000000000")
    );

    // A negative credit is a debit line: `seller_credit` is None and `seller_debit` is set.
    let negative = JournalLot {
        seller_account_id: AccountId::new(RAHIM),
        partner_credits: Vec::new(),
        person_key: None,
        seller_credit: p(-2_436),
        cash_paid: Paisa::ZERO,
        freight_deducted: p(3_000),
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![sale(AKRAM, 644, PaymentType::Credit, 36, 36, 8, 0)],
    };
    let (entry, layout) = build_bikri_entry_with_layout(
        &JournalSource {
            lots: vec![negative],
        },
        &accounts(),
    )
    .unwrap();
    let lot = &layout.lots[0];
    assert_eq!((lot.seller_credit, lot.seller_debit), (None, Some(1)));
    assert_eq!(entry.lines()[1].side, Dr);
    // The same entry as build_bikri_entry makes.
    let plain = build_bikri_entry(
        &JournalSource {
            lots: vec![JournalLot {
                seller_account_id: AccountId::new(RAHIM),
                partner_credits: Vec::new(),
                person_key: None,
                seller_credit: p(-2_436),
                cash_paid: Paisa::ZERO,
                freight_deducted: p(3_000),
                labour: Paisa::ZERO,
                munshiana: Paisa::ZERO,
                store_rent: Paisa::ZERO,
                phone: Paisa::ZERO,
                sales: vec![sale(AKRAM, 644, PaymentType::Credit, 36, 36, 8, 0)],
            }],
        },
        &accounts(),
    )
    .unwrap();
    assert_eq!(plain, entry);
}

// ---- A trip lot: the seller credit goes to the trip partners (PRD FR-12.26, C25) ----

const TRIP_A: i64 = 201;
const TRIP_B: i64 = 202;
const TRIP_C: i64 = 203;
const OWN_GOODS: i64 = 40;

/// E2E-34: an own lot sells 15 to اکرم. The seller credit 8,460 is not on the اپنا مال account; it is
/// split 3,384 / 2,538 / 2,538 on the partners' 0604 accounts. The cost of the goods is not here.
fn trip_lot(credit: i64, parts: [i64; 3], total: i64, comm: i64, laga: i64) -> JournalLot {
    JournalLot {
        seller_account_id: AccountId::new(OWN_GOODS),
        partner_credits: vec![
            (AccountId::new(TRIP_A), p(parts[0])),
            (AccountId::new(TRIP_B), p(parts[1])),
            (AccountId::new(TRIP_C), p(parts[2])),
        ],
        person_key: None,
        seller_credit: p(credit),
        cash_paid: Paisa::ZERO,
        freight_deducted: Paisa::ZERO,
        labour: Paisa::ZERO,
        munshiana: Paisa::ZERO,
        store_rent: Paisa::ZERO,
        phone: Paisa::ZERO,
        sales: vec![sale(AKRAM, total, PaymentType::Credit, comm, comm, laga, 0)],
    }
}

#[test]
fn e2e_34_the_seller_credit_goes_to_the_trip_partners_and_the_entry_is_9_660() {
    let lot = trip_lot(8_460, [3_384, 2_538, 2_538], 9_660, 540, 120);
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    // 9,660 = 540 + 540 + 120 + 8,460.
    assert_eq!(entry.total(), p(9_660));
    assert_eq!(side_sum(&entry, AKRAM, Dr), 9_660);
    assert_eq!(side_sum(&entry, TRIP_A, Cr), 3_384);
    assert_eq!(side_sum(&entry, TRIP_B, Cr), 2_538);
    assert_eq!(side_sum(&entry, TRIP_C, Cr), 2_538);
    assert_eq!(side_sum(&entry, SELLER_COMM, Cr), 540);
    assert_eq!(side_sum(&entry, BUYER_COMM, Cr), 540);
    assert_eq!(side_sum(&entry, LAGA, Cr), 120);
    // Nothing on the اپنا مال account, and no cash line.
    assert_eq!(side_sum(&entry, OWN_GOODS, Cr), 0);
    assert_eq!(side_sum(&entry, OWN_GOODS, Dr), 0);
    assert_eq!(entry.lines().len(), 1 + 3 + 3);
}

#[test]
fn a_loss_on_a_trip_lot_is_a_debit_on_the_partners_never_a_negative_line() {
    // 1 x 10, labour 50: credit 10 - 1 - 50 = -41, split 17 / 12 / 12. Buyer total 10 + 1 + 8 = 19.
    let mut lot = trip_lot(-41, [-17, -12, -12], 19, 1, 8);
    lot.labour = p(50);
    let entry = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap();
    assert_eq!(entry.total(), p(19 + 41));
    assert_eq!(side_sum(&entry, TRIP_A, Dr), 17);
    assert_eq!(side_sum(&entry, TRIP_B, Dr), 12);
    assert_eq!(side_sum(&entry, TRIP_C, Dr), 12);
    assert_eq!(side_sum(&entry, TRIP_A, Cr), 0);
    assert_eq!(side_sum(&entry, LABOUR, Cr), 50);
}

#[test]
fn parts_that_do_not_add_up_to_the_credit_do_not_balance_and_nothing_is_built() {
    // One rupee short on one partner: the entry would not balance, so it is refused.
    let lot = trip_lot(8_460, [3_384, 2_538, 2_537], 9_660, 540, 120);
    let error = build_bikri_entry(&JournalSource { lots: vec![lot] }, &accounts()).unwrap_err();
    assert!(matches!(error, LedgerError::Unbalanced { .. }), "{error:?}");
}
