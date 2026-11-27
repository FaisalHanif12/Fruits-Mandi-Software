//! Plans 76 and 77 end gate (PRD FR-12.17, FR-16.6, FR-16.13, FR-17.9, FR-19.11, FR-9.3; NFR-SPD-24,
//! NFR-REL-1, NFR-SCL-1; Code Standards 4.1 V0, B3, B31). Owner: `robustness`.
//!
//! Part A (every run, small data):
//!
//! * a mixed save (`bikri_mixed_save`) that fails after its first entry, in the middle or at the audit
//!   write, leaves no row behind and burns no number; the undo of a mixed save fails the same way;
//! * hostile dates, numbers and names in a mixed save are refused and write nothing;
//! * a link that cannot be made (wrong account, no partner, unknown partner, a balance that is not 0, a
//!   loan that is open, shares that are not 100%) comes back with its code and writes nothing;
//! * the new read commands (commission month, day, year and range, buyer dues, collection sheet,
//!   statement of a month and of a day, money lent dues, window bounds) refuse bad input with a code and
//!   never end in a database or data error, and a page is never longer than its cap.
//!
//! Part B (only with `MANDI_PERF_VOLUME=1`, release build): the paged reads of the two plans on 10 years of
//! data (the NFR-SCL-1 file of `pnpm perf`, plus the plan 76 tables made here by plain SQL: 1.2 million
//! sale facts, share rows of three partners, loans, drawings and a million audit rows). NFR-SPD-24: the
//! first page with its totals under 1 s, each next page under 300 ms. p95 of 9 runs after a warm-up.
//!
//! `MANDI_PERF_VOLUME=1 cargo test --release -p mandi-services --test robustness_plan76 -- --nocapture`
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "partners/support.rs"]
mod psupport;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
// The file already carries `#![allow(dead_code)]` at its top.
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use common::{raw, set_today};
use mandi_reports::ReportsError;
use mandi_reports::dashboard::{
    BuyerDuesInput, CommissionDayInput, CommissionMonthInput, CommissionParty, CommissionPeriod,
    CommissionPeriodKind, CommissionSide, buyer_dues, commission_day, commission_month,
};
use mandi_reports::reports::{CollectionSheetInput, collection_sheet};
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriSaveInput, CarriedLotInput, ChuktaInput, LotInput, MixedSaleInput, MixedSaleSaveInput,
    MixedUndoInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::khata::{
    KhataCreateInput, KhataCreateRequest, KhataOutInput, KhataSetLinkInput,
};
use mandi_services::core::lent::{
    LentDuesInput, LentListInput, LentPersonCreateInput, LentPersonCreateRequest,
    LentPersonUpdateInput, MoneyLentInput, MoneyLentRequest,
};
use mandi_services::core::links::{Link, LinkKind, PersonType};
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::core::statement::{PartnersStatementDayInput, PartnersStatementMonthInput};
use mandi_services::core::windows::{WindowSource, WindowsBoundsInput};
use mandi_services::db::rusqlite::Connection;
use mandi_services::{ServiceError, khata, ledger, lent, partners, vouchers, windows};
use support::World;

// ---- shared helpers ---------------------------------------------------------------------------------

/// The tables a save may write, counted before and after.
const TABLES: &[&str] = &[
    "journal_entries",
    "journal_lines",
    "bikri_entries",
    "bikri_lots",
    "bikri_sales",
    "analytics_facts",
    "partner_share_rows",
    "stock_moves",
    "audit_log",
    "bill_allocations",
    "advance_allocations",
    "lot_counters",
    "vouchers",
    "accounts",
    "chukta_buyers",
];

/// Row counts of every table above and four sums, so any leftover row or moved number shows.
fn snapshot(w: &World) -> Vec<(String, i64)> {
    let conn = raw(&w.dir);
    let mut out: Vec<(String, i64)> = TABLES
        .iter()
        .map(|table| {
            let n: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            ((*table).to_owned(), n)
        })
        .collect();
    for (name, sql) in [
        (
            "qty_sold",
            "SELECT COALESCE(SUM(qty_sold), 0) FROM bikri_lots",
        ),
        (
            "debits",
            "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines",
        ),
        (
            "credits",
            "SELECT COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
        ),
        (
            "last_number",
            "SELECT COALESCE(MAX(number), 0) FROM bikri_entries",
        ),
    ] {
        out.push((
            name.to_owned(),
            conn.query_row(sql, [], |r| r.get(0)).unwrap(),
        ));
    }
    out
}

fn code_name(error: &ServiceError) -> String {
    format!("{:?}", error.code())
}

fn is_refusal(error: &ServiceError) -> bool {
    matches!(error, ServiceError::Rejected(_))
}

fn reports_refusal(error: &ReportsError) -> bool {
    matches!(error, ReportsError::Rejected(_) | ReportsError::NotFound)
}

/// E2E-16: Event 1 with only 45 of the 50 sold, today 11-10-2026. Returns the carried lot's id.
fn e2e16(w: &World) -> i64 {
    let draft = support::draft(vec![support::with_sales(
        support::lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            support::sale("s-1", w.akram, "20", "600"),
            support::sale("s-2", w.arshad, "25", "580"),
        ],
    )]);
    let saved = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    bikri::get(&w.env.ctx, saved.bikri_id)
        .unwrap()
        .lots
        .first()
        .unwrap()
        .id
}

fn carried(w: &World, lot_id: i64) -> CarriedLotInput {
    CarriedLotInput {
        key: String::new(),
        lot_id,
        sales: vec![support::sale("c-1", w.arshad, "5", "560")],
    }
}

fn new_lot(w: &World) -> LotInput {
    let mut shahid = support::sale("n-1", w.chukta_khata, "10", "1200");
    shahid.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "new-1", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    support::with_sales(lot, vec![shahid])
}

fn mixed(w: &World, lot_id: i64) -> MixedSaleInput {
    MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![carried(w, lot_id)],
        new_lots: vec![new_lot(w)],
    }
}

fn save_input(input: &MixedSaleInput) -> MixedSaleSaveInput {
    MixedSaleSaveInput {
        date: input.date.clone(),
        carried: input.carried.clone(),
        new_lots: input.new_lots.clone(),
        confirm_cash_below_zero: false,
    }
}

// ---- part A1: a failing write in the middle of a mixed save -------------------------------------------

#[test]
fn a_mixed_save_that_fails_after_its_first_entry_leaves_nothing_and_burns_no_number() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = mixed(&w, lot_id);
    let before = snapshot(&w);
    let entries_before = before
        .iter()
        .find(|(name, _)| name == "journal_entries")
        .map(|(_, n)| *n)
        .unwrap();
    let conn = raw(&w.dir);
    // Three places to fail: the new بکری row, the second journal entry of the save, the audit row.
    let triggers = [
        (
            "robustness_no_new_bikri",
            "CREATE TRIGGER robustness_no_new_bikri BEFORE INSERT ON bikri_entries
             BEGIN SELECT RAISE(ABORT, 'robustness: write failed'); END;"
                .to_owned(),
        ),
        (
            "robustness_no_second_entry",
            format!(
                "CREATE TRIGGER robustness_no_second_entry BEFORE INSERT ON journal_entries
                 WHEN (SELECT COUNT(*) FROM journal_entries) >= {}
                 BEGIN SELECT RAISE(ABORT, 'robustness: write failed'); END;",
                entries_before + 1
            ),
        ),
        (
            "robustness_no_audit",
            "CREATE TRIGGER robustness_no_audit BEFORE INSERT ON audit_log
             BEGIN SELECT RAISE(ABORT, 'robustness: write failed'); END;"
                .to_owned(),
        ),
    ];
    for (name, create) in &triggers {
        conn.execute_batch(create).unwrap();
        let failed = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
        assert!(
            failed.is_err(),
            "{name}: the save went through a failing write"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{name}: a failed mixed save left rows behind"
        );
        conn.execute_batch(&format!("DROP TRIGGER {name};"))
            .unwrap();
    }
    // The same screen then saves, and its numbers are the ones a first try would have had.
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    // The hash chains of the journal and the audit log are whole after three rolled back saves and one
    // that went through (a cached head that outlived a rollback would break the next entry).
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    common::assert_chain_ok(&w.dir);
    assert_eq!(
        saved.new_bikri_number,
        Some(2),
        "a failed save burnt a number"
    );
    assert_eq!(saved.carried[0].old_number, 1);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr, "the books do not balance");
    let counters: i64 = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(MAX(last_counter), 0) FROM lot_counters WHERE seller_account_id = ?1",
            [w.karim],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(counters, 1, "a failed save burnt a lot counter of کریم");
}

#[test]
fn the_undo_of_a_mixed_save_that_fails_after_its_first_reversal_leaves_the_save_whole() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = mixed(&w, lot_id);
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    let before = snapshot(&w);
    let conn = raw(&w.dir);
    conn.execute_batch(
        "CREATE TRIGGER robustness_second_reversal BEFORE INSERT ON journal_entries
         WHEN NEW.posting_type = 'reversal'
          AND (SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'reversal') >= 1
         BEGIN SELECT RAISE(ABORT, 'robustness: write failed'); END;",
    )
    .unwrap();
    let undo = MixedUndoInput {
        later_entry_ids: vec![saved.carried[0].entry_id],
        new_bikri_id: saved.new_bikri_id,
        confirm_cash_below_zero: false,
    };
    let failed = bikri::mixed_undo(&w.env.ctx, &w.owner, &undo);
    assert!(failed.is_err(), "the undo went through a failing write");
    assert_eq!(
        snapshot(&w),
        before,
        "a failed undo left a half undo behind"
    );
    conn.execute_batch("DROP TRIGGER robustness_second_reversal;")
        .unwrap();
    let done = bikri::mixed_undo(&w.env.ctx, &w.owner, &undo).unwrap();
    assert_eq!(done.reversal_entry_ids.len(), 2);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    common::assert_chain_ok(&w.dir);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn the_same_mixed_save_sent_twice_posts_once() {
    let w = support::world();
    let lot_id = e2e16(&w);
    // Only the carried part: the 5 نگ are sold once; a double click finds nothing left.
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![carried(&w, lot_id)],
        new_lots: vec![],
    };
    let first = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
    assert!(first.is_ok());
    let after_first = snapshot(&w);
    let second = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
    assert!(second.is_err(), "the same 5 نگ were sold twice");
    assert_eq!(snapshot(&w), after_first, "the second save wrote rows");
}

// ---- part A2: hostile input to a mixed save ------------------------------------------------------------

#[test]
fn hostile_dates_and_numbers_in_a_mixed_save_are_refused_and_write_nothing() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let before = snapshot(&w);
    let long = "9".repeat(5_000);
    let dates = [
        "",
        " ",
        "31-02-2026",
        "12-10-2026",
        "99-99-9999",
        "01-01-0050",
        "0",
        "abc",
        "1e5",
        "-1",
        "11-10-2026'; DROP TABLE journal_entries; --",
        "11-10-2026\u{0}",
        long.as_str(),
    ];
    for date in dates {
        let mut input = mixed(&w, lot_id);
        input.date = date.to_owned();
        let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
        assert!(saved.is_err(), "date {date:?} was accepted");
        if let Err(error) = &saved {
            assert!(is_refusal(error), "date {date:?}: not a refusal: {error:?}");
        }
        // A preview never panics: a refusal, or an answer that lists its errors.
        if let Ok(preview) = bikri::mixed_preview(&w.env.ctx, &input) {
            assert!(
                !preview.preview.errors.is_empty(),
                "date {date:?}: the preview has no error"
            );
        }
    }
    let quantities = [
        "",
        " ",
        "0",
        "-1",
        "1e5",
        "5.5",
        "5,5",
        "99999999999999999999",
        "1000001",
        "6",
        "5+",
        "5x",
        "NaN",
        "inf",
        "0x5",
        "5 5",
    ];
    for qty in quantities {
        let mut input = mixed(&w, lot_id);
        if let Some(line) = input
            .carried
            .first_mut()
            .and_then(|lot| lot.sales.first_mut())
        {
            line.qty = qty.to_owned();
        }
        let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
        assert!(saved.is_err(), "qty {qty:?} was accepted");
    }
    let rates = [
        "",
        " ",
        "0",
        "-5",
        "850.50",
        "1e5",
        "10,00,00,00,001",
        "9999999999999999999999",
        "abc",
        "5-",
    ];
    for rate in rates {
        let mut input = mixed(&w, lot_id);
        if let Some(line) = input
            .new_lots
            .first_mut()
            .and_then(|lot| lot.sales.first_mut())
        {
            line.seller_rate = rate.to_owned();
        }
        let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input));
        assert!(saved.is_err(), "rate {rate:?} was accepted");
    }
    assert_eq!(snapshot(&w), before, "a refused save wrote something");
}

#[test]
fn hostile_walk_in_names_are_refused_or_stored_as_plain_clean_text() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let long_ur = "ش".repeat(5_000);
    let urdu_names = [
        "'; DROP TABLE accounts; --",
        "<script>alert(1)</script>",
        "..\\..\\windows",
        "\u{7}\u{0}",
        "\u{200b}\u{200b}",
        "   ",
        "ش\u{202e}ش",
        "%s%n",
        "ا\nب",
        long_ur.as_str(),
    ];
    for name in urdu_names {
        let mut input = mixed(&w, lot_id);
        if let Some(chukta) = input
            .new_lots
            .first_mut()
            .and_then(|lot| lot.sales.first_mut())
            .and_then(|line| line.chukta.as_mut())
        {
            chukta.name_ur = name.to_owned();
        }
        let before = snapshot(&w);
        match bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)) {
            Err(error) => {
                assert!(
                    is_refusal(&error),
                    "name {name:?}: not a refusal: {error:?}"
                );
                assert_eq!(snapshot(&w), before, "name {name:?}: a refusal wrote rows");
            }
            Ok(_) => {
                // Stored: as text, with no control or zero-width character, and every table still there.
                let conn = raw(&w.dir);
                let stored: Vec<String> = conn
                    .prepare("SELECT name_ur || name_en FROM chukta_buyers")
                    .unwrap()
                    .query_map([], |r| r.get(0))
                    .unwrap()
                    .map(Result::unwrap)
                    .collect();
                for text in stored {
                    assert!(
                        !text.chars().any(|c| c.is_control()
                            || matches!(c, '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}')),
                        "name {name:?}: control or zero-width character stored: {text:?}"
                    );
                }
                let tables: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('accounts', 'journal_entries')",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert_eq!(tables, 2, "name {name:?}: a table is gone");
                // One save went through: the lot is sold, so the loop ends here.
                return;
            }
        }
    }
}

// ---- part A3: links -------------------------------------------------------------------------------------

fn confirm_loop<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(text)) =
                        item.values.get("words")
                    {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn make_khata(w: &World, link: Option<Link>) -> i64 {
    confirm_loop(|words| {
        khata::create_linked(
            &w.env.ctx,
            &w.owner,
            &KhataCreateRequest {
                base: KhataCreateInput {
                    name_ur: "خاندانی کھاتہ".into(),
                    name_en: "Family Khata".into(),
                    note: None,
                    confirmed_rules_words: words.to_vec(),
                },
                link,
            },
        )
    })
    .account
    .id
}

fn make_person(w: &World, link: Option<Link>) -> i64 {
    confirm_loop(|words| {
        lent::person_create_linked(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateRequest {
                base: LentPersonCreateInput {
                    name_ur: "نواز".into(),
                    name_en: "Nawaz".into(),
                    phone: None,
                    staff_id: None,
                    confirmed_rules_words: words.to_vec(),
                },
                person_type: Some(PersonType::GardenOwner),
                link,
            },
        )
    })
    .account
    .id
}

fn shop() -> Link {
    Link {
        kind: LinkKind::Shop,
        partner_id: None,
    }
}

fn partner_link(partner: Option<i64>) -> Link {
    Link {
        kind: LinkKind::Partner,
        partner_id: partner,
    }
}

#[test]
fn a_link_that_cannot_be_made_comes_back_with_its_code_and_writes_nothing() {
    let w = support::world();
    let khata_id = make_khata(&w, None);
    let before = snapshot(&w);
    let set = |account: i64, link: Link| {
        khata::set_link(
            &w.env.ctx,
            &w.owner,
            &KhataSetLinkInput {
                khata_account_id: account,
                link,
            },
        )
    };
    // The wrong kind of account, and accounts that do not exist.
    for account in [w.akram, w.rahim, w.naqdi, w.chukta_khata, 0, -1, i64::MAX] {
        let refused = set(account, shop());
        assert!(refused.is_err(), "account {account} took a link");
        if let Err(error) = &refused {
            assert!(is_refusal(error), "account {account}: {error:?}");
        }
    }
    // A partner link with no partner, with an unknown partner, and a link that names a partner it must not.
    for link in [
        partner_link(None),
        partner_link(Some(999_999)),
        partner_link(Some(-1)),
        partner_link(Some(0)),
    ] {
        let refused = set(khata_id, link);
        assert!(refused.is_err(), "the link {link:?} was accepted");
        if let Err(error) = &refused {
            assert!(
                is_refusal(error),
                "the link {link:?} did not come back as a refusal: {error:?}"
            );
        }
    }
    assert_eq!(snapshot(&w), before, "a refused link wrote rows");

    // A shop link or no link that also names a partner: refused, or taken with the partner dropped. A
    // stored phantom partner on a shop link would make the partner's total wrong later.
    for kind in [LinkKind::Shop, LinkKind::None] {
        let odd = Link {
            kind,
            partner_id: Some(w.partners[0]),
        };
        if set(khata_id, odd).is_ok() {
            let stored: Option<i64> = raw(&w.dir)
                .query_row(
                    "SELECT link_partner_id FROM accounts WHERE id = ?1",
                    [khata_id],
                    |r| r.get(0),
                )
                .unwrap();
            println!("LENIENT: the link {odd:?} was accepted; stored partner {stored:?}");
            assert_eq!(stored, None, "a {kind:?} link kept a partner");
        }
    }
    // Back to no link, so the rest starts from a plain khata again.
    set(
        khata_id,
        Link {
            kind: LinkKind::None,
            partner_id: None,
        },
    )
    .unwrap();

    // A balance that is not 0 locks the link.
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &KhataOutInput {
            date: "10-10-2026".into(),
            khata_account_id: Some(khata_id),
            amount: "1000".into(),
            reason: "خرچ".into(),
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let after_out = snapshot(&w);
    let locked = set(khata_id, partner_link(Some(w.partners[0])));
    assert_eq!(locked.unwrap_err().code(), ErrorCode::ErrLinkLocked);
    assert_eq!(snapshot(&w), after_out, "a locked link wrote rows");

    // A person who owes keeps his link.
    let person = make_person(&w, Some(partner_link(Some(w.partners[1]))));
    vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &MoneyLentRequest {
            base: MoneyLentInput {
                date: "10-10-2026".into(),
                person_account_id: Some(person),
                amount: "500".into(),
                reason: "باغ".into(),
                return_by: None,
                narration: None,
                confirm_cash_below_zero: true,
            },
            link: None,
        },
    )
    .unwrap();
    let after_loan = snapshot(&w);
    for link in [
        shop(),
        partner_link(Some(w.partners[2])),
        Link {
            kind: LinkKind::None,
            partner_id: None,
        },
    ] {
        let refused = lent::person_update(
            &w.env.ctx,
            &w.owner,
            &LentPersonUpdateInput {
                person_account_id: person,
                person_type: None,
                link: Some(link),
            },
        );
        let error = refused.unwrap_err();
        assert!(
            code_name(&error).contains("FixedWhileOwed"),
            "the link of a person who owes changed or gave {}",
            code_name(&error)
        );
    }
    // Hostile person ids.
    for id in [0, -1, i64::MAX, w.akram] {
        let refused = lent::person_update(
            &w.env.ctx,
            &w.owner,
            &LentPersonUpdateInput {
                person_account_id: id,
                person_type: Some(PersonType::Other),
                link: None,
            },
        );
        assert!(refused.is_err(), "person {id} was updated");
        if let Err(error) = &refused {
            assert!(is_refusal(error), "person {id}: {error:?}");
        }
    }
    assert_eq!(
        snapshot(&w),
        after_loan,
        "a refused person update wrote rows"
    );
}

#[test]
fn a_shop_link_without_shares_that_total_100_is_refused() {
    let w = support::world_with(50_000, false);
    let khata_id = make_khata(&w, None);
    let before = snapshot(&w);
    let refused = khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: khata_id,
            link: shop(),
        },
    );
    let error = refused.unwrap_err();
    assert!(
        code_name(&error).contains("Shares"),
        "no shares and a shop link gave {}",
        code_name(&error)
    );
    assert_eq!(snapshot(&w), before);
}

// ---- part A4: the new read commands on bad input -------------------------------------------------------

fn day_period(date: Option<&str>) -> CommissionPeriod {
    CommissionPeriod {
        kind: CommissionPeriodKind::Day,
        date: date.map(str::to_owned),
        ..CommissionPeriod::default()
    }
}

fn range_period(from: Option<&str>, to: Option<&str>) -> CommissionPeriod {
    CommissionPeriod {
        kind: CommissionPeriodKind::Range,
        from: from.map(str::to_owned),
        to: to.map(str::to_owned),
        ..CommissionPeriod::default()
    }
}

fn year_period(year: Option<i32>) -> CommissionPeriod {
    CommissionPeriod {
        kind: CommissionPeriodKind::Year,
        year,
        ..CommissionPeriod::default()
    }
}

fn commission(
    month: Option<&str>,
    period: Option<CommissionPeriod>,
    party: Option<CommissionParty>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> CommissionMonthInput {
    CommissionMonthInput {
        month: month.map(str::to_owned),
        period,
        party,
        goods: None,
        offset,
        limit,
    }
}

#[test]
fn the_commission_page_refuses_bad_periods_and_parties_and_never_ends_in_a_data_error() {
    let w = support::world();
    let lot_id = e2e16(&w);
    bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&mixed(&w, lot_id))).unwrap();
    let db = w.env.ctx.db();
    // Today is 11-10-2026. Each of these must be refused.
    let long = "9".repeat(10_000);
    let refused: Vec<(String, CommissionMonthInput)> = vec![
        (
            "month in the future".into(),
            commission(Some("2026-11"), None, None, None, None),
        ),
        (
            "month 13".into(),
            commission(Some("2026-13"), None, None, None, None),
        ),
        (
            "month 0000-00".into(),
            commission(Some("0000-00"), None, None, None, None),
        ),
        (
            "month text".into(),
            commission(Some("abc"), None, None, None, None),
        ),
        (
            "month sql".into(),
            commission(Some("2026-10'; --"), None, None, None, None),
        ),
        (
            "month long".into(),
            commission(Some(long.as_str()), None, None, None, None),
        ),
        (
            "day missing".into(),
            commission(None, Some(day_period(None)), None, None, None),
        ),
        (
            "day empty".into(),
            commission(None, Some(day_period(Some(""))), None, None, None),
        ),
        (
            "day 31-02".into(),
            commission(None, Some(day_period(Some("31-02-2026"))), None, None, None),
        ),
        (
            "day future".into(),
            commission(None, Some(day_period(Some("12-10-2026"))), None, None, None),
        ),
        (
            "day text".into(),
            commission(None, Some(day_period(Some("abc"))), None, None, None),
        ),
        (
            "day sql".into(),
            commission(
                None,
                Some(day_period(Some("10-10-2026'; --"))),
                None,
                None,
                None,
            ),
        ),
        (
            "year missing".into(),
            commission(None, Some(year_period(None)), None, None, None),
        ),
        (
            "year 0".into(),
            commission(None, Some(year_period(Some(0))), None, None, None),
        ),
        (
            "year -1".into(),
            commission(None, Some(year_period(Some(-1))), None, None, None),
        ),
        (
            "year min".into(),
            commission(None, Some(year_period(Some(i32::MIN))), None, None, None),
        ),
        (
            "year 3000".into(),
            commission(None, Some(year_period(Some(3000))), None, None, None),
        ),
        (
            "year max".into(),
            commission(None, Some(year_period(Some(i32::MAX))), None, None, None),
        ),
        (
            "range reversed".into(),
            commission(
                None,
                Some(range_period(Some("10-10-2026"), Some("01-10-2026"))),
                None,
                None,
                None,
            ),
        ),
        (
            "range to in the future".into(),
            commission(
                None,
                Some(range_period(Some("01-10-2026"), Some("12-10-2026"))),
                None,
                None,
                None,
            ),
        ),
        (
            "range empty".into(),
            commission(None, Some(range_period(None, None)), None, None, None),
        ),
        (
            "range text".into(),
            commission(
                None,
                Some(range_period(Some("abc"), Some("10-10-2026"))),
                None,
                None,
                None,
            ),
        ),
        (
            "party that does not exist".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Buyer,
                    account_id: 999_999,
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
        (
            "party id 0".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Buyer,
                    account_id: 0,
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
        (
            "party id i64 max".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Seller,
                    account_id: i64::MAX,
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
        (
            "a seller read as a buyer".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Buyer,
                    account_id: w.rahim,
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
        (
            "a buyer read as a seller".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Seller,
                    account_id: w.akram,
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
        (
            "the cash account read as a buyer".into(),
            commission(
                None,
                None,
                Some(CommissionParty {
                    side: CommissionSide::Buyer,
                    account_id: support::id_of(&w.env, "01010001"),
                    chukta_person_key: None,
                }),
                None,
                None,
            ),
        ),
    ];
    let mut accepted: Vec<String> = Vec::new();
    for (what, input) in &refused {
        match commission_month(db, input) {
            Ok(_) => accepted.push(what.clone()),
            Err(error) => assert!(
                reports_refusal(&error),
                "{what}: not a refusal but {error:?}"
            ),
        }
    }
    assert!(accepted.is_empty(), "bad input was accepted: {accepted:?}");

    // A key on a buyer who is not a چکتا buyer must not widen the read to every chukta line.
    let odd = commission(
        None,
        None,
        Some(CommissionParty {
            side: CommissionSide::Buyer,
            account_id: w.akram,
            chukta_person_key: Some("x'; DROP TABLE analytics_facts; --".into()),
        }),
        None,
        None,
    );
    match commission_month(db, &odd) {
        Ok(page) => assert_eq!(
            page.totals.total_comm_paisa.as_paisa(),
            0,
            "a chukta key on an ordinary buyer read other people's lines"
        ),
        Err(error) => assert!(reports_refusal(&error), "{error:?}"),
    }
    let tables: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'analytics_facts'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 1);

    // Paging extremes: a huge offset is an empty page; a limit above the cap is refused with its code.
    let range = range_period(Some("01-10-2026"), Some("11-10-2026"));
    match commission_month(
        db,
        &commission(None, Some(range.clone()), None, Some(u32::MAX), Some(100)),
    ) {
        Ok(far) => assert!(far.rows.is_empty(), "an offset past the end gave rows"),
        Err(error) => assert!(reports_refusal(&error), "{error:?}"),
    }
    match commission_month(
        db,
        &commission(None, Some(range.clone()), None, Some(0), Some(u32::MAX)),
    ) {
        Ok(wide) => assert!(
            wide.rows.len() <= 100,
            "a page of {} rows for a limit of u32::MAX",
            wide.rows.len()
        ),
        Err(error) => assert!(reports_refusal(&error), "{error:?}"),
    }
    let ok =
        commission_month(db, &commission(None, Some(range), None, Some(0), Some(100))).unwrap();
    assert!(i64::try_from(ok.rows.len()).unwrap() <= ok.total_count);
    // The day detail with bad numbers.
    for (date, offset, limit) in [
        ("11-10-2026", u32::MAX, 0),
        ("11-10-2026", 0, u32::MAX),
        ("10-10-2026", 0, 0),
    ] {
        let day = commission_day(
            db,
            &CommissionDayInput {
                date: date.into(),
                offset,
                limit,
                party: None,
                goods: None,
            },
        )
        .unwrap();
        assert!(i64::try_from(day.entries.len()).unwrap() <= day.total_count);
    }
    for date in ["", "31-02-2026", "12-10-2026", "abc", "'; --"] {
        let refused = commission_day(
            db,
            &CommissionDayInput {
                date: date.into(),
                offset: 0,
                limit: 0,
                party: None,
                goods: None,
            },
        );
        match refused {
            // A day after today is an empty day here, while the month page refuses it (ERR_DATE_IN_FUTURE).
            Ok(page) if date == "12-10-2026" => {
                println!("LENIENT: commission_day accepted the future day {date}");
                assert_eq!(page.total_count, 0, "a future day has postings");
            }
            Ok(_) => panic!("the day {date:?} was accepted"),
            Err(error) => assert!(reports_refusal(&error), "day {date:?}: {error:?}"),
        }
    }
}

#[test]
fn buyer_dues_and_the_collection_sheet_refuse_bad_input_and_cap_their_pages() {
    let w = support::world();
    let lot_id = e2e16(&w);
    bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&mixed(&w, lot_id))).unwrap();
    let db = w.env.ctx.db();
    let bad_sort = Some(SortInput {
        column: "evil; DROP TABLE accounts".into(),
        direction: SortDirection::Asc,
        language: None,
    });
    let long = "ا".repeat(20_000);
    let searches = [
        "",
        "'; DROP TABLE accounts; --",
        "%",
        "_",
        "\u{0}",
        "\u{200b}",
        "\u{202e}",
        long.as_str(),
    ];
    for search in searches {
        let page = buyer_dues(
            db,
            &BuyerDuesInput {
                party_id: None,
                search: Some(search.into()),
                sort: None,
                offset: 0,
                limit: 0,
            },
        );
        assert!(
            page.is_ok(),
            "buyer dues search {search:?} failed: {page:?}"
        );
        let sheet = collection_sheet(
            db,
            &CollectionSheetInput {
                date: "11-10-2026".into(),
                search: Some(search.into()),
                sort: None,
                offset: 0,
                limit: 0,
            },
        );
        assert!(sheet.is_ok(), "sheet search {search:?} failed: {sheet:?}");
    }
    let refused = buyer_dues(
        db,
        &BuyerDuesInput {
            party_id: None,
            search: None,
            sort: bad_sort.clone(),
            offset: 0,
            limit: 0,
        },
    );
    assert!(
        refused.is_err_and(|e| reports_refusal(&e)),
        "an unknown sort column was taken"
    );
    let huge = buyer_dues(
        db,
        &BuyerDuesInput {
            party_id: None,
            search: None,
            sort: None,
            offset: u32::MAX,
            limit: u32::MAX,
        },
    )
    .unwrap();
    assert!(huge.rows.is_empty(), "an offset past the end gave rows");
    let wide = buyer_dues(
        db,
        &BuyerDuesInput {
            party_id: None,
            search: None,
            sort: None,
            offset: 0,
            limit: u32::MAX,
        },
    )
    .unwrap();
    assert!(wide.rows.len() <= 100, "a page of {} rows", wide.rows.len());
    for date in ["", "12-10-2026", "31-02-2026", "abc", "'; --", "99-99-9999"] {
        let sheet = collection_sheet(
            db,
            &CollectionSheetInput {
                date: date.into(),
                search: None,
                sort: None,
                offset: 0,
                limit: 0,
            },
        );
        // An empty date is today's sheet by design; every other bad date is a refusal.
        match (date, sheet) {
            ("", Ok(_)) => {}
            (_, Ok(_)) => panic!("the sheet of {date:?} was accepted"),
            (_, Err(error)) => assert!(reports_refusal(&error), "sheet {date:?}: {error:?}"),
        }
    }
    let sheet_sort = collection_sheet(
        db,
        &CollectionSheetInput {
            date: "11-10-2026".into(),
            search: None,
            sort: bad_sort,
            offset: 0,
            limit: 0,
        },
    );
    assert!(sheet_sort.is_err_and(|e| reports_refusal(&e)));
    let wide_sheet = collection_sheet(
        db,
        &CollectionSheetInput {
            date: "11-10-2026".into(),
            search: None,
            sort: None,
            offset: 0,
            limit: u32::MAX,
        },
    )
    .unwrap();
    assert!(
        wide_sheet.rows.len() <= 100,
        "a sheet page of {} rows",
        wide_sheet.rows.len()
    );
}

#[test]
fn statements_money_lent_dues_and_window_bounds_refuse_bad_input() {
    let w = support::world();
    let lot_id = e2e16(&w);
    bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&mixed(&w, lot_id))).unwrap();
    let ctx = &w.env.ctx;
    // The statement of a month.
    for partner_id in [0, -1, i64::MAX, 999_999] {
        let refused = partners::statement_month(
            ctx,
            &PartnersStatementMonthInput {
                partner_id,
                month: None,
            },
        );
        let error = refused.unwrap_err();
        assert!(is_refusal(&error), "partner {partner_id}: {error:?}");
    }
    let long = "1".repeat(5_000);
    for month in [
        "2026-11",
        "2026-13",
        "0000-00",
        "abc",
        "2026-10'; --",
        long.as_str(),
    ] {
        let refused = partners::statement_month(
            ctx,
            &PartnersStatementMonthInput {
                partner_id: w.partners[0],
                month: Some(month.into()),
            },
        );
        let error = refused.unwrap_err();
        assert!(is_refusal(&error), "month {month:?}: {error:?}");
    }
    for date in ["", "31-02-2026", "abc", "'; --", "12-10-2026"] {
        let refused = partners::statement_day(
            ctx,
            &PartnersStatementDayInput {
                partner_id: w.partners[0],
                date: date.into(),
            },
        );
        // A day after today has no movement: a refusal or an empty day, never a data error.
        if let Err(error) = refused {
            assert!(is_refusal(&error), "statement day {date:?}: {error:?}");
        }
    }
    // The money lent dues page.
    for month in ["2026-11", "2026-13", "0000-00", "abc", "2026-10'; --"] {
        let refused = lent::dues(
            ctx,
            &LentDuesInput {
                month: Some(month.into()),
                ..LentDuesInput::default()
            },
        );
        let error = refused.unwrap_err();
        assert!(is_refusal(&error), "dues month {month:?}: {error:?}");
    }
    let page = lent::dues(
        ctx,
        &LentDuesInput {
            partner_id: Some(999_999),
            offset: u32::MAX,
            limit: u32::MAX,
            ..LentDuesInput::default()
        },
    )
    .unwrap();
    assert!(page.rows.is_empty());
    // lent_list with a reversed or hostile range.
    let person = make_person(&w, None);
    for (from, to) in [
        ("10-10-2026", "01-10-2026"),
        ("abc", ""),
        ("", "abc"),
        ("12-10-2026", "13-10-2026"),
        ("'; --", "'; --"),
    ] {
        let refused = lent::list(
            ctx,
            &LentListInput {
                from: Some(from.into()),
                to: Some(to.into()),
                show_finished: false,
                group_by_month: false,
                sort: None,
                person_id: person,
                offset: 0,
                limit: 0,
            },
        );
        if let Err(error) = refused {
            assert!(
                is_refusal(&error),
                "lent_list {from:?} to {to:?}: {error:?}"
            );
        }
    }
    // Window bounds: every source on hostile ids.
    let sources = [
        WindowSource::Sales,
        WindowSource::Journal,
        WindowSource::Cashbook,
        WindowSource::Vouchers { voucher_type: None },
        WindowSource::Audit,
        WindowSource::Expenses,
        WindowSource::Fruit,
        WindowSource::Lent,
        WindowSource::LentDues,
        WindowSource::Khata { account_id: None },
        WindowSource::Khata {
            account_id: Some(i64::MAX),
        },
        WindowSource::Capital,
        WindowSource::PartnerTakes {
            partner_id: Some(0),
        },
        WindowSource::PartnerTakes {
            partner_id: Some(999_999),
        },
        WindowSource::PartnerStatement { partner_id: -1 },
        WindowSource::PartnerStatement {
            partner_id: i64::MAX,
        },
        WindowSource::PartyLedger { account_id: -1 },
        WindowSource::PartyLedger {
            account_id: w.rahim,
        },
        WindowSource::Commission,
    ];
    for source in sources {
        let label = format!("{source:?}");
        match windows::bounds(ctx, &WindowsBoundsInput { source }) {
            Ok(bounds) => {
                assert!(bounds.last_date.len() == 10, "{label}: last day {bounds:?}");
                if let Some(first) = &bounds.first_date {
                    assert!(
                        first.as_str() <= bounds.last_date.as_str(),
                        "{label}: first day after today: {bounds:?}"
                    );
                }
            }
            Err(error) => assert!(is_refusal(&error), "{label}: {error:?}"),
        }
    }
}

/// One commission read of the volume test: its name, input, budget, and the days of its SQL oracle.
type CommissionCase = (
    String,
    CommissionMonthInput,
    Duration,
    Option<(&'static str, &'static str)>,
);

#[test]
fn the_sale_rows_of_the_volume_pass_every_check_and_foreign_key_on_a_small_book() {
    // The same SQL as the ten-year file, run with foreign keys ON on a small book: every parent row is in place
    // and every CHECK of bikri_sales and bikri_lots is met, so the file passes foreign_key_check.
    let w = support::world();
    let conn = raw(&w.dir);
    conn.execute_batch(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
         SELECT '2026-10-10', 'bikri_first', 'voucher', 0, 1, 1
         FROM (SELECT 1 AS n UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5 UNION ALL SELECT 6);",
    )
    .unwrap();
    let packing: i64 = conn
        .query_row("SELECT MIN(id) FROM packings", [], |r| r.get(0))
        .unwrap();
    let rows = SaleRows {
        hot_seller: w.rahim,
        hot_buyer: w.akram,
        seller_base: w.rahim,
        sellers: 1,
        buyer_base: w.arshad,
        buyers: 1,
        items: 1,
        packing,
    };
    conn.execute_batch(&sale_rows_sql(&rows)).unwrap();
    let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    assert!(
        count("SELECT COUNT(*) FROM bikri_sales") >= 7,
        "no sale rows were written"
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM analytics_facts"),
        count("SELECT COUNT(*) FROM bikri_sales"),
        "a fact for every sale line"
    );
    assert_eq!(count("SELECT COUNT(*) FROM pragma_foreign_key_check"), 0);
}

// ---- part B: ten years of data ------------------------------------------------------------------------

fn p95(mut v: Vec<Duration>) -> Duration {
    v.sort();
    let i = ((v.len() as f64) * 0.95).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// `MANDI_PERF_ONLY=commission,buyer` runs only the rows whose name has one of the words.
fn selected(name: &str) -> bool {
    match std::env::var("MANDI_PERF_ONLY") {
        Ok(list) if !list.trim().is_empty() => list
            .split(',')
            .any(|part| name.to_lowercase().contains(&part.trim().to_lowercase())),
        _ => true,
    }
}

/// The one-minute load average of the machine, for the log (the Mac is shared with other agents).
fn load_now() -> String {
    std::process::Command::new("uptime")
        .output()
        .map(|o| {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            text.split("load averages:")
                .nth(1)
                .unwrap_or("?")
                .trim()
                .to_owned()
        })
        .unwrap_or_else(|_| "?".to_owned())
}

/// The CPU speed of the machine right now: the fixed money-shaped workload of `core/benches/calibration.rs`
/// (10,000 percent charges in i128, rounded and sorted), best of 20 runs, in microseconds. The reference
/// is 139 us on this Mac when it is idle (`scripts/perf/README.md`), so a reading of 1,390 us means every
/// timing below is about ten times slower than on a quiet machine.
fn cpu_calibration_us() -> f64 {
    let mut state: u64 = 0x5EED_0000_0001;
    let mut items: Vec<(i128, i128)> = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let rupees = i128::from((state >> 16) % 1_000_000_000);
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bps = i128::from((state >> 16) % 10_001);
        items.push((rupees * 100, bps));
    }
    let mut best = f64::MAX;
    for _ in 0..20 {
        let started = Instant::now();
        let mut parts: Vec<i128> = Vec::with_capacity(items.len());
        let mut total: i128 = 0;
        for &(paisa, bps) in &items {
            let rounded = (paisa * bps + 5_000) / 10_000;
            total += rounded;
            parts.push(rounded);
        }
        parts.sort_unstable();
        std::hint::black_box((total, parts.get(5_000).copied()));
        best = best.min(started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    best
}

/// Measures one row of the table, unless the filter leaves it out.
fn timed(
    table: &mut Vec<(String, Duration, Duration)>,
    name: String,
    budget: Duration,
    f: &mut dyn FnMut(),
) {
    if selected(&name) {
        let took = measure(f);
        println!("RAW {name}: p95 {:.1} ms vs {:.0} ms", ms(took), ms(budget));
        println!(
            "LOAD while measuring {name}: {} (cpu calibration {:.0} us, idle 139 us)",
            load_now(),
            cpu_calibration_us()
        );
        table.push((name, took, budget));
    }
}

/// One warm-up, then 9 timed runs (`MANDI_PERF_RUNS=2` for a slow read); the p95 of them.
fn measure(f: &mut dyn FnMut()) -> Duration {
    f();
    let runs = std::env::var("MANDI_PERF_RUNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(9)
        .max(1);
    p95((0..runs)
        .map(|_| {
            let started = Instant::now();
            f();
            started.elapsed()
        })
        .collect())
}

fn run_sql(conn: &Connection, label: &str, sql: &str) {
    let started = Instant::now();
    conn.execute_batch(sql)
        .unwrap_or_else(|e| panic!("generate {label}: {e}"));
    println!("GEN {label}: {:.1} s", started.elapsed().as_secs_f64());
}

fn scalar(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// What the sale rows of the volume need to know about the book they are written into.
struct SaleRows {
    hot_seller: i64,
    hot_buyer: i64,
    seller_base: i64,
    sellers: i64,
    buyer_base: i64,
    buyers: i64,
    items: i64,
    packing: i64,
}

/// The SQL that writes, for every 3rd bikri posting, one بکری, one lot and 7 sale lines with their sale facts,
/// every parent row in place (foreign keys on, every CHECK of the tables met).
fn sale_rows_sql(p: &SaleRows) -> String {
    let SaleRows {
        hot_seller,
        hot_buyer,
        seller_base,
        sellers,
        buyer_base,
        buyers,
        items,
        packing,
    } = p;
    format!(
        "INSERT INTO bikri_entries (id, number, entry_date, bill_date)
             SELECT e.id, e.id, e.entry_date, e.entry_date FROM journal_entries e
             WHERE e.posting_type = 'bikri_first' AND e.id % 3 = 0;
             INSERT INTO bikri_lots (id, bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received)
             SELECT e.id, e.id, 1,
                    CASE WHEN e.id % 12 = 0 THEN {hot_seller} ELSE {seller_base} + (e.id * 7) % {sellers} END,
                    1 + e.id % {items}, {packing}, 4000
             FROM journal_entries e WHERE e.posting_type = 'bikri_first' AND e.id % 3 = 0;
             INSERT INTO bikri_sales (id, bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
                    seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
                    buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
                    laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, payment_type)
             SELECT e.id * 8 + n.n, e.id, e.id, n.n, e.id, e.entry_date,
                    CASE WHEN (e.id + n.n) % 16 = 0 THEN {hot_buyer} ELSE {buyer_base} + (e.id * 11 + n.n) % {buyers} END,
                    1 + (e.id + n.n) % 40,
                    (50 + (e.id + n.n) % 450) * 100,
                    (1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 100,
                    'percent', 600,
                    (((1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 6) / 100) * 100,
                    (50 + (e.id + n.n) % 450) * 100,
                    (1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 100,
                    'percent', 600,
                    (((1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 6) / 100) * 100,
                    'per_unit', 800, (1 + (e.id + n.n) % 40) * 800,
                    (1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 100
                      + ((((1 + (e.id + n.n) % 40) * (50 + (e.id + n.n) % 450) * 6) / 100) * 100)
                      + (1 + (e.id + n.n) % 40) * 800,
                    'credit'
             FROM journal_entries e
             JOIN (SELECT 1 AS n UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4
                   UNION ALL SELECT 5 UNION ALL SELECT 6 UNION ALL SELECT 7) n
             WHERE e.posting_type = 'bikri_first' AND e.id % 3 = 0;
             INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id,
                    item_id, seller_account_id, buyer_account_id, qty, seller_amount_paisa, seller_comm_paisa,
                    buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
             SELECT s.journal_entry_id, s.entry_date, CAST(substr(s.entry_date, 1, 4) AS INTEGER),
                    CAST(substr(s.entry_date, 6, 2) AS INTEGER), s.bikri_id, s.lot_id, s.id, l.item_id,
                    l.seller_account_id, s.buyer_account_id, s.qty, s.seller_amount_paisa, s.seller_comm_paisa,
                    s.buyer_value_paisa, s.buyer_comm_paisa, s.laga_paisa, 1
             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id;"
    )
}

/// Makes the plan 76 tables on 10 years: sale facts, share rows, drawings, capital and loans, audit rows.
/// Plain SQL with the checks of the tables on, foreign keys off (the NFR-SCL-1 file has no sale rows).
fn extend_volume(path: &std::path::Path, v: &volume::Volume, partner_ids: [i64; 3]) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "PRAGMA foreign_keys = OFF; PRAGMA synchronous = OFF; PRAGMA busy_timeout = 120000;",
    )
    .unwrap();
    let [pa, pb, pc] = partner_ids;
    let seller_base = scalar(
        &conn,
        "SELECT MIN(id) FROM accounts WHERE party_type = 'seller'",
    );
    let buyer_base = scalar(
        &conn,
        "SELECT MIN(id) FROM accounts WHERE party_type = 'buyer'",
    );
    let cash = scalar(&conn, "SELECT id FROM accounts WHERE code = '01010001'");
    let (drawings, capital) = (
        scalar(
            &conn,
            &format!("SELECT drawings_account_id FROM partners WHERE id = {pa}"),
        ),
        scalar(
            &conn,
            &format!("SELECT capital_account_id FROM partners WHERE id = {pa}"),
        ),
    );
    // Sale rows with their parents (plan 76 end gate: the file must pass foreign_key_check, or the app's own
    // check after a migration refuses to upgrade it). For every 3rd bikri posting: one بکری, one lot and
    // 7 sale lines; the sale facts are copies of the sale lines, as the save writes them.
    let packing = scalar(&conn, "SELECT MIN(id) FROM packings");
    run_sql(
        &conn,
        "bikri_entries (172k), bikri_lots (172k), bikri_sales (1.2M), analytics_facts (1.2M)",
        &sale_rows_sql(&SaleRows {
            hot_seller: v.busy_seller,
            hot_buyer: v.busy_buyer,
            seller_base,
            sellers: 8_999,
            buyer_base,
            buyers: 8_999,
            items: 500,
            packing,
        }),
    );
    run_sql(
        &conn,
        "partner_share_rows (income of every 3rd bikri posting, expense of every 50th general entry)",
        &format!(
            "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa)
             SELECT e.id, e.entry_date, p.pid, 'income', 10000 * (1 + e.id % 500), p.bps,
                    (10000 * (1 + e.id % 500) * p.bps) / 10000
             FROM journal_entries e
             JOIN (SELECT {pa} AS pid, 4000 AS bps UNION ALL SELECT {pb}, 3000 UNION ALL SELECT {pc}, 3000) p
             WHERE e.posting_type = 'bikri_first' AND e.id % 3 = 0;
             INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa)
             SELECT e.id, e.entry_date, p.pid, 'expense', 10000 * (1 + e.id % 30), p.bps,
                    -((10000 * (1 + e.id % 30) * p.bps) / 10000)
             FROM journal_entries e
             JOIN (SELECT {pa} AS pid, 4000 AS bps UNION ALL SELECT {pb}, 3000 UNION ALL SELECT {pc}, 3000) p
             WHERE e.posting_type = 'general' AND e.id % 50 = 0;"
        ),
    );
    // Drawings (a week apart) and capital (twice a year) of the first partner, as vouchers with entries.
    let base = scalar(&conn, "SELECT COALESCE(MAX(id), 0) FROM journal_entries");
    run_sql(
        &conn,
        "drawings and capital of partner A (520 + 24 vouchers)",
        &format!(
            "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 520)
             INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             SELECT date('2016-10-20', '+' || (i * 7) || ' days'), 'personal_expense', 'voucher', 0, 1, 1 FROM n;
             INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             SELECT id, 1, {drawings}, 500000, 0, 1, 1 FROM journal_entries WHERE id > {base} AND id <= {base} + 520
             UNION ALL
             SELECT id, 2, {cash}, 0, 500000, 1, 1 FROM journal_entries WHERE id > {base} AND id <= {base} + 520;
             INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, partner_id, reason, current_entry_id, status)
             SELECT 'personal_expense', id - {base}, entry_date, NULL, 500000, 'cash', {pa}, 'house', id, 'saved'
             FROM journal_entries WHERE id > {base} AND id <= {base} + 520;
             UPDATE journal_entries SET status = 'posted', total_paisa = 500000, chain_hash = printf('%064x', id)
             WHERE id > {base} AND id <= {base} + 520;
             WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 24)
             INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             SELECT date('2016-11-01', '+' || (i * 150) || ' days'), 'capital_in', 'voucher', 0, 1, 1 FROM n;
             INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             SELECT id, 1, {cash}, 10000000, 0, 1, 1 FROM journal_entries WHERE id > {base} + 520 AND id <= {base} + 544
             UNION ALL
             SELECT id, 2, {capital}, 0, 10000000, 1, 1 FROM journal_entries WHERE id > {base} + 520 AND id <= {base} + 544;
             INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, partner_id, current_entry_id, status)
             SELECT 'capital_in', id - {base} - 520, entry_date, NULL, 10000000, 'cash', {pa}, id, 'saved'
             FROM journal_entries WHERE id > {base} + 520 AND id <= {base} + 544;
             UPDATE journal_entries SET status = 'posted', total_paisa = 10000000, chain_hash = printf('%064x', id)
             WHERE id > {base} + 520 AND id <= {base} + 544;"
        ),
    );
    // Money lent: 400 people, 8 loans and 4 returns each, over 10 years.
    run_sql(
        &conn,
        "money lent (400 people, 3,200 loans, 1,600 returns)",
        "CREATE TEMP TABLE perf_p AS
           SELECT row_number() OVER (ORDER BY a.id) - 1 AS idx, a.id AS account_id
           FROM accounts a JOIN account_controls c ON c.id = a.control_id
           WHERE c.code = '0103' ORDER BY a.id LIMIT 400;",
    );
    let lent_base = scalar(&conn, "SELECT COALESCE(MAX(id), 0) FROM journal_entries");
    run_sql(
        &conn,
        "loans",
        &format!(
            "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 3200)
             INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             SELECT date('2017-01-01', '+' || ((((i - 1) % 8) * 400) + (((i - 1) / 8) % 300)) || ' days'),
                    'money_lent', 'voucher', 0, 1, 1 FROM n;
             INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             SELECT e.id, 1, p.account_id, (2000 + (p.idx * 13) % 5000) * 100, 0, 1, 1
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {lent_base} - 1) / 8
             WHERE e.id > {lent_base} AND e.id <= {lent_base} + 3200
             UNION ALL
             SELECT e.id, 2, {cash}, 0, (2000 + (p.idx * 13) % 5000) * 100, 1, 1
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {lent_base} - 1) / 8
             WHERE e.id > {lent_base} AND e.id <= {lent_base} + 3200;
             INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, reason, current_entry_id, status)
             SELECT 'money_lent', e.id - {lent_base}, e.entry_date, p.account_id, (2000 + (p.idx * 13) % 5000) * 100,
                    'cash', 'garden', e.id, 'saved'
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {lent_base} - 1) / 8
             WHERE e.id > {lent_base} AND e.id <= {lent_base} + 3200;
             UPDATE journal_entries SET status = 'posted', chain_hash = printf('%064x', id),
                    total_paisa = (SELECT SUM(dr_paisa) FROM journal_lines l WHERE l.entry_id = journal_entries.id)
             WHERE id > {lent_base} AND id <= {lent_base} + 3200;"
        ),
    );
    let ret_base = scalar(&conn, "SELECT COALESCE(MAX(id), 0) FROM journal_entries");
    run_sql(
        &conn,
        "returns",
        &format!(
            "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 1600)
             INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             SELECT date('2017-06-01', '+' || ((((i - 1) % 4) * 800) + (((i - 1) / 4) % 300) + 30) || ' days'),
                    'money_lent_return', 'voucher', 0, 1, 1 FROM n;
             INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             SELECT e.id, 1, {cash}, (1000 + (p.idx * 7) % 2500) * 100, 0, 1, 1
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {ret_base} - 1) / 4
             WHERE e.id > {ret_base} AND e.id <= {ret_base} + 1600
             UNION ALL
             SELECT e.id, 2, p.account_id, 0, (1000 + (p.idx * 7) % 2500) * 100, 1, 1
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {ret_base} - 1) / 4
             WHERE e.id > {ret_base} AND e.id <= {ret_base} + 1600;
             INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, current_entry_id, status)
             SELECT 'money_lent_return', e.id - {ret_base}, e.entry_date, p.account_id, (1000 + (p.idx * 7) % 2500) * 100,
                    'cash', e.id, 'saved'
             FROM journal_entries e JOIN perf_p p ON p.idx = (e.id - {ret_base} - 1) / 4
             WHERE e.id > {ret_base} AND e.id <= {ret_base} + 1600;
             UPDATE journal_entries SET status = 'posted', chain_hash = printf('%064x', id),
                    total_paisa = (SELECT SUM(dr_paisa) FROM journal_lines l WHERE l.entry_id = journal_entries.id)
             WHERE id > {ret_base} AND id <= {ret_base} + 1600;"
        ),
    );
    run_sql(
        &conn,
        "audit_log (1,000,000 rows)",
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 1000000)
         INSERT INTO audit_log (at, user_id, action, entity, entity_id, chain_hash)
         SELECT strftime('%Y-%m-%dT%H:%M:%fZ', '2016-10-11T08:00:00Z', '+' || (i / 280) || ' days', '+' || (i % 280) || ' seconds'),
                1, 'bikri.save', 'bikri', i, printf('%064x', i)
         FROM n;",
    );
    let broken = scalar(&conn, "SELECT COUNT(*) FROM pragma_foreign_key_check");
    println!("VOLUME foreign_key_check: {broken} problems");
    assert_eq!(broken, 0, "the extended file has foreign key problems");
    for table in [
        "bikri_sales",
        "analytics_facts",
        "partner_share_rows",
        "vouchers",
        "journal_entries",
        "audit_log",
    ] {
        println!(
            "VOLUME {table}: {} rows",
            scalar(&conn, &format!("SELECT COUNT(*) FROM {table}"))
        );
    }
}

#[test]
fn the_paged_reads_of_plans_76_and_77_on_ten_years_against_nfr_spd_24() {
    if std::env::var("MANDI_PERF_VOLUME").is_err() {
        println!("MANDI_PERF_VOLUME is not set: the ten-year check of plans 76 and 77 is skipped");
        return;
    }
    let v = volume::ensure_volume();
    volume::describe(&v);
    // The extended file is kept between runs (the first run takes minutes to make it).
    let kept = v.dir.with_file_name("nfr_scl1_plan76");
    let marker = kept.join("plan76.ok");
    if std::fs::read_to_string(&marker).unwrap_or_default().trim() != "plan76 v2" {
        let _ = std::fs::remove_dir_all(&kept);
        std::fs::create_dir_all(kept.join("data")).unwrap();
        std::fs::copy(
            v.dir.join("data").join("mandi.db"),
            kept.join("data").join("mandi.db"),
        )
        .unwrap();
        // Opening the copy runs migration 23 on 10 years of data: it must not take the data away.
        let started = Instant::now();
        let (env, actor) = volume::open(&kept);
        println!(
            "MIGRATION open + upgrade of the 10-year file: {:.1} s (load {})",
            started.elapsed().as_secs_f64(),
            load_now()
        );
        let ids = psupport::three_partners(&env, &actor);
        extend_volume(&kept.join("data").join("mandi.db"), &v, ids);
        drop(env);
        std::fs::write(&marker, "plan76 v2").unwrap();
    }
    let started = Instant::now();
    let (env, _actor) = volume::open(&kept);
    println!(
        "OPEN the extended file: {:.1} s (load {})",
        started.elapsed().as_secs_f64(),
        load_now()
    );
    let partner_rows: Vec<i64> = {
        let conn = Connection::open(kept.join("data").join("mandi.db")).unwrap();
        let mut statement = conn.prepare("SELECT id FROM partners ORDER BY id").unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let (pa, _pb, _pc) = (partner_rows[0], partner_rows[1], partner_rows[2]);
    let ctx = &*env.ctx;
    let db = ctx.db();
    let oracle = Connection::open(kept.join("data").join("mandi.db")).unwrap();

    let mut table: Vec<(String, Duration, Duration)> = Vec::new();
    let first_page = Duration::from_millis(1_000);
    let next_page = Duration::from_millis(300);

    // -- windows_bounds: the plan of each question and its time.
    for (name, sql) in [
        (
            "cashbook",
            "SELECT MIN(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted' JOIN accounts a ON a.id = l.account_id WHERE a.code = '01010001'",
        ),
        ("audit", "SELECT MIN(substr(at, 1, 10)) FROM audit_log"),
        ("sales", "SELECT MIN(entry_date) FROM bikri_entries"),
        (
            "vouchers by type",
            "SELECT MIN(entry_date) FROM vouchers WHERE status = 'saved' AND type IN ('shop_expense', 'personal_expense', 'personal_return')",
        ),
        (
            "journal",
            "SELECT MIN(entry_date) FROM journal_entries WHERE status = 'posted'",
        ),
    ] {
        let plan: Vec<String> = oracle
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
            .unwrap()
            .query_map([], |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        println!("PLAN windows_bounds {name}: {}", plan.join(" | "));
    }
    for (name, source) in [
        ("sales", WindowSource::Sales),
        ("journal", WindowSource::Journal),
        ("cashbook", WindowSource::Cashbook),
        ("vouchers", WindowSource::Vouchers { voucher_type: None }),
        ("audit (1,000,000 rows)", WindowSource::Audit),
        ("expenses", WindowSource::Expenses),
        ("lent", WindowSource::Lent),
        ("capital", WindowSource::Capital),
        (
            "partner takes",
            WindowSource::PartnerTakes {
                partner_id: Some(pa),
            },
        ),
        (
            "partner statement",
            WindowSource::PartnerStatement { partner_id: pa },
        ),
        (
            "party ledger (busiest buyer)",
            WindowSource::PartyLedger {
                account_id: v.busy_buyer,
            },
        ),
        ("commission", WindowSource::Commission),
    ] {
        let input = WindowsBoundsInput { source };
        let bounds = windows::bounds(ctx, &input).unwrap();
        println!("BOUNDS {name}: first day {:?}", bounds.first_date);
        timed(
            &mut table,
            format!("windows_bounds {name}"),
            next_page,
            &mut || {
                windows::bounds(ctx, &input).unwrap();
            },
        );
    }

    // -- commission page.
    let sum_oracle = |from: &str, to: &str| -> (i64, i64, i64, i64) {
        oracle
            .query_row(
                "SELECT COALESCE(SUM(sign * seller_comm_paisa), 0), COALESCE(SUM(sign * buyer_comm_paisa), 0),
                        COALESCE(SUM(sign * laga_paisa), 0),
                        (SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
                         WHERE kind = 'income' AND entry_date BETWEEN ?1 AND ?2)
                 FROM analytics_facts WHERE entry_date BETWEEN ?1 AND ?2",
                [from, to],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
    };
    let buyer = CommissionParty {
        side: CommissionSide::Buyer,
        account_id: v.busy_buyer,
        chukta_person_key: None,
    };
    let seller = CommissionParty {
        side: CommissionSide::Seller,
        account_id: v.busy_seller,
        chukta_person_key: None,
    };
    let month_period = |m: &str| CommissionPeriod {
        kind: CommissionPeriodKind::Month,
        month: Some(m.to_owned()),
        ..CommissionPeriod::default()
    };
    let commission_cases: Vec<CommissionCase> = vec![
        (
            "commission month (current)".into(),
            commission(Some("2026-10"), None, None, None, None),
            first_page,
            Some(("2026-10-01", "2026-10-31")),
        ),
        (
            "commission month 5 years back".into(),
            commission(None, Some(month_period("2021-10")), None, None, None),
            next_page,
            Some(("2021-10-01", "2021-10-31")),
        ),
        (
            "commission month, first month of data".into(),
            commission(None, Some(month_period("2016-10")), None, None, None),
            next_page,
            Some(("2016-10-01", "2016-10-31")),
        ),
        (
            "commission year (current)".into(),
            commission(None, Some(year_period(Some(2026))), None, None, None),
            first_page,
            Some(("2026-01-01", "2026-12-31")),
        ),
        (
            "commission year 5 years back".into(),
            commission(None, Some(year_period(Some(2021))), None, None, None),
            next_page,
            Some(("2021-01-01", "2021-12-31")),
        ),
        (
            "commission range 90 days, page 1".into(),
            commission(
                None,
                Some(range_period(Some("11-07-2026"), Some("10-10-2026"))),
                None,
                Some(0),
                Some(100),
            ),
            first_page,
            Some(("2026-07-11", "2026-10-10")),
        ),
        (
            "commission range 10 years, page 1".into(),
            commission(
                None,
                Some(range_period(Some("11-10-2016"), Some("10-10-2026"))),
                None,
                Some(0),
                Some(100),
            ),
            first_page,
            Some(("2016-10-11", "2026-10-10")),
        ),
        (
            "commission range 10 years, page 2".into(),
            commission(
                None,
                Some(range_period(Some("11-10-2016"), Some("10-10-2026"))),
                None,
                Some(100),
                Some(100),
            ),
            next_page,
            None,
        ),
        (
            "commission month, busiest buyer".into(),
            commission(Some("2026-10"), None, Some(buyer.clone()), None, None),
            first_page,
            None,
        ),
        (
            "commission year, busiest buyer".into(),
            commission(
                None,
                Some(year_period(Some(2021))),
                Some(buyer.clone()),
                None,
                None,
            ),
            first_page,
            None,
        ),
        (
            "commission range 10 years, busiest buyer".into(),
            commission(
                None,
                Some(range_period(Some("11-10-2016"), Some("10-10-2026"))),
                Some(buyer),
                Some(0),
                Some(100),
            ),
            first_page,
            None,
        ),
        (
            "commission month, busiest seller".into(),
            commission(Some("2026-10"), None, Some(seller.clone()), None, None),
            first_page,
            None,
        ),
        (
            "commission range 10 years, busiest seller".into(),
            commission(
                None,
                Some(range_period(Some("11-10-2016"), Some("10-10-2026"))),
                Some(seller),
                Some(0),
                Some(100),
            ),
            first_page,
            None,
        ),
    ];
    for (name, input, budget, check) in commission_cases {
        let page = commission_month(db, &input).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        println!(
            "SHAPE {name}: {} rows of {}, total commission {} paisa",
            page.rows.len(),
            page.total_count,
            page.totals.total_comm_paisa.as_paisa()
        );
        if let (Some((from, to)), None) = (check, &input.party) {
            let (s, b, l, shared) = sum_oracle(from, to);
            assert_eq!(
                page.totals.seller_comm_paisa.as_paisa(),
                s,
                "{name}: seller commission"
            );
            assert_eq!(
                page.totals.buyer_comm_paisa.as_paisa(),
                b,
                "{name}: buyer commission"
            );
            assert_eq!(page.totals.laga_paisa.as_paisa(), l, "{name}: لاگا");
            assert_eq!(
                page.totals.shared_income_paisa.as_paisa(),
                shared,
                "{name}: shared income"
            );
            if let Some(shares) = &page.totals.partner_shares {
                let parts: i64 = shares.iter().map(|p| p.share_paisa.as_paisa()).sum();
                assert_eq!(
                    parts, shared,
                    "{name}: the partner shares do not add up to the shared income"
                );
            }
        }
        timed(&mut table, name, budget, &mut || {
            commission_month(db, &input).unwrap();
        });
    }
    let day_input = CommissionDayInput {
        date: "09-10-2026".into(),
        offset: 0,
        limit: 0,
        party: None,
        goods: None,
    };
    match commission_day(db, &day_input) {
        Ok(a_day) => {
            println!(
                "SHAPE commission day 09-10-2026: {} postings",
                a_day.total_count
            );
            timed(
                &mut table,
                "commission day detail".into(),
                first_page,
                &mut || {
                    commission_day(db, &day_input).unwrap();
                },
            );
        }
        Err(error) => println!(
            "NOTE commission day detail did not run on the synthetic data (no sale rows): {error:?}"
        ),
    }

    // -- buyer dues and the collection sheet.
    let dues_total: i64 = oracle
        .query_row(
            "SELECT COALESCE(SUM(b), 0) FROM (
               SELECT SUM(l.dr_paisa) - SUM(l.cr_paisa) AS b
               FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
               JOIN accounts a ON a.id = l.account_id JOIN account_controls c ON c.id = a.control_id
               WHERE c.code = '0304' AND a.code NOT IN ('0030', '0031', '03040000') GROUP BY l.account_id HAVING b > 0)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let first = buyer_dues(
        db,
        &BuyerDuesInput {
            party_id: None,
            search: None,
            sort: None,
            offset: 0,
            limit: 100,
        },
    )
    .unwrap();
    println!(
        "SHAPE buyer dues: {} buyers, total {} paisa (oracle {dues_total})",
        first.total_count,
        first.total_paisa.as_paisa()
    );
    assert_eq!(
        first.total_paisa.as_paisa(),
        dues_total,
        "buyer dues: the total is not the sum of the balances"
    );
    let by = |column: &str| {
        Some(SortInput {
            column: column.into(),
            direction: SortDirection::Desc,
            language: None,
        })
    };
    for (name, input, budget) in [
        (
            "buyer dues page 1 (by name)",
            BuyerDuesInput {
                party_id: None,
                search: None,
                sort: None,
                offset: 0,
                limit: 100,
            },
            first_page,
        ),
        (
            "buyer dues next page (offset 100)",
            BuyerDuesInput {
                party_id: None,
                search: None,
                sort: None,
                offset: 100,
                limit: 100,
            },
            next_page,
        ),
        (
            "buyer dues deep page (offset 8000)",
            BuyerDuesInput {
                party_id: None,
                search: None,
                sort: None,
                offset: 8_000,
                limit: 100,
            },
            next_page,
        ),
        (
            "buyer dues sorted by balance",
            BuyerDuesInput {
                party_id: None,
                search: None,
                sort: by("balance"),
                offset: 0,
                limit: 100,
            },
            first_page,
        ),
        (
            "buyer dues search",
            BuyerDuesInput {
                party_id: None,
                search: Some("akram".into()),
                sort: None,
                offset: 0,
                limit: 100,
            },
            first_page,
        ),
    ] {
        timed(&mut table, name.into(), budget, &mut || {
            buyer_dues(db, &input).unwrap();
        });
    }
    for (name, date, offset, budget) in [
        (
            "collection sheet today, page 1",
            "10-10-2026",
            0,
            first_page,
        ),
        (
            "collection sheet today, next page",
            "10-10-2026",
            100,
            next_page,
        ),
        ("collection sheet 5 years back", "10-10-2021", 0, first_page),
    ] {
        let input = CollectionSheetInput {
            date: date.into(),
            search: None,
            sort: None,
            offset,
            limit: 100,
        };
        let sheet = collection_sheet(db, &input).unwrap();
        println!(
            "SHAPE {name}: {} rows of {}",
            sheet.rows.len(),
            sheet.total_count
        );
        timed(&mut table, name.into(), budget, &mut || {
            collection_sheet(db, &input).unwrap();
        });
    }

    // -- partner statement.
    let closing_oracle: i64 = {
        let shares: i64 = oracle
            .query_row(&format!("SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE partner_id = {pa}"), [], |r| r.get(0))
            .unwrap();
        let lines: i64 = oracle
            .query_row(
                &format!(
                    "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
                     JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted' AND e.source_type <> 'month_close'
                     WHERE l.account_id IN (SELECT capital_account_id FROM partners WHERE id = {pa}
                                            UNION SELECT drawings_account_id FROM partners WHERE id = {pa})"
                ),
                [],
                |r| r.get(0),
            )
            .unwrap();
        shares + lines
    };
    let current = partners::statement_month(
        ctx,
        &PartnersStatementMonthInput {
            partner_id: pa,
            month: None,
        },
    )
    .unwrap();
    println!(
        "SHAPE partner statement {}: {} days, closing {} paisa (oracle {closing_oracle})",
        current.month,
        current.days.len(),
        current.closing_total_paisa.as_paisa()
    );
    assert_eq!(
        current.closing_total_paisa.as_paisa(),
        closing_oracle,
        "the closing of the statement is not his total"
    );
    for (name, month, budget) in [
        ("partner statement month (current)", None, first_page),
        (
            "partner statement month 5 years back",
            Some("2021-10"),
            next_page,
        ),
        ("partner statement first month", Some("2016-10"), next_page),
    ] {
        let input = PartnersStatementMonthInput {
            partner_id: pa,
            month: month.map(str::to_owned),
        };
        timed(&mut table, name.into(), budget, &mut || {
            partners::statement_month(ctx, &input).unwrap();
        });
    }
    if let Some(day) = current.days.last() {
        let input = PartnersStatementDayInput {
            partner_id: pa,
            date: day.date.clone(),
        };
        timed(
            &mut table,
            "partner statement day".into(),
            next_page,
            &mut || {
                partners::statement_day(ctx, &input).unwrap();
            },
        );
    }

    // -- money lent and the three money lists with a range.
    let (this_month, five_back) = (Some("2026-10".to_owned()), Some("2021-10".to_owned()));
    for (name, month, budget) in [
        ("lent dues (current month)", this_month, first_page),
        ("lent dues 5 years back", five_back, next_page),
    ] {
        let input = LentDuesInput {
            month,
            ..LentDuesInput::default()
        };
        let page = lent::dues(ctx, &input).unwrap();
        println!(
            "SHAPE {name}: {} rows, {} people open, {} paisa",
            page.total_count,
            page.totals.open_people_count,
            page.totals.open_paisa.as_paisa()
        );
        timed(&mut table, name.into(), budget, &mut || {
            lent::dues(ctx, &input).unwrap();
        });
    }
    let one_person = oracle
        .query_row(
            "SELECT account_id FROM vouchers WHERE type = 'money_lent' ORDER BY id LIMIT 1",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap();
    for (name, from, to) in [
        (
            "lent_list one person, month range",
            "01-10-2024",
            "31-10-2024",
        ),
        ("lent_list one person, whole life", "", ""),
    ] {
        let input = LentListInput {
            from: (!from.is_empty()).then(|| from.to_owned()),
            to: (!to.is_empty()).then(|| to.to_owned()),
            show_finished: false,
            group_by_month: false,
            sort: None,
            person_id: one_person,
            offset: 0,
            limit: 100,
        };
        timed(
            &mut table,
            name.into(),
            if from.is_empty() {
                first_page
            } else {
                next_page
            },
            &mut || {
                lent::list(ctx, &input).unwrap();
            },
        );
    }
    for (name, from, to) in [
        ("partner_takes month range", "01-10-2024", "31-10-2024"),
        ("partner_takes whole life", "", ""),
    ] {
        let input = mandi_services::core::expenses::PartnerTakesInput {
            partner_id: Some(pa),
            from: (!from.is_empty()).then(|| from.to_owned()),
            to: (!to.is_empty()).then(|| to.to_owned()),
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
            show_finished: false,
        };
        let page = partners::takes(ctx, &input).unwrap();
        println!("SHAPE {name}: {} rows", page.total_count);
        timed(
            &mut table,
            name.into(),
            if from.is_empty() {
                first_page
            } else {
                next_page
            },
            &mut || {
                partners::takes(ctx, &input).unwrap();
            },
        );
    }
    for (name, from, to) in [
        ("capital_history month range", "01-11-2024", "30-11-2024"),
        ("capital_history whole life", "", ""),
    ] {
        let input = mandi_services::core::capital::CapitalHistoryInput {
            partner_id: pa,
            from: (!from.is_empty()).then(|| from.to_owned()),
            to: (!to.is_empty()).then(|| to.to_owned()),
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        };
        timed(
            &mut table,
            name.into(),
            if from.is_empty() {
                first_page
            } else {
                next_page
            },
            &mut || {
                partners::capital_history(ctx, &input).unwrap();
            },
        );
    }

    let mut over = Vec::new();
    for (name, measured, budget) in &table {
        let ok = measured <= budget;
        println!(
            "PERF plan76: {name}: p95 {:.1} ms vs {:.0} ms ({})",
            ms(*measured),
            ms(*budget),
            if ok { "OK" } else { "OVER" }
        );
        if !ok {
            over.push(name.clone());
        }
    }
    if !cfg!(debug_assertions) {
        assert!(over.is_empty(), "over the NFR-SPD-24 budget: {over:?}");
    }
}
