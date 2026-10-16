//! Bill matching and advance matching on real data (PRD FR-15.4, FR-15.5, FR-15.8, plan 13), and
//! the canonical re-match that edit and cancel will use (plan 16).

use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{BikriDraft, PaymentType};
use mandi_services::core::money::Paisa;
use mandi_services::db::matching as repo;
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::matching;

use crate::common::raw;
use crate::helpers::*;
use crate::support::*;

/// A cash receipt posted straight into the journal: cash بنام, the buyer (or his person) جمع.
/// It stands for a وصولی voucher (plan 15) and re-matches the key, as that voucher will.
fn receipt(w: &World, buyer: i64, key: Option<&str>, rupees_text: &str) -> i64 {
    receipt_on(w, buyer, key, rupees_text, (2026, 10, 10))
}

/// The same receipt on another day.
fn receipt_on(
    w: &World,
    buyer: i64,
    key: Option<&str>,
    rupees_text: &str,
    (year, month, day): (i32, u32, u32),
) -> i64 {
    let iso = format!("{year:04}-{month:02}-{day:02}");
    let amount = Paisa::from_rupees(rupees_text.replace(',', "").parse().unwrap()).unwrap();
    let cash = id_of(&w.env, "01010001");
    w.env
        .ctx
        .db()
        .write(|tx| {
            let mut credit = LedgerLine::new(
                mandi_services::core::ids::AccountId::new(buyer),
                Side::Cr,
                amount,
            );
            if let Some(key) = key {
                credit = credit.with_person(key);
            }
            let entry = build_entry(vec![
                LedgerLine::new(
                    mandi_services::core::ids::AccountId::new(cash),
                    Side::Dr,
                    amount,
                ),
                credit,
            ])
            .unwrap();
            let posted = ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::from_ymd(year, month, day)
                        .unwrap(),
                    posting_type: PostingType::Receipt,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )?;
            // A receipt is a new credit at the end of the key: the fast path of plan 15.
            let credit_line: i64 = tx.query_row(
                "SELECT id FROM journal_lines WHERE entry_id = ?1 AND account_id = ?2 AND cr_paisa > 0",
                [posted.entry_id, buyer],
                |r| r.get(0),
            )?;
            matching::match_new_credits(
                tx,
                w.owner.user_id,
                "2026-10-10T09:00:00.000Z",
                buyer,
                key,
                &iso,
                &[matching::NewLine {
                    line_id: credit_line,
                    amount,
                }],
            )?;
            Ok::<_, mandi_services::ServiceError>(posted.entry_id)
        })
        .unwrap()
}

fn line_id(w: &World, entry_id: i64, account: i64, side: &str) -> i64 {
    let column = if side == "dr" { "dr_paisa" } else { "cr_paisa" };
    raw(&w.dir)
        .query_row(
            &format!(
                "SELECT id FROM journal_lines WHERE entry_id = ?1 AND account_id = ?2 AND {column} > 0 ORDER BY line_no LIMIT 1"
            ),
            [entry_id, account],
            |r| r.get(0),
        )
        .unwrap()
}

fn opening_entry(w: &World, account: i64) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT e.id FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id
             WHERE e.posting_type = 'opening' AND l.account_id = ?1",
            [account],
            |r| r.get(0),
        )
        .unwrap()
}

/// One credit sale of 20 × 600 to a buyer: 12,000 + 720 + 160 = 12,880.
fn credit_sale(w: &World, buyer: i64) -> BikriDraft {
    draft(vec![with_sales(
        bare_lot(w, "lot-1", w.rahim, w.banana, "20"),
        vec![sale("s-1", buyer, "20", "600")],
    )])
}

#[test]
fn the_cash_of_a_cash_sale_never_pays_an_older_bill() {
    // Money agent B3: an old open bill of 2,880 and today a cash sale of 12,800 to the same
    // buyer. The 12,800 credit is pinned to the 12,800 debit; the 2,880 stays open.
    let w = world();
    open_account(&w, w.akram, "2,880", "");
    let mut line = sale("s-1", w.akram, "10", "1200");
    line.payment_type = Some(PaymentType::Cash);
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![line],
    )]);
    let saved = save_ok(&w, &d);
    let bill = line_id(&w, saved.entry_id, w.akram, "dr");
    let cash_credit = line_id(&w, saved.entry_id, w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(bill, cash_credit, 12_800)]);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    // Nothing touched the opening bill.
    let old = line_id(&w, opening_entry(&w, w.akram), w.akram, "dr");
    assert!(rows(&w, "bill_allocations").iter().all(|r| r.0 != old));
    // The canonical working agrees: nothing to change.
    let changed = w
        .env
        .ctx
        .db()
        .write(|tx| {
            matching::rematch(
                tx,
                w.owner.user_id,
                "2026-10-10T09:00:00.000Z",
                w.akram,
                None,
            )
        })
        .unwrap();
    assert_eq!((changed.switched_off, changed.added), (0, 0));
}

#[test]
fn an_old_prepayment_settles_the_next_credit_bill() {
    // Money agent B2: a prepayment of 5,000 and a new credit bill of 12,880 leave 7,880 open.
    let w = world();
    open_account(&w, w.akram, "", "5,000");
    let saved = save_ok(&w, &credit_sale(&w, w.akram));
    let bill = line_id(&w, saved.entry_id, w.akram, "dr");
    let prepaid = line_id(&w, opening_entry(&w, w.akram), w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(bill, prepaid, 5_000)]);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 7_880));
}

#[test]
fn a_receipt_settles_the_oldest_bill_first_and_a_cancel_gives_the_age_back() {
    // PRD FR-15.4 and FR-12.22 with the money agent's example: A 2,880 (old), B 12,880 (بکری 1),
    // receipt R 5,000 pays A 2,880 and B 2,120. Cancel B: R keeps paying A, 2,120 of R is
    // unmatched, A stays the oldest open bill.
    let w = world();
    open_account(&w, w.akram, "2,880", "");
    let saved = save_ok(&w, &credit_sale(&w, w.akram));
    let a = line_id(&w, opening_entry(&w, w.akram), w.akram, "dr");
    let b = line_id(&w, saved.entry_id, w.akram, "dr");
    let r_entry = receipt(&w, w.akram, None, "5,000");
    let r = line_id(&w, r_entry, w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(a, r, 2_880), (b, r, 2_120)]);

    // Cancel B by reversal, then re-match the key (what plan 16 does inside its transaction).
    w.env
        .ctx
        .db()
        .write(|tx| {
            ledger::reverse(&w.env.ctx, tx, w.owner.user_id, saved.entry_id, "cancel")?;
            matching::rematch(
                tx,
                w.owner.user_id,
                "2026-10-10T09:00:00.000Z",
                w.akram,
                None,
            )
        })
        .unwrap();
    assert_eq!(rows(&w, "bill_allocations"), [(a, r, 2_880)]);
    // The reversed bill and its reversal are not live lines any more.
    let live = w
        .env
        .ctx
        .db()
        .read(|c| repo::live_key_lines(c, w.akram, None))
        .unwrap();
    let ids: Vec<i64> = live.iter().map(|l| l.line_id).collect();
    assert_eq!(ids, [a, r]);
    // His balance: 2,880 + 12,880 - 5,000 - 12,880 = 2,880 - 5,000 = 2,120 جمع.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 2_120));
}

#[test]
fn a_receipt_of_one_one_time_person_never_pays_another_persons_bill() {
    // PRD FR-13.5: شاہد and ناصر buy on credit through the same پختہ account.
    let w = world();
    let mut a = sale("s-1", w.pukhta_buyer, "10", "1000");
    a.pukhta = person("شاہد", "Shahid", "0300-0000000");
    a.payment_type = Some(PaymentType::Credit);
    let mut b = sale("s-2", w.pukhta_buyer, "10", "1000");
    b.pukhta = person("ناصر", "Nasir", "0300-1111111");
    b.payment_type = Some(PaymentType::Credit);
    let saved = save_ok(
        &w,
        &draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.rahim, w.banana, "20"),
            vec![a, b],
        )]),
    );
    // Each owes 10,000 + 600 + 80 = 10,680.
    let shahid = "شاہد|03000000000";
    let nasir = "ناصر|03001111111";
    let balance_of = |key: &str| ledger::person_balance(&w.env.ctx, w.pukhta_buyer, key).unwrap();
    assert_eq!(balance_of(shahid).amount_paisa.rupees(), 10_680);
    receipt(&w, w.pukhta_buyer, Some(shahid), "10,680");
    assert_eq!(balance_of(shahid).side, BalanceSide::Zero);
    assert_eq!(balance_of(nasir).amount_paisa.rupees(), 10_680);
    // The matching row points at شاہد's bill only.
    let lines = entry_lines(&w, saved.entry_id);
    assert_eq!(lines[0].3.as_deref(), Some(shahid));
    assert_eq!(lines[1].3.as_deref(), Some(nasir));
    let matched = rows(&w, "bill_allocations");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].2, 10_680);
    let bill_key: Option<String> = raw(&w.dir)
        .query_row(
            "SELECT pukhta_person_key FROM journal_lines WHERE id = ?1",
            [matched[0].0],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(bill_key.as_deref(), Some(shahid));
}

#[test]
fn a_back_dated_bikri_falls_back_to_the_full_working() {
    // Today is 11-10. بکری X (11-10) is a bill of 12,880 and the receipt R (11-10) of 5,000 pays
    // it in part. Then a بکری Y dated 10-10 is saved: its bill is older than X, so it is not
    // the newest line of the key and the fast path does not apply. The full working: R pays the
    // older bill Y first, X stays fully open: (Y, R, 5,000).
    let w = world();
    crate::common::set_today(&w.env, 2026, 10, 11);
    let mut newer = credit_sale(&w, w.akram);
    newer.header.entry_date = "11-10-2026".into();
    let x_entry = save_ok(&w, &newer).entry_id;
    let x = line_id(&w, x_entry, w.akram, "dr");
    let r_entry = receipt_on(&w, w.akram, None, "5,000", (2026, 10, 11));
    let r = line_id(&w, r_entry, w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(x, r, 5_000)]);
    let y_entry = save_ok(&w, &credit_sale(&w, w.akram)).entry_id;
    let y = line_id(&w, y_entry, w.akram, "dr");
    assert_eq!(rows(&w, "bill_allocations"), [(y, r, 5_000)]);
    // The row of the first working was switched off, not deleted.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bill_allocations WHERE is_active = 0 AND undone_by IS NOT NULL"
        ),
        1
    );
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 20_760));
}

/// A small deterministic random source for the sequence test.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const SHAHID: &str = "شاہد|03000000000";
const NASIR: &str = "ناصر|03001111111";

/// What the rows and the lines of a key say: the open bills, the unmatched credits, and the
/// balance, in paisa (debit positive).
fn key_state(w: &World, account: i64, key: Option<&str>) -> (i64, i64, i64) {
    let conn = raw(&w.dir);
    let sums = |sql: &str| -> i64 {
        conn.query_row(sql, rusqlite_params(account, key), |r| r.get(0))
            .unwrap()
    };
    let dr = sums(
        "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines WHERE account_id = ?1 AND pukhta_person_key IS ?2",
    );
    let cr = sums(
        "SELECT COALESCE(SUM(cr_paisa), 0) FROM journal_lines WHERE account_id = ?1 AND pukhta_person_key IS ?2",
    );
    let matched = sums(
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations WHERE account_id = ?1 AND pukhta_person_key IS ?2 AND is_active = 1",
    );
    (dr - matched, cr - matched, dr - cr)
}

fn rusqlite_params(account: i64, key: Option<&str>) -> (i64, Option<String>) {
    (account, key.map(str::to_owned))
}

#[test]
fn after_any_sequence_of_sales_and_receipts_the_rows_are_the_canonical_ones() {
    // Money agent property (g): the save and the receipt use the fast paths; running the full
    // working after each step must change nothing. The sequence has cash and credit lines of
    // one buyer in one بکری, two one-time persons, back-dated postings (dates 10, 11 and 12-10
    // with today 12-10) and receipts on their own days.
    let w = world();
    crate::common::set_today(&w.env, 2026, 10, 12);
    let keys: [(i64, Option<&str>); 4] = [
        (w.akram, None),
        (w.arshad, None),
        (w.pukhta_buyer, Some(SHAHID)),
        (w.pukhta_buyer, Some(NASIR)),
    ];
    for seed in [
        0x9E37_79B9_7F4A_7C15_u64,
        0xD1B5_4A32_D192_ED03,
        0x2545_F491_4F6C_DD1D,
    ] {
        let mut rng = Rng(seed);
        for step in 0..40 {
            let day = 10 + u32::try_from(rng.below(3)).unwrap();
            let date = format!("{day:02}-10-2026");
            if rng.below(3) == 0 {
                let (account, key) = keys[usize::try_from(rng.below(4)).unwrap()];
                let amount = 100 + rng.below(20_000);
                receipt_on(&w, account, key, &amount.to_string(), (2026, 10, day));
            } else {
                let lines = 1 + rng.below(3);
                let qty_each = 1 + rng.below(8);
                let mut sales = Vec::new();
                for j in 0..lines {
                    let which = rng.below(4);
                    let rate = (100 + rng.below(900)).to_string();
                    let mut line = sale(
                        &format!("s-{j}"),
                        [w.akram, w.arshad, w.pukhta_buyer, w.pukhta_buyer]
                            [usize::try_from(which).unwrap()],
                        &qty_each.to_string(),
                        &rate,
                    );
                    if which >= 2 {
                        let (ur, en, phone) = if which == 2 {
                            ("شاہد", "Shahid", "0300-0000000")
                        } else {
                            ("ناصر", "Nasir", "0300-1111111")
                        };
                        line.pukhta = person(ur, en, phone);
                    }
                    line.payment_type = Some(if rng.below(2) == 0 {
                        PaymentType::Cash
                    } else {
                        PaymentType::Credit
                    });
                    sales.push(line);
                }
                let total = (lines * qty_each).to_string();
                let mut d = draft(vec![with_sales(
                    bare_lot(&w, "lot-1", w.rahim, w.banana, &total),
                    sales,
                )]);
                d.header.entry_date = date;
                save_ok(&w, &d);
            }
            for (account, key) in keys {
                let again = w
                    .env
                    .ctx
                    .db()
                    .write(|tx| {
                        matching::rematch(
                            tx,
                            w.owner.user_id,
                            "2026-10-12T09:00:00.000Z",
                            account,
                            key,
                        )
                    })
                    .unwrap();
                assert_eq!(
                    (again.switched_off, again.added),
                    (0, 0),
                    "seed {seed:x} step {step}: the full working changed the rows of {key:?}"
                );
                // The rows never take more than a line has, no old bill waits next to an unmatched
                // credit, and the two add up to the balance.
                let (open_bills, unmatched, balance) = key_state(&w, account, key);
                assert!(
                    open_bills >= 0 && unmatched >= 0,
                    "seed {seed:x} step {step}"
                );
                assert!(
                    open_bills == 0 || unmatched == 0,
                    "seed {seed:x} step {step}"
                );
                assert_eq!(open_bills - unmatched, balance, "seed {seed:x} step {step}");
            }
        }
    }
}

#[test]
fn a_corrected_opening_balance_of_a_buyer_re_matches_his_key() {
    // Money agent P1: opening credit 5,000, then a credit sale of 12,880 (5,000 matched). The
    // Owner corrects the opening to 3,000: 3,000 matches the oldest bill, and the next sale
    // (856) is a new open bill behind it.
    let w = world();
    open_account(&w, w.akram, "", "5,000");
    let first = save_ok(&w, &credit_sale(&w, w.akram));
    let bill = line_id(&w, first.entry_id, w.akram, "dr");
    assert_eq!(
        rows(&w, "bill_allocations")
            .iter()
            .map(|r| r.2)
            .sum::<i64>(),
        5_000
    );
    open_account(&w, w.akram, "", "3,000");
    let credit = line_id(&w, latest_opening_entry(&w, w.akram), w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(bill, credit, 3_000)]);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 9_880));
    let small = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "1"),
        vec![sale("s-1", w.akram, "1", "800")],
    )]);
    save_ok(&w, &small);
    assert_eq!(rows(&w, "bill_allocations"), [(bill, credit, 3_000)]);
    let again = w
        .env
        .ctx
        .db()
        .write(|tx| {
            matching::rematch(
                tx,
                w.owner.user_id,
                "2026-10-10T09:00:00.000Z",
                w.akram,
                None,
            )
        })
        .unwrap();
    assert_eq!((again.switched_off, again.added), (0, 0));
}

#[test]
fn a_first_opening_credit_of_a_buyer_with_bills_settles_the_oldest_bill() {
    let w = world();
    let first = save_ok(&w, &credit_sale(&w, w.akram));
    let bill = line_id(&w, first.entry_id, w.akram, "dr");
    assert!(rows(&w, "bill_allocations").is_empty());
    open_account(&w, w.akram, "", "5,000");
    let credit = line_id(&w, latest_opening_entry(&w, w.akram), w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(bill, credit, 5_000)]);
}

fn latest_opening_entry(w: &World, account: i64) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT MAX(e.id) FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id
             WHERE e.posting_type = 'opening' AND l.account_id = ?1",
            [account],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn a_corrected_opening_advance_of_a_seller_adjusts_again() {
    // Money agent P2: opening advance 15,000, Event 1 adjusts 15,000. The Owner corrects the
    // advance to 10,000: 10,000 is adjusted, nothing is open, and the payable is 24,316 - 5,000
    // - 10,000 = 9,316 جمع, which is رحیم's ledger.
    let w = world();
    open_advance(&w, w.rahim, "15,000");
    save_ok(&w, &event1(&w));
    assert_eq!(adjusted(&w), 15_000);
    open_advance(&w, w.rahim, "10,000");
    let advance_line = line_id(&w, latest_opening_entry(&w, w.rahim), w.rahim, "dr");
    let adjusting = rows(&w, "advance_allocations");
    assert_eq!(adjusting.len(), 1, "{adjusting:?}");
    assert_eq!((adjusting[0].0, adjusting[0].2), (advance_line, 10_000));
    assert_eq!(adjusted(&w), 10_000);
    assert_eq!(
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa,
        Paisa::ZERO
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 9_316));
    // A raise to 20,000 leaves 5,000 of the advance open and adjusts the whole credit 24,316?
    // No: the credit is 24,316 and the advance 20,000, so all 20,000 is adjusted.
    open_advance(&w, w.rahim, "20,000");
    assert_eq!(adjusted(&w), 20_000);
    // The advance of a seller who ticked off stays open.
    let w2 = world();
    open_advance(&w2, w2.rahim, "15,000");
    let mut off = event1(&w2);
    off.lots[0].adjust_advance = false;
    save_ok(&w2, &off);
    open_advance(&w2, w2.rahim, "12,000");
    assert_eq!(adjusted(&w2), 0);
}

#[test]
fn an_opening_debit_saved_late_is_still_the_oldest_bill_of_its_day() {
    // The opening of a day is the balance the day starts with, so a receipt pays it before a
    // sale of the same day, even when the opening was entered (or corrected) after the sale.
    let w = world();
    save_ok(&w, &credit_sale(&w, w.akram));
    open_account(&w, w.akram, "5,000", "");
    let opening = line_id(&w, latest_opening_entry(&w, w.akram), w.akram, "dr");
    let entry = receipt(&w, w.akram, None, "5,000");
    let paid = line_id(&w, entry, w.akram, "cr");
    assert_eq!(rows(&w, "bill_allocations"), [(opening, paid, 5_000)]);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 12_880));
}

#[test]
fn an_unrelated_opening_never_makes_a_credit_take_a_later_advance() {
    // Money agent P6 (PRD FR-15.8, "each later seller credit"): Event 1 with the tick on and no
    // advance, then 15,000 is given as an advance (a later line of the same day), then an
    // opening credit of 1,000 on the seller re-matches him. The credit of Event 1 comes before
    // the advance, so nothing is adjusted, now or after any re-match.
    let w = world();
    save_ok(&w, &event1(&w));
    give_advance(&w, w.rahim, "15,000");
    assert_eq!(rows(&w, "advance_allocations"), []);
    open_account(&w, w.rahim, "", "1,000");
    assert_eq!(rows(&w, "advance_allocations"), []);
    assert_eq!(
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa,
        Paisa::from_rupees(15_000).unwrap()
    );
}

#[test]
fn a_back_dated_bikri_never_takes_an_advance_given_after_its_day() {
    // Today is 11-10. The advance is given on 11-10, then Event 1 is saved with the date
    // 10-10 and the tick on. The credit is older than the advance, so it adjusts nothing: the
    // fast path does not apply (a later entry exists), and the full working agrees.
    let w = world();
    crate::common::set_today(&w.env, 2026, 10, 11);
    give_advance_on(&w, w.rahim, "15,000", "11-10-2026");
    save_ok(&w, &event1(&w));
    assert_eq!(rows(&w, "advance_allocations"), []);
    // The same working after an unrelated opening gives the same answer.
    open_account(&w, w.rahim, "", "1,000");
    assert_eq!(rows(&w, "advance_allocations"), []);
    assert_eq!(adjusted(&w), 0);
}

#[test]
fn an_advance_entered_late_is_adjusted_by_the_credit_after_it() {
    // Money agent third review, blocker 1. Today is 11-10. On 11-10 a بکری (tick on, no advance
    // yet) credits 24,316 and adjusts nothing. Then the Owner enters an advance of 15,000
    // dated 10-10, which he forgot yesterday. The credit comes after the advance, so it adjusts
    // 15,000 at once: advance open 0, payable 24,316 - 5,000 - 15,000 = 4,316.
    let w = world();
    crate::common::set_today(&w.env, 2026, 10, 11);
    let mut newer = event1(&w);
    newer.header.entry_date = "11-10-2026".into();
    newer.header.bill_date = "11-10-2026".into();
    save_ok(&w, &newer);
    assert_eq!(adjusted(&w), 0);
    give_advance_on(&w, w.rahim, "15,000", "10-10-2026");
    assert_eq!(adjusted(&w), 15_000);
    assert_eq!(
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa,
        Paisa::ZERO
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_316));
    // The same answer after an unrelated opening re-matches the seller.
    open_account(&w, w.rahim, "", "1,000");
    assert_eq!(adjusted(&w), 15_000);
}

#[test]
fn an_advance_before_a_back_dated_bikri_is_still_adjusted() {
    // Today is 11-10. An advance of 15,000 on 10-10, a بکری dated 11-10 (the newest line, fast
    // path), then a بکری dated 10-10 for the same seller: the second credit is older than the
    // first, both come after the advance, and the oldest credit takes the advance first.
    let w = world();
    crate::common::set_today(&w.env, 2026, 10, 11);
    give_advance(&w, w.rahim, "15,000");
    let mut newer = event1(&w);
    newer.header.entry_date = "11-10-2026".into();
    newer.header.bill_date = "11-10-2026".into();
    save_ok(&w, &newer);
    assert_eq!(adjusted(&w), 15_000);
    save_ok(&w, &event1(&w));
    // The credit of the 10-10 بکری is older, so it takes the whole advance.
    assert_eq!(adjusted(&w), 15_000);
    let adjusting = rows(&w, "advance_allocations");
    assert_eq!(adjusting.len(), 1, "{adjusting:?}");
    let older_credit = scalar(
        &w,
        "SELECT l.id FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = (SELECT id FROM accounts WHERE code = '030110001')
           AND l.cr_paisa > 0 AND e.posting_type = 'bikri_first' AND e.entry_date = '2026-10-10'",
    );
    assert_eq!(adjusting[0].1, older_credit);
}

#[test]
fn a_lot_that_grew_by_a_later_sale_does_not_break_the_advance_working() {
    // Money agent finding 2: the credit of a lot's FIRST posting is worked out from the lines of
    // that posting, not from the lot's running totals, which later sales change. A later sale
    // is simulated here by adding to the running totals the way plan 14 will.
    let w = world();
    open_advance(&w, w.rahim, "15,000");
    save_ok(&w, &event1(&w));
    assert_eq!(adjusted(&w), 15_000);
    raw(&w.dir)
        .execute(
            "UPDATE bikri_lots SET gross_paisa = gross_paisa + 560000,
                    seller_comm_paisa = seller_comm_paisa + 33600,
                    deductions_paisa = deductions_paisa + 33600,
                    net_paisa = net_paisa + 526400",
            [],
        )
        .unwrap();
    open_advance(&w, w.rahim, "10,000");
    assert_eq!(adjusted(&w), 10_000);
}

fn adjusted(w: &World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE is_active = 1",
    ) / 100
}
