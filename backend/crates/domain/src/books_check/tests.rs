//! Unit tests of the books comparisons with the numbers of PRD 4.4 and 4.5 (E2E-1 to E2E-5).

use super::*;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn account(id: i64) -> AccountId {
    AccountId::new(id)
}

fn entry(id: i64) -> EntryId {
    EntryId::new(id)
}

fn system() -> SystemAccounts {
    SystemAccounts {
        cash: account(1),
        seller_commission: account(2),
        buyer_commission: account(3),
        laga: account(4),
        phone: account(5),
        rate_diff: account(6),
        labour: account(7),
        freight: account(8),
        store_rent: account(9),
        munshiana: account(10),
    }
}

const RAHIM: i64 = 20;
const AKRAM: i64 = 30;
const ARSHAD: i64 = 31;

fn line(entry_id: i64, account_id: AccountId, debit: i64, credit: i64) -> LineSums {
    LineSums {
        entry_id: entry(entry_id),
        account_id,
        debit: rs(debit),
        credit: rs(credit),
    }
}

/// The lines of PRD 4.4 (Event 1), as `ledger::post` writes them: sums per account.
fn event_one_lines() -> Vec<LineSums> {
    let s = system();
    vec![
        line(1, account(AKRAM), 12_880, 0),
        line(1, account(ARSHAD), 18_684, 0),
        // رحیم: credit 24,316 and the نقد debit 5,000.
        line(1, account(RAHIM), 5_000, 24_316),
        line(1, s.seller_commission, 0, 1_764),
        line(1, s.buyer_commission, 0, 1_764),
        line(1, s.laga, 0, 400),
        line(1, s.phone, 0, 20),
        line(1, s.labour, 0, 250),
        line(1, s.freight, 0, 3_000),
        line(1, s.munshiana, 0, 50),
        // The cash part: روکڑ credit 5,000.
        line(1, s.cash, 0, 5_000),
    ]
}

fn event_one_lot() -> LotPosting {
    LotPosting {
        entry_id: entry(1),
        lot_id: 1,
        seller_account_id: account(RAHIM),
        trip: false,
        first: true,
        seller_amount: rs(29_400),
        seller_comm: rs(1_764),
        freight: rs(3_000),
        freight_by_firm: true,
        labour: rs(250),
        munshiana: rs(50),
        store_rent: Paisa::ZERO,
        phone: rs(20),
        cash_paid: rs(5_000),
    }
}

fn buyers_event_one() -> Vec<BuyerSums> {
    vec![
        BuyerSums {
            entry_id: entry(1),
            account_id: account(AKRAM),
            buyer_total: rs(12_880),
            paid: Paisa::ZERO,
        },
        BuyerSums {
            entry_id: entry(1),
            account_id: account(ARSHAD),
            buyer_total: rs(18_684),
            paid: Paisa::ZERO,
        },
    ]
}

fn sales_event_one() -> Vec<SaleSums> {
    vec![SaleSums {
        entry_id: entry(1),
        seller_comm: rs(1_764),
        buyer_comm: rs(1_764),
        laga: rs(400),
        rate_diff: Paisa::ZERO,
    }]
}

struct Book {
    entries: Vec<PostingEntry>,
    buyers: Vec<BuyerSums>,
    sales: Vec<SaleSums>,
    lots: Vec<LotPosting>,
    parts: Vec<TripPart>,
    lines: Vec<LineSums>,
    mirror: Vec<AccountId>,
}

impl Book {
    fn event_one() -> Book {
        Book {
            entries: vec![PostingEntry {
                entry_id: entry(1),
                live: true,
            }],
            buyers: buyers_event_one(),
            sales: sales_event_one(),
            lots: vec![event_one_lot()],
            parts: Vec::new(),
            lines: event_one_lines(),
            mirror: Vec::new(),
        }
    }

    fn compare(&self) -> Vec<Difference> {
        compare_postings(&PostingInputs {
            entries: &self.entries,
            buyers: &self.buyers,
            sales: &self.sales,
            lots: &self.lots,
            parts: &self.parts,
            system: system(),
            lines: &self.lines,
            mirror: &self.mirror,
        })
        .unwrap()
    }
}

#[test]
fn the_seller_credit_of_prd_4_4_is_24_316() {
    // 29,400 − 1,764 − (3,000 + 250 + 50 + 0 + 20) = 24,316.
    assert_eq!(lot_credit(&event_one_lot()).unwrap(), rs(24_316));
}

#[test]
fn a_later_posting_carries_no_lot_charge() {
    // PRD FR-12.17: the later sale of E2E-16b, 2,800 less 168 commission = 2,632.
    let later = LotPosting {
        first: false,
        seller_amount: rs(2_800),
        seller_comm: rs(168),
        ..event_one_lot()
    };
    assert_eq!(lot_credit(&later).unwrap(), rs(2_632));
}

#[test]
fn freight_paid_by_the_seller_is_not_deducted() {
    // PRD E6: 29,400 − 1,764 − (250 + 50 + 20) = 27,316.
    let lot = LotPosting {
        freight_by_firm: false,
        ..event_one_lot()
    };
    assert_eq!(lot_credit(&lot).unwrap(), rs(27_316));
}

#[test]
fn event_one_matches_its_journal() {
    assert_eq!(Book::event_one().compare(), Vec::<Difference>::new());
}

#[test]
fn event_two_a_chukta_buyer_paid_in_full_matches() {
    // PRD 4.5: 12,800 debit and 12,800 credit on 0030; روکڑ debit 12,800; کریم credit 10,160.
    let s = system();
    let karim = 21;
    let chukta = 40;
    let book = Book {
        entries: vec![PostingEntry {
            entry_id: entry(2),
            live: true,
        }],
        buyers: vec![BuyerSums {
            entry_id: entry(2),
            account_id: account(chukta),
            buyer_total: rs(12_800),
            paid: rs(12_800),
        }],
        sales: vec![SaleSums {
            entry_id: entry(2),
            seller_comm: rs(720),
            buyer_comm: rs(720),
            laga: rs(80),
            rate_diff: Paisa::ZERO,
        }],
        lots: vec![LotPosting {
            entry_id: entry(2),
            lot_id: 2,
            seller_account_id: account(karim),
            trip: false,
            first: true,
            seller_amount: rs(12_000),
            seller_comm: rs(720),
            freight: rs(1_000),
            freight_by_firm: true,
            labour: rs(50),
            munshiana: rs(50),
            store_rent: Paisa::ZERO,
            phone: rs(20),
            cash_paid: Paisa::ZERO,
        }],
        parts: Vec::new(),
        lines: vec![
            line(2, account(chukta), 12_800, 12_800),
            line(2, account(karim), 0, 10_160),
            line(2, s.seller_commission, 0, 720),
            line(2, s.buyer_commission, 0, 720),
            line(2, s.laga, 0, 80),
            line(2, s.phone, 0, 20),
            line(2, s.labour, 0, 50),
            line(2, s.freight, 0, 1_000),
            line(2, s.munshiana, 0, 50),
            line(2, s.cash, 12_800, 0),
        ],
        mirror: Vec::new(),
    };
    assert_eq!(book.compare(), Vec::<Difference>::new());
}

#[test]
fn a_changed_buyer_total_is_found_and_named() {
    // The buyer rate of the first line raised by one rupee with its value and total: 20 more.
    let mut book = Book::event_one();
    book.buyers[0].buyer_total = rs(12_900);
    let found = book.compare();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].table, "bikri_sales");
    assert!(found[0].what.contains("journal entry 1"));
    assert!(found[0].what.contains("account 30"));
}

#[test]
fn a_changed_seller_amount_is_found() {
    // Seller rate +1 on 30 nag: the gross and the credit differ by 30.
    let mut book = Book::event_one();
    book.lots[0].seller_amount = rs(29_430);
    let found = book.compare();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].what.contains(&format!("account {RAHIM}")));
}

#[test]
fn a_changed_rate_difference_is_found_on_a_reversed_posting_too() {
    let mut book = Book::event_one();
    book.entries[0].live = false;
    book.lots.clear();
    book.sales[0].rate_diff = rs(100);
    let found = book.compare();
    assert_eq!(found.len(), 1, "{found:?}");
}

#[test]
fn a_reversed_posting_is_compared_on_its_sale_accounts_only() {
    // The lot rows are not given for a reversed posting, so the seller, cash and the payables are
    // not compared; the buyers and the four income accounts are.
    let mut book = Book::event_one();
    book.entries[0].live = false;
    book.lots.clear();
    assert_eq!(book.compare(), Vec::<Difference>::new());
}

#[test]
fn an_extra_line_on_a_live_posting_is_found() {
    let mut book = Book::event_one();
    book.lines.push(line(1, account(99), 100, 0));
    let found = book.compare();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].what.contains("account 99"));
}

#[test]
fn a_missing_line_on_a_live_posting_is_found() {
    let mut book = Book::event_one();
    book.lines.retain(|l| l.account_id != system().munshiana);
    assert_eq!(book.compare().len(), 1);
}

#[test]
fn a_deleted_sale_line_of_a_live_posting_is_found() {
    // Both sale lines gone: every account of the entry now differs.
    let mut book = Book::event_one();
    book.buyers.clear();
    book.sales.clear();
    book.lots[0].seller_amount = Paisa::ZERO;
    book.lots[0].seller_comm = Paisa::ZERO;
    assert!(!book.compare().is_empty());
}

#[test]
fn a_negative_rate_difference_is_a_debit() {
    // A rate difference of −200 is posted as a debit of 200 on 04010005.
    let s = system();
    let mut book = Book::event_one();
    book.sales[0].rate_diff = rs(-200);
    book.lines.push(line(1, s.rate_diff, 200, 0));
    // The buyer value is 200 lower than the seller amount, so the buyer total is lower too.
    book.buyers[0].buyer_total = rs(12_680);
    book.lines.retain(|l| l.account_id != account(AKRAM));
    book.lines.push(line(1, account(AKRAM), 12_680, 0));
    assert_eq!(book.compare(), Vec::<Difference>::new());
}

#[test]
fn a_trip_lot_is_posted_to_its_partners() {
    // PRD E2E-34: the seller credit 9,000 − 540 = 8,460 goes to 40 / 30 / 30: 3,384 / 2,538 / 2,538.
    let s = system();
    let own = 50;
    let (a, b, c) = (60, 61, 62);
    let buyer = AKRAM;
    let book = Book {
        entries: vec![PostingEntry {
            entry_id: entry(3),
            live: true,
        }],
        buyers: vec![BuyerSums {
            entry_id: entry(3),
            account_id: account(buyer),
            buyer_total: rs(9_660),
            paid: Paisa::ZERO,
        }],
        sales: vec![SaleSums {
            entry_id: entry(3),
            seller_comm: rs(540),
            buyer_comm: rs(540),
            laga: rs(120),
            rate_diff: Paisa::ZERO,
        }],
        lots: vec![LotPosting {
            entry_id: entry(3),
            lot_id: 3,
            seller_account_id: account(own),
            trip: true,
            first: true,
            seller_amount: rs(9_000),
            seller_comm: rs(540),
            freight: Paisa::ZERO,
            freight_by_firm: true,
            labour: Paisa::ZERO,
            munshiana: Paisa::ZERO,
            store_rent: Paisa::ZERO,
            phone: Paisa::ZERO,
            cash_paid: Paisa::ZERO,
        }],
        parts: [(a, 3_384), (b, 2_538), (c, 2_538)]
            .into_iter()
            .map(|(id, part)| TripPart {
                entry_id: entry(3),
                lot_id: 3,
                account_id: account(id),
                amount: rs(part),
            })
            .collect(),
        lines: vec![
            line(3, account(buyer), 9_660, 0),
            line(3, account(a), 0, 3_384),
            line(3, account(b), 0, 2_538),
            line(3, account(c), 0, 2_538),
            line(3, s.seller_commission, 0, 540),
            line(3, s.buyer_commission, 0, 540),
            line(3, s.laga, 0, 120),
        ],
        mirror: Vec::new(),
    };
    assert_eq!(book.compare(), Vec::<Difference>::new());
    // One partner's part raised by a rupee and his line with it: the parts no longer add up.
    let mut changed = Book { ..book };
    changed.parts[0].amount = rs(3_385);
    changed.lines[1] = line(3, account(a), 0, 3_385);
    let found = changed.compare();
    assert!(
        found.iter().any(|d| d.table == "lot_credit_parts"),
        "{found:?}"
    );
}

#[test]
fn the_totals_of_a_lot_of_prd_4_4_are_right() {
    let lot = LotRow {
        cancelled: false,
        lines_qty: 50,
        lines_gross: rs(29_400),
        lines_comm: rs(1_764),
        qty_sold: 50,
        gross: rs(29_400),
        seller_comm: rs(1_764),
        deductions: rs(10_084),
        net: rs(19_316),
        freight: rs(3_000),
        freight_by_firm: true,
        labour: rs(250),
        munshiana: rs(50),
        cash_paid: rs(5_000),
        store_rent: Paisa::ZERO,
        phone: rs(20),
    };
    assert_eq!(lot_difference(&lot).unwrap(), None);
    let raised = LotRow {
        gross: rs(29_500),
        net: rs(19_416),
        ..lot
    };
    assert_eq!(lot_difference(&raised).unwrap(), Some("gross"));
    let net_only = LotRow {
        net: rs(19_416),
        ..lot
    };
    assert_eq!(lot_difference(&net_only).unwrap(), Some("net"));
    let deductions_only = LotRow {
        deductions: rs(10_000),
        net: rs(19_400),
        ..lot
    };
    assert_eq!(
        lot_difference(&deductions_only).unwrap(),
        Some("deductions")
    );
    let qty = LotRow {
        qty_sold: 49,
        ..lot
    };
    assert_eq!(lot_difference(&qty).unwrap(), Some("qty_sold"));
}

fn share(id: i64, partner: i64, bps: i32, from: &str, at: &str) -> ShareHistory {
    ShareHistory {
        id,
        partner_id: PartnerId::new(partner),
        share_bps: Bps::new(bps).unwrap(),
        effective_from: from.to_owned(),
        created_at: at.to_owned(),
    }
}

fn forty_thirty_thirty() -> Vec<ShareHistory> {
    vec![
        share(1, 1, 4_000, "2026-09-01", "2026-09-01T08:00:00Z"),
        share(2, 2, 3_000, "2026-09-01", "2026-09-01T08:00:00Z"),
        share(3, 3, 3_000, "2026-09-01", "2026-09-01T08:00:00Z"),
    ]
}

fn stored_of(rows: &[(i64, ShareKind, i64, i32, i64)]) -> Vec<StoredShare> {
    rows.iter()
        .map(|&(partner, kind, base, bps, amount)| StoredShare {
            partner_id: PartnerId::new(partner),
            kind,
            base: rs(base),
            share_bps: Bps::new(bps).unwrap(),
            amount: rs(amount),
        })
        .collect()
}

#[test]
fn shared_income_3948_is_1580_1184_1184() {
    // PRD 4.4, E2E-4.
    let history = forty_thirty_thirty();
    let states = share_states(&history, "2026-10-10");
    // One real state; the shorter prefixes of the history (not every partner) are kept as well.
    let newest: Vec<i32> = states[0].iter().map(|(_, b)| b.get()).collect();
    assert_eq!(newest, vec![4_000, 3_000, 3_000]);
    let bases = bases_of((Paisa::ZERO, rs(3_948)), (Paisa::ZERO, Paisa::ZERO)).unwrap();
    let rows = stored_of(&[
        (1, ShareKind::Income, 3_948, 4_000, 1_580),
        (2, ShareKind::Income, 3_948, 3_000, 1_184),
        (3, ShareKind::Income, 3_948, 3_000, 1_184),
    ]);
    assert!(share_rows_match(&states, bases, &rows));
    // The leftover rupee moved to another partner: found.
    let moved = stored_of(&[
        (1, ShareKind::Income, 3_948, 4_000, 1_579),
        (2, ShareKind::Income, 3_948, 3_000, 1_185),
        (3, ShareKind::Income, 3_948, 3_000, 1_184),
    ]);
    assert!(!share_rows_match(&states, bases, &moved));
}

#[test]
fn a_shop_expense_of_3000_is_split_and_made_negative() {
    // PRD 4.8, C11: −1,200 / −900 / −900.
    let history = forty_thirty_thirty();
    let states = share_states(&history, "2026-10-10");
    let bases = bases_of((Paisa::ZERO, Paisa::ZERO), (rs(3_000), Paisa::ZERO)).unwrap();
    let rows = stored_of(&[
        (1, ShareKind::Expense, 3_000, 4_000, -1_200),
        (2, ShareKind::Expense, 3_000, 3_000, -900),
        (3, ShareKind::Expense, 3_000, 3_000, -900),
    ]);
    assert!(share_rows_match(&states, bases, &rows));
}

#[test]
fn an_entry_with_no_income_or_expense_line_has_no_rows() {
    let states = share_states(&forty_thirty_thirty(), "2026-10-10");
    let none = ShareBases {
        income: Paisa::ZERO,
        expense: Paisa::ZERO,
    };
    assert!(share_rows_match(&states, none, &[]));
    let extra = stored_of(&[(1, ShareKind::Income, 100, 4_000, 40)]);
    assert!(!share_rows_match(&states, none, &extra));
}

#[test]
fn a_share_change_today_leaves_an_earlier_entry_of_today_valid() {
    // 40 / 30 / 30 from 01-09-26, changed to 50 / 25 / 25 on 10-10-26. An entry of 10-10-26 saved
    // before the change was split 40 / 30 / 30; one saved after it was split 50 / 25 / 25.
    let mut history = forty_thirty_thirty();
    history.push(share(4, 1, 5_000, "2026-10-10", "2026-10-10T12:00:00Z"));
    history.push(share(5, 2, 2_500, "2026-10-10", "2026-10-10T12:00:00Z"));
    history.push(share(6, 3, 2_500, "2026-10-10", "2026-10-10T12:00:00Z"));
    let day_before = share_states(&history, "2026-10-09");
    let states = share_states(&history, "2026-10-10");
    // The newest state comes first.
    let newest: Vec<i32> = states[0].iter().map(|(_, b)| b.get()).collect();
    assert_eq!(newest, vec![5_000, 2_500, 2_500]);
    let bases = bases_of((Paisa::ZERO, rs(1_000)), (Paisa::ZERO, Paisa::ZERO)).unwrap();
    let old = stored_of(&[
        (1, ShareKind::Income, 1_000, 4_000, 400),
        (2, ShareKind::Income, 1_000, 3_000, 300),
        (3, ShareKind::Income, 1_000, 3_000, 300),
    ]);
    let new = stored_of(&[
        (1, ShareKind::Income, 1_000, 5_000, 500),
        (2, ShareKind::Income, 1_000, 2_500, 250),
        (3, ShareKind::Income, 1_000, 2_500, 250),
    ]);
    assert!(share_rows_match(&states, bases, &old));
    assert!(share_rows_match(&states, bases, &new));
    // An entry of 09-10-26 split with the later shares is not valid; one with the old shares is.
    assert!(!share_rows_match(&day_before, bases, &new));
    assert!(share_rows_match(&day_before, bases, &old));
}

#[test]
fn a_share_change_saved_after_an_entry_of_its_day_is_known_whatever_the_clock_says() {
    // NFR-REL-11: the clock may be wrong. The rows only grow, so whatever time they carry, an entry
    // was split with a prefix of them: here the same time on every row.
    let at = "2026-10-10T12:00:00Z";
    let mut history = forty_thirty_thirty();
    for (id, partner, bps) in [(4, 1, 5_000), (5, 2, 2_500), (6, 3, 2_500)] {
        history.push(share(id, partner, bps, "2026-10-10", at));
    }
    for (id, partner, bps) in [(7, 1, 6_000), (8, 2, 2_000), (9, 3, 2_000)] {
        history.push(share(id, partner, bps, "2026-10-10", at));
    }
    let states = share_states(&history, "2026-10-10");
    let bases = bases_of((Paisa::ZERO, rs(1_000)), (Paisa::ZERO, Paisa::ZERO)).unwrap();
    for (a, b, c) in [
        (4_000, 3_000, 3_000),
        (5_000, 2_500, 2_500),
        (6_000, 2_000, 2_000),
    ] {
        let rows = stored_of(&[
            (1, ShareKind::Income, 1_000, a, i64::from(a) / 10),
            (2, ShareKind::Income, 1_000, b, i64::from(b) / 10),
            (3, ShareKind::Income, 1_000, c, i64::from(c) / 10),
        ]);
        assert!(share_rows_match(&states, bases, &rows), "{a} / {b} / {c}");
    }
    // Shares that never existed are not accepted.
    let never = stored_of(&[
        (1, ShareKind::Income, 1_000, 7_000, 700),
        (2, ShareKind::Income, 1_000, 1_500, 150),
        (3, ShareKind::Income, 1_000, 1_500, 150),
    ]);
    assert!(!share_rows_match(&states, bases, &never));
}

#[test]
fn the_row_saved_last_wins_on_the_same_day() {
    let mut history = forty_thirty_thirty();
    history.push(share(4, 1, 5_000, "2026-10-10", "2026-10-10T12:00:00Z"));
    history.push(share(5, 2, 2_500, "2026-10-10", "2026-10-10T12:00:00Z"));
    history.push(share(6, 3, 2_500, "2026-10-10", "2026-10-10T12:00:00Z"));
    history.push(share(7, 1, 6_000, "2026-10-10", "2026-10-10T15:00:00Z"));
    history.push(share(8, 2, 2_000, "2026-10-10", "2026-10-10T15:00:00Z"));
    history.push(share(9, 3, 2_000, "2026-10-10", "2026-10-10T15:00:00Z"));
    let states = share_states(&history, "2026-10-10");
    let first = states.first().unwrap();
    let bps: Vec<i32> = first.iter().map(|(_, b)| b.get()).collect();
    assert_eq!(bps, vec![6_000, 2_000, 2_000]);
}

#[test]
fn no_shares_on_the_day_means_no_split_is_possible() {
    // Shares that start after the day cannot have split an entry of it.
    let history = forty_thirty_thirty();
    let states = share_states(&history, "2026-08-31");
    assert!(states.is_empty());
    let bases = bases_of((Paisa::ZERO, rs(100)), (Paisa::ZERO, Paisa::ZERO)).unwrap();
    assert!(!share_rows_match(&states, bases, &[]));
}

#[test]
fn a_reversal_copies_the_original_rows_negated() {
    let original = stored_of(&[
        (1, ShareKind::Income, 3_948, 4_000, 1_580),
        (2, ShareKind::Income, 3_948, 3_000, 1_184),
    ]);
    let reversal = stored_of(&[
        (2, ShareKind::Income, -3_948, 3_000, -1_184),
        (1, ShareKind::Income, -3_948, 4_000, -1_580),
    ]);
    assert!(reversal_rows_match(&original, &reversal).unwrap());
    let wrong = stored_of(&[
        (1, ShareKind::Income, -3_948, 4_000, -1_580),
        (2, ShareKind::Income, -3_948, 3_000, -1_183),
    ]);
    assert!(!reversal_rows_match(&original, &wrong).unwrap());
    // A reversal of an entry with no rows has none.
    assert!(reversal_rows_match(&[], &[]).unwrap());
}

#[test]
fn the_bases_follow_the_signs_of_c9_and_c11() {
    // Income: credit 5,000 and debit 200 (a negative rate difference): base 4,800. Expense: debit
    // 3,000 less a refund credit 500: base 2,500.
    let bases = bases_of((rs(200), rs(5_000)), (rs(3_000), rs(500))).unwrap();
    assert_eq!(bases.income, rs(4_800));
    assert_eq!(bases.expense, rs(2_500));
}

#[test]
fn an_overflow_is_an_error_not_a_wrap() {
    let big = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
    // Credit minus a negative debit: more than the largest amount.
    assert!(bases_of((rs(-1), big), (Paisa::ZERO, Paisa::ZERO)).is_err());
}

#[test]
fn the_mirror_lines_of_a_linked_khata_are_left_out_but_must_cancel() {
    // PRD FR-16.6: a khata lot linked to the shop adds 0605 lines per partner and the contra
    // 03070001 on the other side. They are not compared line by line, but add up to nothing.
    let (a, b, contra) = (70, 71, 72);
    let mut book = Book::event_one();
    book.mirror = vec![account(a), account(b), account(contra)];
    book.lines.push(line(1, account(a), 0, 3_600));
    book.lines.push(line(1, account(b), 0, 5_400));
    book.lines.push(line(1, account(contra), 9_000, 0));
    assert_eq!(book.compare(), Vec::<Difference>::new());
    // One part raised by a rupee: they no longer add up to nothing.
    book.lines[11] = line(1, account(a), 0, 3_700);
    let found = book.compare();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].what.contains("linked money"));
}

#[test]
fn a_cancelled_lot_keeps_its_columns_and_has_no_live_lines() {
    // plan 16: a cancel keeps the totals of the lot as the record of what was cancelled.
    let lot = LotRow {
        cancelled: true,
        lines_qty: 0,
        lines_gross: Paisa::ZERO,
        lines_comm: Paisa::ZERO,
        qty_sold: 50,
        gross: rs(29_400),
        seller_comm: rs(1_764),
        deductions: rs(10_084),
        net: rs(19_316),
        freight: rs(3_000),
        freight_by_firm: true,
        labour: rs(250),
        munshiana: rs(50),
        cash_paid: rs(5_000),
        store_rent: Paisa::ZERO,
        phone: rs(20),
    };
    assert_eq!(lot_difference(&lot).unwrap(), None);
    let alive = LotRow {
        lines_qty: 1,
        ..lot
    };
    assert!(lot_difference(&alive).unwrap().is_some());
}

#[test]
fn bases_of_is_the_formula_of_share_bases() {
    // One definition of C9 and C11 (Code Standards B5): the bases worked out from the sums per head
    // equal `partners::share_bases` on the lines of an entry.
    use crate::ledger::{LedgerLine, Side, build_entry};
    use crate::partners::{Nature, share_bases};
    let income = account(1);
    let expense = account(2);
    let other = account(3);
    // Income: credit 5,000 and debit 200; expense: debit 3,000 and credit 500.
    let entry = build_entry(vec![
        LedgerLine::new(income, Side::Cr, rs(5_000)),
        LedgerLine::new(income, Side::Dr, rs(200)),
        LedgerLine::new(expense, Side::Dr, rs(3_000)),
        LedgerLine::new(expense, Side::Cr, rs(500)),
        // Debit 200 + 3,000 and credit 5,000 + 500: the rest balances on another account.
        LedgerLine::new(other, Side::Dr, rs(2_300)),
    ])
    .unwrap();
    let by_lines = share_bases(&entry, |id| {
        if id == income {
            Nature::Income
        } else if id == expense {
            Nature::Expense
        } else {
            Nature::Other
        }
    })
    .unwrap();
    let by_sums = bases_of((rs(200), rs(5_000)), (rs(3_000), rs(500))).unwrap();
    assert_eq!(by_lines, by_sums);
}

#[test]
fn a_line_moved_between_chukta_persons_is_found() {
    // PRD FR-13.5: شاہد buys 12,800 and pays 12,800 (net 0); the lines show the same on ناصر.
    let sale = |person: &str, total: i64, paid: i64| PersonSale {
        entry_id: entry(2),
        person: person.to_owned(),
        buyer_total: rs(total),
        paid: rs(paid),
    };
    let lines = |person: &str, debit: i64, credit: i64| PersonLines {
        entry_id: entry(2),
        person: person.to_owned(),
        debit: rs(debit),
        credit: rs(credit),
    };
    let sales = [sale("shahid", 12_800, 12_800), sale("nasir", 2_000, 0)];
    let same = [lines("shahid", 12_800, 12_800), lines("nasir", 2_000, 0)];
    assert_eq!(
        compare_persons(&sales, &same).unwrap(),
        Vec::<Difference>::new()
    );
    // The two persons swap their lines: the account total is the same, the persons are not.
    let moved = [lines("nasir", 12_800, 12_800), lines("shahid", 2_000, 0)];
    assert_eq!(compare_persons(&sales, &moved).unwrap().len(), 2);
}

#[test]
fn a_weight_line_is_the_weight_times_the_rate_rounded_once() {
    // PRD C1: 12.345 kg at 100 a kilogram = 1,234.5 -> 1,235.
    assert_eq!(
        weight_line_difference(12_345, rs(100), rs(1_235), rs(100), rs(1_235)).unwrap(),
        None
    );
    assert_eq!(
        weight_line_difference(12_345, rs(100), rs(1_234), rs(100), rs(1_235)).unwrap(),
        Some("seller amount")
    );
    assert_eq!(
        weight_line_difference(12_345, rs(100), rs(1_235), rs(110), rs(1_235)).unwrap(),
        Some("buyer value")
    );
    // A weight below zero is not a weight.
    assert!(weight_line_difference(-1, rs(100), rs(0), rs(100), rs(0)).is_err());
}
