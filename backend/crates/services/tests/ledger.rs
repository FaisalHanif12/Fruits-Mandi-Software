//! The journal engine (PRD 6, FR-6.2, FR-6.3, NFR-SEC-6, plan 07): post, reverse, the open-day
//! rule, the trial balance, the account ledger and the keyed hash chain over the entries.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::ids::AccountId;
use mandi_services::core::money::Paisa;
use mandi_services::domain::ledger::{
    LedgerError, LedgerLine, PostingType, Side, SourceType, build_entry,
};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap()
}

fn date(text: &str) -> BusinessDate {
    BusinessDate::parse_iso(text).unwrap()
}

fn buyer_akram(env: &TestEnv, owner: &auth::Actor) -> i64 {
    let form = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        control_id: None,
        code: String::new(),
        name_ur: "اکرم ٹول ٹیکس".into(),
        name_en: "Akram Toll Tax".into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "09-10-2026".into(),
        opening_dr: "12,880".into(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    };
    accounts::save(&env.ctx, owner, &form).unwrap().id
}

/// The receipt of PRD Section 4.6: cash Dr 10,000, Akram Cr 10,000.
fn receipt(env: &TestEnv, owner: &auth::Actor, akram: i64, day: &str) -> Result<i64, ServiceError> {
    let cash = id_of(env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(10_000)),
        LedgerLine::new(AccountId::new(akram), Side::Cr, rupees(10_000)),
    ])
    .unwrap();
    env.ctx.db().write(|tx| {
        ledger::post(
            &env.ctx,
            tx,
            owner.user_id,
            &PostingEvent {
                date: date(day),
                posting_type: PostingType::Receipt,
                source_type: SourceType::Voucher,
                source_id: 1,
                narration: Some("receipt".into()),
                reverses: None,
                entry,
            },
        )
        .map(|posted| posted.entry_id)
    })
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_receipt_updates_the_ledger_and_the_trial_balance() {
    // PRD 4.6: Akram owes 12,880, pays 10,000 and still owes 2,880.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let entry_id = receipt(&env, &owner, akram, "2026-10-10").unwrap();

    let balance = ledger::account_balance(&env.ctx, akram).unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa),
        (BalanceSide::Dr, rupees(2_880))
    );
    let cash = ledger::account_balance(&env.ctx, id_of(&env, "01010001")).unwrap();
    assert_eq!(
        (cash.side, cash.amount_paisa),
        (BalanceSide::Dr, rupees(10_000))
    );

    let ledger_rows = ledger::ledger(&env.ctx, akram, None, Some("2026-10-10"), None).unwrap();
    assert_eq!(
        (ledger_rows.opening.side, ledger_rows.opening.amount_paisa),
        (BalanceSide::Dr, rupees(12_880))
    );
    assert_eq!(ledger_rows.lines.len(), 1);
    assert_eq!(ledger_rows.lines[0].cr, rupees(10_000));
    assert_eq!(ledger_rows.lines[0].posting_type, "receipt");
    assert_eq!(ledger_rows.closing.amount_paisa, rupees(2_880));
    let whole = ledger::ledger(&env.ctx, akram, None, None, None).unwrap();
    assert_eq!(whole.lines.len(), 2, "the opening and the receipt");
    assert_eq!(whole.lines[0].balance.amount_paisa, rupees(12_880));
    assert_eq!(whole.lines[1].balance.amount_paisa, rupees(2_880));

    // The entry is posted, has its total and a chain hash, and the books balance.
    let (status, total, hash): (String, i64, Option<String>) = raw(&dir)
        .query_row(
            "SELECT status, total_paisa, chain_hash FROM journal_entries WHERE id = ?1",
            [entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((status.as_str(), total), ("posted", 1_000_000));
    assert_eq!(hash.map(|h| h.len()), Some(64));
    let trial = ledger::trial_balance(&env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    // Up to the day before the receipt only the opening counts.
    let before = ledger::trial_balance(&env.ctx, Some("2026-10-09")).unwrap();
    assert_eq!(before.total_dr, rupees(12_880));
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn a_correction_is_a_reversal_on_the_same_date_and_only_once() {
    // PRD FR-6.3: posted rows are never changed or deleted.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let entry_id = receipt(&env, &owner, akram, "2026-10-10").unwrap();
    let lines_before = count(&dir, "SELECT COUNT(*) FROM journal_lines");

    let reversal = env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, entry_id, "wrong amount"))
        .unwrap();
    let balance = ledger::account_balance(&env.ctx, akram).unwrap();
    assert_eq!(
        balance.amount_paisa,
        rupees(12_880),
        "back to before the receipt"
    );
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_lines"),
        lines_before + 2,
        "old rows stay"
    );
    let (kind, reverses, day): (String, i64, String) = raw(&dir)
        .query_row(
            "SELECT posting_type, reverses_entry_id, entry_date FROM journal_entries WHERE id = ?1",
            [reversal.entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (kind.as_str(), reverses, day.as_str()),
        ("reversal", entry_id, "2026-10-10")
    );

    // A second reversal of the same entry, a reversal of a reversal, and a missing entry.
    for target in [entry_id, reversal.entry_id] {
        let error = env
            .ctx
            .db()
            .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, target, "again"))
            .unwrap_err();
        assert!(matches!(error, ServiceError::Data(_)), "{error:?}");
    }
    let error = env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, 99_999, "none"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrNotFound);
    ledger::verify_journal_chain(&env.ctx).unwrap();
    let trial = ledger::trial_balance(&env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn only_an_open_day_up_to_today_takes_a_posting() {
    // PRD FR-6.6. The test clock says 10-10-2026.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = receipt(&env, &owner, akram, "2026-10-11").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateInFuture);
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-09' WHERE id = 1",
            [],
        )
        .unwrap();
    let error = receipt(&env, &owner, akram, "2026-10-09").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateClosed);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        entries,
        "nothing is left behind"
    );
    // The first open day works.
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    // Its reversal is refused once its day is closed (the Owner reopens first, FR-21.2).
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-10' WHERE id = 1",
            [],
        )
        .unwrap();
    let last: i64 = count(&dir, "SELECT MAX(id) FROM journal_entries");
    let error = env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, last, "late"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateClosed);
}

#[test]
fn an_unbalanced_entry_never_reaches_the_database() {
    // PRD FR-6.2: refused before any row is written, with the one code the user sees.
    let cash = AccountId::new(1);
    let other = AccountId::new(2);
    let bad = build_entry(vec![
        LedgerLine::new(cash, Side::Dr, rupees(10_000)),
        LedgerLine::new(other, Side::Cr, rupees(9_900)),
    ])
    .unwrap_err();
    assert!(matches!(bad, LedgerError::Unbalanced { .. }));
    assert_eq!(
        ledger::ledger_error(bad).code(),
        ErrorCode::ErrJournalUnbalanced
    );
    // The first side with nothing on the other.
    let one_side = build_entry(vec![LedgerLine::new(cash, Side::Dr, rupees(100))]).unwrap_err();
    assert_eq!(
        ledger::ledger_error(one_side).code(),
        ErrorCode::ErrJournalUnbalanced
    );
    // A zero line and an amount over the limit have their own refusals.
    let zero = build_entry(vec![
        LedgerLine::new(cash, Side::Dr, rupees(0)),
        LedgerLine::new(other, Side::Cr, rupees(0)),
    ])
    .unwrap_err();
    assert!(matches!(zero, LedgerError::ZeroAmount { .. }));
    let huge = Paisa::try_from_paisa(100_000_000_000_100).unwrap();
    let too_large = build_entry(vec![
        LedgerLine::new(cash, Side::Dr, huge),
        LedgerLine::new(other, Side::Cr, huge),
    ])
    .unwrap_err();
    // The entry limit (10^12 rupees) has its own code, so its `max` is not mistaken for the
    // limit of one typed amount (10^10 rupees, end-gate review 14).
    let refused = ledger::ledger_error(too_large);
    assert_eq!(refused.code(), ErrorCode::ErrEntryTooLarge);
    let ServiceError::Rejected(items) = refused else {
        panic!("not a rejection")
    };
    assert_eq!(
        items[0].values.get("max"),
        Some(&ErrorValue::Number(1_000_000_000_000))
    );
}

#[test]
fn the_hash_chain_finds_an_entry_changed_outside_the_app() {
    // NFR-SEC-6: a changed total, a changed date and a removed hash all break the chain, at the
    // entry that was touched.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    ledger::verify_journal_chain(&env.ctx).unwrap();

    let conn = raw(&dir);
    conn.execute_batch("DROP TRIGGER journal_entries_posted_is_final;")
        .unwrap();
    let last: i64 = count(&dir, "SELECT MAX(id) FROM journal_entries");
    for tamper in [
        format!("UPDATE journal_entries SET total_paisa = total_paisa + 100 WHERE id = {last}"),
        format!("UPDATE journal_entries SET entry_date = '2026-10-01' WHERE id = {last}"),
        format!("UPDATE journal_entries SET chain_hash = NULL WHERE id = {last}"),
    ] {
        let saved: (i64, String, Option<String>) = conn
            .query_row(
                "SELECT total_paisa, entry_date, chain_hash FROM journal_entries WHERE id = ?1",
                [last],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        conn.execute(&tamper, []).unwrap();
        let error = ledger::verify_journal_chain(&env.ctx).unwrap_err();
        assert!(
            error.to_string().contains(&last.to_string()),
            "{tamper}: the break names entry {last}: {error}"
        );
        conn.execute(
            "UPDATE journal_entries SET total_paisa = ?2, entry_date = ?3, chain_hash = ?4 WHERE id = ?1",
            (last, saved.0, saved.1, saved.2),
        )
        .unwrap();
        ledger::verify_journal_chain(&env.ctx).unwrap();
    }
}

#[test]
fn a_changed_line_or_a_turned_back_entry_breaks_the_chain() {
    // The line amounts, the accounts and the status are all covered by the check.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    let conn = raw(&dir);
    conn.execute_batch(
        "DROP TRIGGER journal_lines_no_update; DROP TRIGGER journal_entries_posted_is_final;",
    )
    .unwrap();
    let last: i64 = count(&dir, "SELECT MAX(id) FROM journal_entries");
    for (tamper, undo) in [
        (
            format!(
                "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE entry_id = {last} AND dr_paisa > 0"
            ),
            format!(
                "UPDATE journal_lines SET dr_paisa = dr_paisa - 100 WHERE entry_id = {last} AND dr_paisa > 0"
            ),
        ),
        (
            format!(
                "UPDATE journal_lines SET account_id = {akram} WHERE entry_id = {last} AND dr_paisa > 0"
            ),
            format!(
                "UPDATE journal_lines SET account_id = {} WHERE entry_id = {last} AND dr_paisa > 0",
                id_of(&env, "01010001")
            ),
        ),
        (
            format!("UPDATE journal_entries SET status = 'draft' WHERE id = {last}"),
            format!("UPDATE journal_entries SET status = 'posted' WHERE id = {last}"),
        ),
    ] {
        conn.execute(&tamper, []).unwrap();
        let error = ledger::verify_journal_chain(&env.ctx).unwrap_err();
        assert!(
            error.to_string().contains(&last.to_string()),
            "{tamper}: {error}"
        );
        conn.execute(&undo, []).unwrap();
        ledger::verify_journal_chain(&env.ctx).unwrap();
    }
}

#[test]
fn the_newest_entry_cannot_be_removed_without_a_trace() {
    // No later entry points at the newest one, so the recorded head is what finds it missing.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    ledger::verify_journal_chain(&env.ctx).unwrap();
    let conn = raw(&dir);
    conn.execute_batch(
        "DROP TRIGGER journal_lines_no_delete; DROP TRIGGER journal_entries_no_delete;",
    )
    .unwrap();
    let last: i64 = count(&dir, "SELECT MAX(id) FROM journal_entries");
    conn.execute("DELETE FROM journal_lines WHERE entry_id = ?1", [last])
        .unwrap();
    conn.execute("DELETE FROM journal_entries WHERE id = ?1", [last])
        .unwrap();
    let error = ledger::verify_journal_chain(&env.ctx).unwrap_err();
    assert!(error.to_string().contains(&last.to_string()), "{error}");
}

#[test]
fn a_line_on_a_pukhta_khata_account_needs_its_person_and_others_must_not_have_one() {
    // PRD FR-13.5: balances there are kept per person.
    let (dir, env, owner) = ready();
    let cash = id_of(&env, "01010001");
    let pukhta = id_of(&env, "03049999");
    let build = |person: Option<&str>, on_pukhta: bool, other_person: Option<&str>| {
        let mut a = LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(100));
        a.person_key = other_person.map(str::to_owned);
        let mut b = LedgerLine::new(
            AccountId::new(if on_pukhta { pukhta } else { cash }),
            Side::Cr,
            rupees(100),
        );
        b.person_key = person.map(str::to_owned);
        build_entry(vec![a, b]).unwrap()
    };
    let try_post = |entry| {
        env.ctx.db().write(|tx| {
            ledger::post(
                &env.ctx,
                tx,
                owner.user_id,
                &PostingEvent {
                    date: date("2026-10-10"),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
    };
    // No person on a پختہ کھاتہ line, an empty person, and a person on a normal account.
    assert!(matches!(
        try_post(build(None, true, None)),
        Err(ServiceError::Data(_))
    ));
    assert!(matches!(
        try_post(build(Some("  "), true, None)),
        Err(ServiceError::Data(_))
    ));
    assert!(matches!(
        try_post(build(Some("shahid|0300"), false, None)),
        Err(ServiceError::Data(_))
    ));
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        0,
        "nothing was written"
    );
    // The right way works, and the person is kept on the line.
    try_post(build(Some("shahid|0300"), true, None)).unwrap();
    let person = ledger::person_balance(&env.ctx, pukhta, "shahid|0300").unwrap();
    assert_eq!(
        (person.side, person.amount_paisa),
        (BalanceSide::Cr, rupees(100))
    );
    assert_eq!(
        ledger::person_balance(&env.ctx, pukhta, "naseer|0301")
            .unwrap()
            .side,
        BalanceSide::Zero
    );
}

#[test]
fn a_date_that_is_not_iso_is_refused_not_compared_as_text() {
    let (_dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    for bad in ["10-10-2026", "2026-13-01", "today"] {
        assert!(
            matches!(
                ledger::ledger(&env.ctx, akram, None, None, Some(bad)),
                Err(ServiceError::Data(_))
            ),
            "{bad}"
        );
        assert!(
            matches!(
                ledger::ledger(&env.ctx, akram, None, Some(bad), None),
                Err(ServiceError::Data(_))
            ),
            "{bad}"
        );
        assert!(
            matches!(
                ledger::trial_balance(&env.ctx, Some(bad)),
                Err(ServiceError::Data(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn an_account_ledger_follows_the_running_balance_through_a_change_of_side() {
    // A buyer who pays more than he owes ends on the credit side (PRD E11).
    let (_dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let cash = id_of(&env, "01010001");
    for (amount, day) in [(10_000, "2026-10-10"), (5_000, "2026-10-10")] {
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(amount)),
            LedgerLine::new(AccountId::new(akram), Side::Cr, rupees(amount)),
        ])
        .unwrap();
        env.ctx
            .db()
            .write(|tx| {
                ledger::post(
                    &env.ctx,
                    tx,
                    owner.user_id,
                    &PostingEvent {
                        date: date(day),
                        posting_type: PostingType::Receipt,
                        source_type: SourceType::Voucher,
                        source_id: 1,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap();
    }
    let rows = ledger::ledger(&env.ctx, akram, None, None, None).unwrap();
    let shown: Vec<(BalanceSide, i64)> = rows
        .lines
        .iter()
        .map(|l| (l.balance.side, l.balance.amount_paisa.as_paisa() / 100))
        .collect();
    assert_eq!(
        shown,
        [
            (BalanceSide::Dr, 12_880),
            (BalanceSide::Dr, 2_880),
            (BalanceSide::Cr, 2_120)
        ]
    );
    assert_eq!(
        (rows.closing.side, rows.closing.amount_paisa),
        (BalanceSide::Cr, rupees(2_120))
    );
}

#[test]
fn openings_through_the_grid_chain_with_every_other_entry() {
    // The opening entries and the postings of the day are one chain.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let cash = id_of(&env, "01010001");
    let version = accounts::get(&env.ctx, cash).unwrap().version;
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: cash,
                version,
                opening_date: "31-08-2026".into(),
                opening_dr: "65000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    ledger::verify_journal_chain(&env.ctx).unwrap();
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE chain_hash IS NULL"
        ),
        0
    );
    assert_chain_ok(&dir);
}

#[test]
fn a_new_posting_on_an_inactive_account_is_refused_and_writes_nothing() {
    // End-gate review 20, PRD FR-10.2: inactive accounts take no new postings. The check is in
    // `ledger::post`, so every module (vouchers, بکری, openings, month close) has it.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    accounts::set_active(&env.ctx, &owner, akram, false).unwrap();
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let lines = count(&dir, "SELECT COUNT(*) FROM journal_lines");

    let error = receipt(&env, &owner, akram, "2026-10-10").unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].code, ErrorCode::ErrAccountInactive);
    assert_eq!(
        items[0].values.get("account_id"),
        Some(&ErrorValue::Number(akram))
    );
    // Not even a draft is left behind.
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_lines"), lines);

    // Active again, the same posting works.
    accounts::set_active(&env.ctx, &owner, akram, true).unwrap();
    receipt(&env, &owner, akram, "2026-10-10").unwrap();
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn a_reversal_still_works_on_an_account_made_inactive_after_the_posting() {
    // The books must always be correctable (PRD FR-6.3): reversing takes an entry back, it does
    // not add new business on the account.
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let entry_id = receipt(&env, &owner, akram, "2026-10-10").unwrap();
    accounts::set_active(&env.ctx, &owner, akram, false).unwrap();

    let reversed = env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, entry_id, "wrong buyer"))
        .unwrap();
    assert_eq!(reversed.total, rupees(10_000));
    // The 10,000 receipt is taken back: Akram owes the full 12,880 again, inactive or not.
    let balance = ledger::account_balance(&env.ctx, akram).unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa),
        (BalanceSide::Dr, rupees(12_880))
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id IS NOT NULL"
        ),
        1
    );
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn one_inactive_account_among_active_ones_refuses_the_whole_entry() {
    let (dir, env, owner) = ready();
    let akram = buyer_akram(&env, &owner);
    let cash = id_of(&env, "01010001");
    accounts::set_active(&env.ctx, &owner, akram, false).unwrap();
    let before = count(&dir, "SELECT COUNT(*) FROM journal_lines");
    // The inactive account is the second of three lines; the others are active.
    let other = id_of(&env, "04010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(1_000)),
        LedgerLine::new(AccountId::new(akram), Side::Cr, rupees(400)),
        LedgerLine::new(AccountId::new(other), Side::Cr, rupees(600)),
    ])
    .unwrap();
    let error = env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &env.ctx,
                tx,
                owner.user_id,
                &PostingEvent {
                    date: date("2026-10-10"),
                    posting_type: PostingType::Receipt,
                    source_type: SourceType::Voucher,
                    source_id: 9,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrAccountInactive);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_lines"), before);
}
