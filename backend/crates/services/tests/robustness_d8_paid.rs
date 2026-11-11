//! D8 end gate, attack on money paid on the spot (PRD FR-12.44, plan 65, plan 60 P1 to P14; Code Standards
//! 4.2 "Paid on the spot"; NFR-REL-1). Owner: `robustness`.
//!
//! 1. `hostile_text_in_the_paid_cell`: about 60 texts (decimals, negatives, sums, Urdu and other digits,
//!    zero-width and bidi characters, control characters, SQL, huge and very long numbers, one rupee over,
//!    exactly the total). Each is either worked out as a whole-rupee paid with paid + left = buyer total, or
//!    refused at `lots[0].sales[0].paid`. A refusal writes nothing. The same texts go through a paid-only edit.
//! 2. `a_buyer_who_must_pay_in_full_cannot_be_given_a_smaller_paid`: naqdi, چکتا and a no-credit buyer with
//!    typed 0, 5 and a huge number: paid is the buyer total, the cell is fixed.
//! 3. `the_edge_of_the_total_in_rupees`: total - 1, total, total + 1 on lines with a different buyer rate.
//! 4. `random_sales_edits_cancels_and_receipts_keep_the_books_exact`: a seeded run of 80 operations. After
//!    every one: the trial balance difference is 0, the audit chain verifies, every buyer's balance is the sum
//!    of what is left on his live lines less his receipts, and his matching rows are the canonical ones
//!    (matched = min(debits, credits), no line over-matched).
//! 5. `an_edit_whose_new_rate_makes_the_old_paid_too_big_is_refused_and_leaves_the_file_as_it_was`.
//! 6. `cancel_of_part_paid_lines_with_receipts_matched`.
//! 7. `a_legacy_part_paid_line_of_a_buyer_who_lost_credit_keeps_its_paid_until_the_line_changes`.
//! 8. `two_threads_save_the_same_draft_and_two_threads_edit_the_same_entry`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::{assert_chain_ok, raw};
use helpers::*;
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::bikri::{BikriDraft, BikriPreview, ChuktaInput};
use mandi_services::core::error::{ErrorCode, FieldError};
use mandi_services::{ServiceError, bikri, ledger};
use support::*;

/// A small deterministic generator (the tests of this crate use no external seed).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn one_line(w: &World, buyer: i64, qty: &str, rate: &str, paid: Option<&str>) -> BikriDraft {
    let mut s = sale("s-1", buyer, qty, rate);
    s.paid = paid.map(str::to_owned);
    draft(vec![with_sales(
        bare_lot(w, "lot-1", w.rahim, w.banana, qty),
        vec![s],
    )])
}

fn at_paid(errors: &[FieldError]) -> Vec<(ErrorCode, String)> {
    errors
        .iter()
        .filter(|e| e.field.ends_with(".paid"))
        .map(|e| (e.code, e.field.clone()))
        .collect()
}

/// The whole of a preview agrees with itself for every complete line: paid + left = buyer total, paid is
/// a whole number of rupees, not negative and not above the total, and the summary adds the lines up.
fn assert_line_figures(p: &BikriPreview, what: &str) {
    let mut paid_sum = 0_i64;
    let mut left_sum = 0_i64;
    let mut total_sum = 0_i64;
    for lot in &p.lots {
        for line in &lot.lines {
            if let Some(c) = &line.calc {
                let (paid, left, total) = (
                    c.paid_paisa.as_paisa(),
                    c.left_paisa.as_paisa(),
                    c.buyer_total.as_paisa(),
                );
                assert_eq!(paid + left, total, "{what}: paid + left != total");
                assert!(paid >= 0 && paid <= total, "{what}: paid {paid} of {total}");
                assert_eq!(paid % 100, 0, "{what}: paid is not whole rupees");
                assert_eq!(left % 100, 0, "{what}: left is not whole rupees");
                paid_sum += paid;
                left_sum += left;
                total_sum += total;
            }
        }
    }
    assert_eq!(p.summary.buyer_cash.as_paisa(), paid_sum, "{what}: cash");
    assert_eq!(
        p.summary.buyer_credit.as_paisa(),
        left_sum,
        "{what}: credit"
    );
    assert_eq!(p.summary.buyer_total.as_paisa(), total_sum, "{what}: total");
}

#[derive(Clone, Copy, Debug)]
enum Want {
    /// Worked out, with exactly this paid in rupees.
    Paid(i64),
    /// Refused at `.paid` with this code.
    Refused(ErrorCode),
    /// The spec leaves it open: worked out with exactly this paid, or refused at `.paid` (any code).
    PaidOrRefused(i64),
}

/// The texts. The line is 50 x 900: value 45,000, commission 2,700, لاگا 400, total 48,100.
fn hostile_cases() -> Vec<(String, Want)> {
    use ErrorCode::*;
    use Want::*;
    let long_digits = "9".repeat(100_000);
    let long_zeros = format!("{}5000", "0".repeat(5_000));
    let mut v: Vec<(String, Want)> = vec![
        ("".into(), Paid(0)),
        ("   ".into(), Paid(0)),
        ("0".into(), Paid(0)),
        ("00".into(), Paid(0)),
        ("10000".into(), Paid(10_000)),
        ("10,000".into(), Paid(10_000)),
        ("  10,000  ".into(), Paid(10_000)),
        ("10000\n".into(), Paid(10_000)),
        ("\t10000".into(), Paid(10_000)),
        ("48100".into(), Paid(48_100)),
        ("48,100".into(), Paid(48_100)),
        ("48101".into(), Refused(ErrPaidExceedsTotal)),
        ("48,101".into(), Refused(ErrPaidExceedsTotal)),
        ("48200".into(), Refused(ErrPaidExceedsTotal)),
        ("99999999".into(), Refused(ErrPaidExceedsTotal)),
        ("10000000000".into(), Refused(ErrPaidExceedsTotal)),
        ("10,00,00,00,000".into(), Refused(ErrPaidExceedsTotal)),
        ("10000000001".into(), Refused(ErrAmountTooLarge)),
        ("99999999999".into(), Refused(ErrAmountTooLarge)),
        (
            "99999999999999999999999999".into(),
            Refused(ErrAmountTooLarge),
        ),
        ("20+25".into(), Paid(45)),
        ("20000+20000".into(), Paid(40_000)),
        ("20000+30000".into(), Refused(ErrPaidExceedsTotal)),
        ("20000-30000".into(), Refused(ErrNegativeNotAllowed)),
        ("-1".into(), Refused(ErrNegativeNotAllowed)),
        ("-10,000".into(), Refused(ErrNegativeNotAllowed)),
        ("10000.5".into(), Refused(ErrDecimalsNotAllowed)),
        ("10000.50".into(), Refused(ErrDecimalsNotAllowed)),
        ("850.50".into(), Refused(ErrDecimalsNotAllowed)),
        ("0.5".into(), Refused(ErrDecimalsNotAllowed)),
        (".5".into(), Refused(ErrDecimalsNotAllowed)),
        ("5.".into(), Refused(ErrDecimalsNotAllowed)),
        ("10000.00".into(), Refused(ErrDecimalsNotAllowed)),
        ("1e5".into(), Refused(ErrNotANumber)),
        ("1E3".into(), Refused(ErrNotANumber)),
        ("20*2".into(), Refused(ErrNotANumber)),
        ("20x2".into(), Refused(ErrNotANumber)),
        ("20\u{d7}2".into(), Refused(ErrNotANumber)),
        ("20/2".into(), Refused(ErrNotANumber)),
        ("abc".into(), Refused(ErrNotANumber)),
        ("NaN".into(), Refused(ErrNotANumber)),
        ("inf".into(), Refused(ErrNotANumber)),
        ("Infinity".into(), Refused(ErrNotANumber)),
        ("0x1F".into(), Refused(ErrNotANumber)),
        ("10 rupees".into(), Refused(ErrNotANumber)),
        ("rs 5000".into(), Refused(ErrNotANumber)),
        (
            "'; DROP TABLE bikri_sales; --".into(),
            Refused(ErrNotANumber),
        ),
        ("' OR 1=1 --".into(), Refused(ErrNotANumber)),
        ("<script>alert(1)</script>".into(), Refused(ErrNotANumber)),
        ("..\\..\\windows".into(), Refused(ErrNotANumber)),
        ("5000\u{0}".into(), Refused(ErrNotANumber)),
        ("50\u{0}00".into(), Refused(ErrNotANumber)),
        ("5000\u{7}".into(), Refused(ErrNotANumber)),
        // The spec says Urdu digits are accepted and converted (Code Standards 4.2).
        ("\u{6f1}\u{6f0}\u{6f0}\u{6f0}\u{6f0}".into(), Paid(10_000)),
        ("\u{6f1}\u{6f0},\u{6f0}\u{6f0}\u{6f0}".into(), Paid(10_000)),
        // Other digit shapes: accepted with the right value, or refused. Never another value.
        (
            "\u{661}\u{660}\u{660}\u{660}\u{660}".into(),
            PaidOrRefused(10_000),
        ),
        (
            "\u{ff11}\u{ff10}\u{ff10}\u{ff10}\u{ff10}".into(),
            PaidOrRefused(10_000),
        ),
        ("\u{200b}5000".into(), PaidOrRefused(5_000)),
        ("50\u{200b}00".into(), PaidOrRefused(5_000)),
        ("5000\u{200d}".into(), PaidOrRefused(5_000)),
        ("5\u{202e}000".into(), PaidOrRefused(5_000)),
        ("\u{feff}5000".into(), PaidOrRefused(5_000)),
        ("\u{a0}5000\u{a0}".into(), PaidOrRefused(5_000)),
        ("+5000".into(), PaidOrRefused(5_000)),
        ("5_000".into(), PaidOrRefused(5_000)),
        ("5 000".into(), PaidOrRefused(5_000)),
        ("1,2,3".into(), PaidOrRefused(123)),
        (",5000".into(), PaidOrRefused(5_000)),
        ("5000,".into(), PaidOrRefused(5_000)),
        ("5,,000".into(), PaidOrRefused(5_000)),
        ("0000000000000000000005000".into(), PaidOrRefused(5_000)),
    ];
    v.push((long_digits, Want::Refused(ErrAmountTooLarge)));
    v.push((long_zeros, Want::PaidOrRefused(5_000)));
    v
}

fn judge(what: &str, want: Want, p: &BikriPreview) -> Option<i64> {
    let refused = at_paid(&p.errors);
    let c = p
        .lots
        .first()
        .and_then(|l| l.lines.first())
        .and_then(|l| l.calc.as_ref());
    match want {
        Want::Paid(r) => {
            assert!(refused.is_empty(), "{what}: refused {refused:?}");
            let c = c.unwrap_or_else(|| panic!("{what}: no figures"));
            assert_eq!(c.paid_paisa.rupees(), r, "{what}");
            assert_eq!(c.left_paisa.rupees(), 481 * 100 - r, "{what}");
            Some(r)
        }
        Want::Refused(code) => {
            assert_eq!(
                refused,
                vec![(code, "lots[0].sales[0].paid".to_owned())],
                "{what}: {:?}",
                p.errors
            );
            None
        }
        Want::PaidOrRefused(r) => {
            if refused.is_empty() {
                let c = c.unwrap_or_else(|| panic!("{what}: no figures"));
                assert_eq!(
                    c.paid_paisa.rupees(),
                    r,
                    "{what}: accepted with another value"
                );
                Some(r)
            } else {
                assert_eq!(refused.len(), 1, "{what}: {refused:?}");
                assert_eq!(refused[0].1, "lots[0].sales[0].paid", "{what}");
                None
            }
        }
    }
}

#[test]
fn hostile_text_in_the_paid_cell() {
    let w = world();
    let cases = hostile_cases();
    let mut accepted = 0;
    let mut saved_paid_total = 0_i64;
    let cash_before = tools::cash(&w);
    for (text, want) in &cases {
        let shown: String = text.chars().take(40).collect();
        let what = format!("paid {shown:?}");
        let d = one_line(&w, w.akram, "50", "900", Some(text));
        let before = snapshot(&raw(&w.dir));
        let started = std::time::Instant::now();
        let p = bikri::preview(&w.env.ctx, &d).unwrap_or_else(|e| panic!("{what}: preview {e:?}"));
        assert!(
            started.elapsed().as_millis() < 2_000,
            "{what}: the preview took {:?}",
            started.elapsed()
        );
        assert_eq!(snapshot(&raw(&w.dir)), before, "{what}: a preview wrote");
        assert_line_figures(&p, &what);
        let result = judge(&what, *want, &p);
        match result {
            Some(r) => {
                let saved = save(&w, &d).unwrap_or_else(|e| panic!("{what}: save {e:?}"));
                assert_eq!(
                    saved.preview, p,
                    "{what}: the save differs from its preview"
                );
                let stored: i64 = raw(&w.dir)
                    .query_row(
                        "SELECT paid_paisa FROM bikri_sales WHERE bikri_id = ?1",
                        [saved.bikri_id],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert_eq!(stored, r * 100, "{what}: stored paid");
                accepted += 1;
                saved_paid_total += r;
            }
            None => {
                let e = save(&w, &d).expect_err(&what);
                assert!(
                    matches!(e, ServiceError::Rejected(_)),
                    "{what}: not a refusal: {e:?}"
                );
                assert_eq!(
                    snapshot(&raw(&w.dir)),
                    before,
                    "{what}: a refused save wrote something"
                );
            }
        }
    }
    // The tables are still there after the SQL texts, the books balance, the chain is whole.
    assert!(accepted >= 20, "only {accepted} texts were accepted");
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
    // Cash: every accepted paid came in, and nothing else moved it (the lots carry no نقد).
    assert_eq!(tools::cash(&w), cash_before + saved_paid_total);
    let integrity: String = raw(&w.dir)
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn the_same_hostile_texts_in_a_paid_only_edit() {
    let w = world();
    let saved = save_ok(&w, &one_line(&w, w.akram, "50", "900", None));
    let mut entry_id = saved.entry_id;
    let mut expected_cash = tools::cash(&w);
    let mut expected_paid = 0_i64;
    for (text, want) in &hostile_cases() {
        let shown: String = text.chars().take(40).collect();
        let what = format!("edit paid {shown:?}");
        let opened = tools::open(&w, saved.bikri_id, entry_id);
        let mut d = opened.draft.clone();
        d.lots[0].sales[0].paid = Some(text.clone());
        let before = tools::fingerprint(&raw(&w.dir));
        let p = bikri::edit_preview(
            &w.env.ctx,
            &mandi_services::core::bikri::EditPreviewInput {
                entry_id,
                draft: d.clone(),
            },
        )
        .unwrap_or_else(|e| panic!("{what}: {e:?}"));
        assert_line_figures(&p, &what);
        match judge(&what, *want, &p) {
            Some(r) => {
                let edited = tools::try_edit(&w, &opened, &d, true)
                    .unwrap_or_else(|e| panic!("{what}: edit {e:?}"));
                assert_eq!(edited.preview, p, "{what}: save differs from its preview");
                entry_id = edited.entry_id;
                expected_cash += r - expected_paid;
                expected_paid = r;
                assert_eq!(tools::cash(&w), expected_cash, "{what}: cash");
            }
            None => {
                let e = tools::try_edit(&w, &opened, &d, true).expect_err(&what);
                assert!(matches!(e, ServiceError::Rejected(_)), "{what}: {e:?}");
                assert_eq!(
                    tools::fingerprint(&raw(&w.dir)),
                    before,
                    "{what}: a refused edit changed the file"
                );
            }
        }
    }
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
    // One live line, paid as the last accepted text said.
    let (live, paid): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(paid_paisa), 0) FROM bikri_sales WHERE is_active = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(live, 1);
    assert_eq!(paid, expected_paid * 100);
}

#[test]
fn a_buyer_who_must_pay_in_full_cannot_be_given_a_smaller_paid() {
    let w = world();
    let no_credit = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "نقد والا",
        "Cash Only",
        false,
        "",
    );
    // (buyer, chukta person)
    let buyers: Vec<(i64, Option<ChuktaInput>, &str)> = vec![
        (w.naqdi, None, "naqdi"),
        (
            w.chukta_khata,
            person("شاہد", "Shahid", "0300-0000000"),
            "chukta",
        ),
        (no_credit, None, "no credit"),
    ];
    for (buyer, chukta, name) in &buyers {
        for typed in ["0", "5", "48099", "999999999999", "", "20+25"] {
            let mut s = sale("s-1", *buyer, "50", "900");
            s.chukta = chukta.clone();
            s.paid = if typed.is_empty() {
                None
            } else {
                Some(typed.to_owned())
            };
            let d = draft(vec![with_sales(
                bare_lot(&w, "lot-1", w.rahim, w.banana, "50"),
                vec![s],
            )]);
            let p = preview(&w, &d);
            let what = format!("{name} typed {typed:?}");
            if typed == "999999999999" {
                // Tampered input above the largest allowed amount: ignored or refused at .paid, never kept.
                if !p.errors.is_empty() {
                    assert_eq!(
                        at_paid(&p.errors).len(),
                        p.errors.len(),
                        "{what}: {:?}",
                        p.errors
                    );
                    assert!(save(&w, &d).is_err());
                    continue;
                }
            }
            assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
            let c = calc(&p.lots[0], 0);
            assert!(c.paid_fixed, "{what}: the cell is not fixed");
            assert_eq!(c.paid_paisa, c.buyer_total, "{what}");
            assert_eq!(r(c.left_paisa), 0, "{what}");
            let saved = save_ok(&w, &d);
            let stored: (i64, i64) = raw(&w.dir)
                .query_row(
                    "SELECT paid_paisa, buyer_total_paisa FROM bikri_sales WHERE bikri_id = ?1",
                    [saved.bikri_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(stored.0, stored.1, "{what}: stored paid is not the total");
        }
    }
    // Nobody on those accounts owes anything, and 0030 ends at 0.
    for account in [w.naqdi, w.chukta_khata, no_credit] {
        let (_, amount) = balance(&w, account);
        assert_eq!(amount, 0);
    }
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
}

#[test]
fn the_edge_of_the_total_in_rupees() {
    let w = world();
    // Different quantities, rates and a different buyer rate: the total moves, the edge moves with it.
    for (qty, rate, buyer_rate) in [
        ("1", "1", None),
        ("7", "333", None),
        ("50", "900", Some("910")),
        ("1000", "1", None),
        ("13", "97", Some("101")),
    ] {
        let mut s = sale("s-1", w.akram, qty, rate);
        if let Some(b) = buyer_rate {
            s.rate_differs = true;
            s.buyer_rate = Some(b.to_owned());
        }
        let probe = draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.rahim, w.banana, qty),
            vec![s.clone()],
        )]);
        let total = r(calc(&preview(&w, &probe).lots[0], 0).buyer_total);
        for (delta, ok) in [(-1_i64, true), (0, true), (1, false), (100, false)] {
            let typed = (total + delta).max(0).to_string();
            let mut line = s.clone();
            line.paid = Some(typed.clone());
            let d = draft(vec![with_sales(
                bare_lot(&w, "lot-1", w.rahim, w.banana, qty),
                vec![line],
            )]);
            let p = preview(&w, &d);
            let what = format!("{qty} x {rate}: total {total}, paid {typed}");
            if ok {
                assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
                let c = calc(&p.lots[0], 0);
                assert_eq!(r(c.paid_paisa), total + delta, "{what}");
                assert_eq!(r(c.left_paisa), -delta, "{what}");
            } else {
                assert_eq!(
                    at_paid(&p.errors),
                    vec![(
                        ErrorCode::ErrPaidExceedsTotal,
                        "lots[0].sales[0].paid".to_owned()
                    )],
                    "{what}"
                );
                assert_eq!(
                    tools::number(&p.errors, ErrorCode::ErrPaidExceedsTotal, "max"),
                    Some(total),
                    "{what}: max"
                );
                // The over-paid line counts as paid 0: no cash is inflated by a refused line.
                assert_eq!(r(p.summary.buyer_cash), 0, "{what}");
                assert!(save(&w, &d).is_err(), "{what}: saved");
            }
        }
    }
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
}

// ---------------------------------------------------------------------------------------------------
// The books after any sequence

/// What every buyer owes by his live lines, and what the matching rows say.
fn assert_books(w: &World, buyers: &[i64], what: &str) {
    assert_eq!(tools::trial_difference(w), 0, "{what}: trial balance");
    assert_chain_ok(&w.dir);
    let conn = raw(&w.dir);
    for &b in buyers {
        let (dr, cr): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND e.status = 'posted'",
                [b],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let (side, amount) = balance(w, b);
        let signed = match side {
            BalanceSide::Cr => -amount,
            _ => amount,
        };
        assert_eq!(signed * 100, dr - cr, "{what}: balance of account {b}");
        // What he owes from his lines less all his credits other than the cash of his own lines.
        let left: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(s.buyer_total_paisa - s.paid_paisa), 0)
                 FROM bikri_sales s JOIN bikri_entries e ON e.id = s.bikri_id
                 WHERE s.buyer_account_id = ?1 AND s.is_active = 1 AND e.status = 'saved'",
                [b],
                |r| r.get(0),
            )
            .unwrap();
        let receipts: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM vouchers
                 WHERE account_id = ?1 AND type = 'receipt' AND status = 'saved'",
                [b],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            dr - cr,
            left - receipts,
            "{what}: account {b}: balance != left on lines - receipts"
        );
        // Matching works on the live lines only: an entry that was reversed and its reversal are left out.
        let (ldr, lcr): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND e.status = 'posted'
                   AND e.reverses_entry_id IS NULL
                   AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)",
                [b],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        // Matching: matched = min(live debits, live credits); no bill and no payment line over-matched.
        let matched: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations
                 WHERE account_id = ?1 AND is_active = 1",
                [b],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(matched, ldr.min(lcr), "{what}: account {b}: matched amount");
        let over_bill: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                   SELECT a.bill_line_id, SUM(a.amount_paisa) AS m FROM bill_allocations a
                   WHERE a.account_id = ?1 AND a.is_active = 1 GROUP BY a.bill_line_id) x
                 JOIN journal_lines l ON l.id = x.bill_line_id WHERE x.m > l.dr_paisa",
                [b],
                |r| r.get(0),
            )
            .unwrap();
        let over_pay: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                   SELECT a.payment_line_id, SUM(a.amount_paisa) AS m FROM bill_allocations a
                   WHERE a.account_id = ?1 AND a.is_active = 1 GROUP BY a.payment_line_id) x
                 JOIN journal_lines l ON l.id = x.payment_line_id WHERE x.m > l.cr_paisa",
                [b],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            (over_bill, over_pay),
            (0, 0),
            "{what}: account {b}: over-matched"
        );
        let (open, unmatched) = tools::open_bills(w, b);
        assert_eq!(
            (open * 100, unmatched * 100),
            (ldr - matched, lcr - matched),
            "{what}: account {b}: open bills / unmatched credit"
        );
    }
}

#[test]
fn random_sales_edits_cancels_and_receipts_keep_the_books_exact() {
    let w = world_with(0, true);
    let extra = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق محمود",
        "Tariq Mehmood",
        true,
        "",
    );
    let buyers = [w.akram, w.arshad, extra];
    let mut rng = Rng(0xD8_0065);
    let mut live: Vec<(i64, i64)> = Vec::new(); // (bikri id, entry id)
    let mut count = [0_u32; 6]; // saved, save refused, edited, edit refused, cancelled, receipts
    for step in 0..80 {
        let what = format!("step {step}");
        let kind = rng.below(10);
        if kind < 5 || live.is_empty() {
            // A new بکری of 1 or 2 lots with 1 to 4 lines each; paid random, sometimes over the total.
            let mut lots = Vec::new();
            for l in 0..(1 + rng.below(2)) {
                let n = 1 + rng.below(4) as usize;
                let qtys: Vec<i64> = (0..n).map(|_| 1 + rng.below(20) as i64).collect();
                let received: i64 = qtys.iter().sum();
                let sales: Vec<_> = qtys
                    .iter()
                    .enumerate()
                    .map(|(i, q)| {
                        let buyer = buyers[rng.below(3) as usize];
                        let rate = 100 + rng.below(1_500);
                        let mut s = sale(
                            &format!("s-{l}-{i}"),
                            buyer,
                            &q.to_string(),
                            &rate.to_string(),
                        );
                        // The true total less a rupee or two: value + 6% (rounded down here) + 8 a نگ.
                        let total = (q * (rate as i64) * 106 / 100 + q * 8 - 2).max(1);
                        s.paid = match rng.below(10) {
                            0..=2 => None,
                            3 => Some(total.to_string()),
                            4..=6 => Some((rng.below(total as u64 + 1)).to_string()),
                            7 => Some((total + 3 + rng.below(50) as i64).to_string()),
                            _ => Some((rng.below(total as u64 / 2 + 1) / 100 * 100).to_string()),
                        };
                        s
                    })
                    .collect();
                lots.push(with_sales(
                    bare_lot(
                        &w,
                        &format!("lot-{l}"),
                        w.rahim,
                        w.banana,
                        &received.to_string(),
                    ),
                    sales,
                ));
            }
            let d = draft(lots);
            let p = preview(&w, &d);
            assert_line_figures(&p, &what);
            match save(&w, &d) {
                Ok(saved) => {
                    assert!(p.errors.is_empty(), "{what}");
                    assert_eq!(saved.preview, p, "{what}: save differs from preview");
                    live.push((saved.bikri_id, saved.entry_id));
                    count[0] += 1;
                }
                Err(e) => {
                    count[1] += 1;
                    assert!(
                        !p.errors.is_empty(),
                        "{what}: refused with a clean preview: {e:?}"
                    );
                    assert!(
                        p.errors
                            .iter()
                            .all(|e| e.code == ErrorCode::ErrPaidExceedsTotal),
                        "{what}: {:?}",
                        p.errors
                    );
                }
            }
        } else if kind < 7 {
            // A paid-only edit of one live entry: clear, set to the total, set to a part, or over.
            let index = rng.below(live.len() as u64) as usize;
            let (bikri_id, entry_id) = live[index];
            let opened = tools::open(&w, bikri_id, entry_id);
            let mut d = opened.draft.clone();
            let lot = rng.below(d.lots.len() as u64) as usize;
            let line = rng.below(d.lots[lot].sales.len() as u64) as usize;
            let total =
                r(calc(&tools::preview(&w, &opened, &opened.draft).lots[lot], line).buyer_total);
            d.lots[lot].sales[line].paid = match rng.below(4) {
                0 => None,
                1 => Some(total.to_string()),
                2 => Some((total + 1).to_string()),
                _ => Some((rng.below(total as u64 + 1)).to_string()),
            };
            let p = tools::preview(&w, &opened, &d);
            assert_line_figures(&p, &what);
            match tools::try_edit(&w, &opened, &d, true) {
                Ok(done) => {
                    assert!(p.errors.is_empty(), "{what}");
                    assert_eq!(done.preview, p, "{what}: edit differs from preview");
                    live[index] = (bikri_id, done.entry_id);
                    count[2] += 1;
                }
                Err(_) => {
                    count[3] += 1;
                    assert!(
                        !p.errors.is_empty(),
                        "{what}: edit refused with a clean preview"
                    );
                }
            }
        } else if kind < 8 && live.len() > 1 {
            let index = rng.below(live.len() as u64) as usize;
            let (bikri_id, _) = live.remove(index);
            tools::cancel(&w, bikri_id, true).unwrap_or_else(|e| panic!("{what}: cancel {e:?}"));
            count[4] += 1;
        } else {
            // A receipt from a buyer, at most what he owes (an open bill) or a small amount.
            let b = buyers[rng.below(3) as usize];
            let (open, _) = tools::open_bills(&w, b);
            if open > 0 {
                let amount = 1 + rng.below(open as u64);
                tools::receipt(&w, b, &amount.to_string());
                count[5] += 1;
            }
        }
        assert_books(&w, &buyers, &what);
    }
    assert!(!live.is_empty());
    eprintln!(
        "random run: saved {} / save refused {} / edited {} / edit refused {} / cancelled {} / receipts {}",
        count[0], count[1], count[2], count[3], count[4], count[5]
    );
    assert!(
        count[0] >= 15 && count[2] >= 4 && count[4] >= 2 && count[5] >= 2,
        "the run did too little: {count:?}"
    );
}

#[test]
fn an_edit_whose_new_rate_makes_the_old_paid_too_big_is_refused_and_leaves_the_file_as_it_was() {
    let w = world();
    // 50 x 900 = 48,100 with 40,000 paid on the spot.
    let saved = save_ok(&w, &one_line(&w, w.akram, "50", "900", Some("40000")));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots[0].sales[0].paid.as_deref(), Some("40000"));
    let before = tools::fingerprint(&raw(&w.dir));
    // The rate falls to 100: the line is 5,000 + commission + لاگا, far below the 40,000 sent back.
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "100".into();
    let p = tools::preview(&w, &opened, &d);
    assert!(
        p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrPaidExceedsTotal && e.field == "lots[0].sales[0].paid"),
        "{:?}",
        p.errors
    );
    let e = tools::try_edit(&w, &opened, &d, true).expect_err("the edit must be refused");
    assert!(tools::refused(
        &e,
        ErrorCode::ErrPaidExceedsTotal,
        "lots[0].sales[0].paid"
    ));
    assert_eq!(
        tools::refusal_number(&e, ErrorCode::ErrPaidExceedsTotal, "max"),
        Some(r(calc(&p.lots[0], 0).buyer_total)),
    );
    assert_eq!(tools::fingerprint(&raw(&w.dir)), before);
    // The user fixes paid: it saves, and the books are exact.
    d.lots[0].sales[0].paid = Some("2000".into());
    let done = tools::edit(&w, &opened, &d);
    assert_eq!(r(done.preview.summary.buyer_cash), 2_000);
    assert_books(&w, &[w.akram], "after the corrected edit");
    // Qty down by a half with paid unchanged at the old total: the same refusal.
    let opened = tools::open(&w, saved.bikri_id, done.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].paid = Some("40000".into());
    d.lots[0].sales[0].qty = "10".into();
    d.lots[0].qty_received = "10".into();
    let e = tools::try_edit(&w, &opened, &d, true).expect_err("refused");
    assert!(tools::refused(
        &e,
        ErrorCode::ErrPaidExceedsTotal,
        "lots[0].sales[0].paid"
    ));
}

#[test]
fn cancel_of_part_paid_lines_with_receipts_matched() {
    let w = world();
    let cash0 = tools::cash(&w);
    // ارشد: 3 lines with paid 0, part and all. اکرم: one part-paid line.
    let mut s1 = sale("a", w.arshad, "10", "600");
    s1.paid = None;
    let mut s2 = sale("b", w.arshad, "10", "600");
    s2.paid = Some("3000".into());
    let mut s3 = sale("c", w.arshad, "10", "600");
    s3.paid = Some("6,000".into());
    let mut s4 = sale("d", w.akram, "20", "600");
    s4.paid = Some("5000".into());
    let d = draft(vec![
        with_sales(
            bare_lot(&w, "lot-1", w.rahim, w.banana, "30"),
            vec![s1, s2, s3],
        ),
        with_sales(bare_lot(&w, "lot-2", w.karim, w.apple, "20"), vec![s4]),
    ]);
    let saved = save_ok(&w, &d);
    assert_books(&w, &[w.akram, w.arshad], "after the sale");
    // Receipts on the left parts, spanning bills.
    let (open, _) = tools::open_bills(&w, w.arshad);
    tools::receipt(&w, w.arshad, &(open - 100).to_string());
    tools::receipt(&w, w.akram, "2000");
    assert_books(&w, &[w.akram, w.arshad], "after the receipts");
    // Cancel the whole بکری: the paid cash goes back out, the receipts stay as credit, the books balance.
    let cancelled = tools::cancel(&w, saved.bikri_id, true).unwrap();
    let _ = cancelled;
    assert_books(&w, &[w.akram, w.arshad], "after the cancel");
    // Only the receipts remain: cash is cash0 + receipts; both buyers hold their receipt as credit.
    assert_eq!(tools::cash(&w), cash0 + (open - 100) + 2_000);
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Cr, open - 100));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 2_000));
    assert_eq!(tools::open_bills(&w, w.arshad), (0, open - 100));
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bill_allocations WHERE is_active = 1"
        ),
        0,
        "a cancelled sale left a live matching row"
    );
    // A second cancel is refused and changes nothing.
    let before = tools::fingerprint(&raw(&w.dir));
    assert!(tools::cancel(&w, saved.bikri_id, true).is_err());
    assert_eq!(tools::fingerprint(&raw(&w.dir)), before);
    // A new sale for the same buyers takes the credit.
    save_ok(&w, &one_line(&w, w.arshad, "5", "600", Some("100")));
    assert_books(&w, &[w.akram, w.arshad], "after the next sale");
}

#[test]
fn a_legacy_part_paid_line_of_a_buyer_who_lost_credit_keeps_its_paid_until_the_line_changes() {
    let w = world();
    let tariq = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق محمود",
        "Tariq Mehmood",
        true,
        "",
    );
    // Two lines for him: 10 x 600 paid 1,000 and 10 x 700 paid 0.
    let mut a = sale("a", tariq, "10", "600");
    a.paid = Some("1000".into());
    let b = sale("b", tariq, "10", "700");
    let saved = save_ok(
        &w,
        &draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.rahim, w.banana, "20"),
            vec![a, b],
        )]),
    );
    // He loses credit.
    lose_credit(&w, tariq, "طارق محمود", "Tariq Mehmood");
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    // An edit that changes only the second line's qty: line one is unchanged and keeps its 1,000.
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].qty = "9".into();
    let p = tools::preview(&w, &opened, &d);
    let c0 = calc(&p.lots[0], 0);
    assert_eq!(
        r(c0.paid_paisa),
        1_000,
        "an unchanged line changed its paid"
    );
    assert!(c0.paid_fixed);
    let c1 = calc(&p.lots[0], 1);
    assert_eq!(
        c1.paid_paisa, c1.buyer_total,
        "a changed line of a must-pay buyer is paid in full"
    );
    let done = tools::edit(&w, &opened, &d);
    assert_books(&w, &[tariq], "after the edit");
    let paids: Vec<i64> = raw(&w.dir)
        .prepare("SELECT paid_paisa / 100 FROM bikri_sales WHERE is_active = 1 ORDER BY line_no")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(paids[0], 1_000);
    assert_eq!(paids[1], r(calc(&done.preview.lots[0], 1).buyer_total));
}

#[test]
fn two_threads_save_the_same_draft_and_two_threads_edit_the_same_entry() {
    let w = world();
    let d = one_line(&w, w.akram, "50", "900", Some("10000"));
    // Two saves of the same draft at the same time: the backend has no request number (documented in
    // robustness_fault.rs), so both succeed as two بکری. They must not corrupt anything.
    let results: Vec<Result<_, ServiceError>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2).map(|_| scope.spawn(|| save(&w, &d))).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let ok = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 2, "{results:?}");
    let numbers: Vec<i64> = raw(&w.dir)
        .prepare("SELECT number FROM bikri_entries ORDER BY number")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(numbers, [1, 2], "numbers are not consecutive and unique");
    assert_books(&w, &[w.akram], "after two parallel saves");

    // Two parallel edits of the same entry with the same version: exactly one wins.
    let first = results[0].as_ref().unwrap();
    let opened = tools::open(&w, first.bikri_id, first.entry_id);
    let mut d1 = opened.draft.clone();
    d1.lots[0].sales[0].paid = Some("1000".into());
    let mut d2 = opened.draft.clone();
    d2.lots[0].sales[0].paid = Some("2000".into());
    let outcomes: Vec<Result<_, ServiceError>> = std::thread::scope(|scope| {
        let a = scope.spawn(|| tools::try_edit(&w, &opened, &d1, true));
        let b = scope.spawn(|| tools::try_edit(&w, &opened, &d2, true));
        vec![a.join().unwrap(), b.join().unwrap()]
    });
    let won = outcomes.iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        won, 1,
        "two edits of one version both saved or both failed: {outcomes:?}"
    );
    assert_books(&w, &[w.akram], "after two parallel edits");
    let live: i64 = scalar(
        &w,
        "SELECT COUNT(*) FROM bikri_sales s JOIN bikri_entries e ON e.id = s.bikri_id
         WHERE s.is_active = 1 AND e.number = 1",
    );
    assert_eq!(live, 1);

    // An edit racing a cancel: the book ends in one clean state either way.
    let second = results[1].as_ref().unwrap();
    let opened2 = tools::open(&w, second.bikri_id, second.entry_id);
    let mut d3 = opened2.draft.clone();
    d3.lots[0].sales[0].paid = Some("3000".into());
    let (edit_result, cancel_result) = std::thread::scope(|scope| {
        let a = scope.spawn(|| tools::try_edit(&w, &opened2, &d3, true));
        let b = scope.spawn(|| tools::cancel(&w, second.bikri_id, true));
        (a.join().unwrap(), b.join().unwrap())
    });
    assert!(
        edit_result.is_ok() || cancel_result.is_ok(),
        "neither won: {edit_result:?} / {cancel_result:?}"
    );
    assert_books(&w, &[w.akram], "after an edit racing a cancel");
    let _ = ledger::trial_balance;
}

/// Takes the right to buy on credit away from a buyer (the Owner edits his account).
fn lose_credit(w: &World, id: i64, ur: &str, en: &str) {
    let row = mandi_services::accounts::get(&w.env.ctx, id).unwrap();
    let input = mandi_services::core::accounts::AccountInput {
        id: Some(id),
        version: Some(row.version),
        account_type: AccountType::Buyer,
        control_id: None,
        code: row.code.clone(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: false,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    mandi_services::accounts::save(&w.env.ctx, &w.owner, &input).unwrap();
}
