//! The v1.2 additions to migration 0001 (plan 03 Section 9, PRD v1.2, Architecture 15 v1.2):
//! the advance tick on a lot, partner money taken and returned, money lent, salary deduction,
//! the staff account link and the advance matching trigger.
//! Every refusal here is a CHECK, a foreign key, a unique index or a trigger, not the service.
#![cfg(test)]

mod common;

use std::cell::Cell;

use common::{account_id, classify, draft_entry, draft_entry_of, fixture, line, setup};
use mandi_db::DbError;
use mandi_db::rusqlite::{Connection, Error, params};

fn is_check(error: Error) -> bool {
    matches!(classify(error), DbError::Constraint(m) if m.contains("CHECK constraint failed"))
}

/// The parts of a voucher row that the v1.2 rules look at.
#[derive(Clone, Copy, Default)]
struct V<'a> {
    kind: &'a str,
    mode: Option<&'a str>,
    bank: Option<i64>,
    head: Option<i64>,
    account: Option<i64>,
    partner: Option<i64>,
    reason: Option<&'a str>,
    return_by: Option<&'a str>,
    taken_by: Option<&'a str>,
}

/// Rows the voucher tests point at.
struct Rows {
    /// A partner (capital and drawings accounts are seeded accounts used as stand-ins).
    partner: i64,
    /// A ادھار رقم account of staff member Aslam (control 0103).
    aslam: i64,
    /// The salary expense head.
    salary: i64,
    /// A bank account.
    bank: i64,
}

fn rows(conn: &Connection) -> Rows {
    let capital = account_id(conn, "06010001");
    let drawings = account_id(conn, "06019999");
    conn.execute(
        "INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id) VALUES ('الف', 'Partner A', ?1, ?2)",
        params![capital, drawings],
    )
    .unwrap();
    let partner = conn.last_insert_rowid();
    let control: i64 = conn
        .query_row(
            "SELECT id FROM account_controls WHERE code = '0103'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES (?1, '01030001', 1, 'other', 'اسلم', 'Aslam', 'اسلم', 'aslam', 'aslm')",
        [control],
    )
    .unwrap();
    let aslam = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO expense_heads (code, name_ur, name_en, search_ur, search_en, search_phonetic, account_id)
         VALUES ('1', 'تنخواہ', 'Salaries', 'تنخواہ', 'salaries', 'slrs', ?1)",
        [account_id(conn, "01010001")],
    )
    .unwrap();
    let salary = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO bank_accounts (bank_name, title, number, account_id) VALUES ('HBL', 'Maher Adnan', '0001', ?1)",
        [account_id(conn, "01020001")],
    )
    .unwrap();
    let bank = conn.last_insert_rowid();
    Rows {
        partner,
        aslam,
        salary,
        bank,
    }
}

/// Inserts a voucher of 5,000 rupees with the given parts. The number is new every time.
fn voucher(conn: &Connection, counter: &Cell<i64>, v: V<'_>) -> Result<usize, Error> {
    counter.set(counter.get() + 1);
    conn.execute(
        "INSERT INTO vouchers (type, number, entry_date, amount_paisa, mode, bank_account_id, expense_head_id,
            account_id, partner_id, reason, return_by_date, taken_by)
         VALUES (?1, ?2, '2026-10-10', 500000, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            v.kind,
            counter.get(),
            v.mode.unwrap_or("cash"),
            v.bank,
            v.head,
            v.account,
            v.partner,
            v.reason,
            v.return_by,
            v.taken_by
        ],
    )
}

#[test]
fn a_lot_adjusts_the_advance_by_default_and_the_tick_is_zero_or_one() {
    // PRD FR-12.23: ticked by default; the munshi can untick it (E2E-28).
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let tick = |lot: i64| -> i64 {
        conn.query_row(
            "SELECT adjust_advance FROM bikri_lots WHERE id = ?1",
            [lot],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(tick(f.lot), 1, "the default is on");
    conn.execute(
        "UPDATE bikri_lots SET adjust_advance = 0 WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    assert_eq!(tick(f.lot), 0);
    conn.execute(
        "UPDATE bikri_lots SET adjust_advance = 1 WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    for bad in [2, -1] {
        let refused = conn.execute(
            "UPDATE bikri_lots SET adjust_advance = ?2 WHERE id = ?1",
            params![f.lot, bad],
        );
        assert!(is_check(refused.unwrap_err()), "adjust_advance = {bad}");
    }
    assert_eq!(tick(f.lot), 1);
}

#[test]
fn the_new_voucher_types_are_accepted_and_unknown_ones_are_not() {
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    // PRD 16.3 (Partner A takes 5,000 for home and returns it) and 16.5 (Aslam is lent 5,000).
    let taken = V {
        kind: "personal_expense",
        partner: Some(r.partner),
        reason: Some("گھر کا خرچ"),
        ..V::default()
    };
    let returned = V {
        kind: "personal_return",
        partner: Some(r.partner),
        ..V::default()
    };
    let lent = V {
        kind: "money_lent",
        account: Some(r.aslam),
        reason: Some("گھریلو ضرورت"),
        ..V::default()
    };
    let lent_back = V {
        kind: "money_lent_return",
        account: Some(r.aslam),
        ..V::default()
    };
    for v in [taken, returned, lent, lent_back] {
        assert!(voucher(&conn, &n, v).is_ok(), "{}", v.kind);
    }
    // The old types still work.
    for kind in ["receipt", "payment", "advance", "shop_expense", "general"] {
        assert!(
            voucher(
                &conn,
                &n,
                V {
                    kind,
                    ..V::default()
                }
            )
            .is_ok(),
            "{kind}"
        );
    }
    let unknown = voucher(
        &conn,
        &n,
        V {
            kind: "loan",
            ..V::default()
        },
    );
    assert!(is_check(unknown.unwrap_err()));
    // Numbers are their own series per type (PRD 15.1): number 1 of a new type is free.
    conn.execute(
        "INSERT INTO vouchers (type, number, entry_date, amount_paisa, partner_id) VALUES ('personal_return', 900, '2026-10-10', 100, ?1)",
        [r.partner],
    )
    .unwrap();
    assert!(
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa, partner_id) VALUES ('personal_return', 900, '2026-10-10', 100, ?1)",
            [r.partner],
        )
        .is_err(),
        "(type, number) stays unique"
    );
}

#[test]
fn a_reason_is_required_for_money_taken_and_money_lent() {
    // PRD FR-16.2 and FR-16.4. A return needs none.
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    for (kind, partner, account) in [
        ("personal_expense", Some(r.partner), None),
        ("money_lent", None, Some(r.aslam)),
    ] {
        for reason in [None, Some(""), Some("   ")] {
            let v = V {
                kind,
                partner,
                account,
                reason,
                ..V::default()
            };
            assert!(
                is_check(voucher(&conn, &n, v).unwrap_err()),
                "{kind} with reason {reason:?}"
            );
        }
        let v = V {
            kind,
            partner,
            account,
            reason: Some("گھر کا خرچ"),
            ..V::default()
        };
        assert!(voucher(&conn, &n, v).is_ok(), "{kind}");
    }
    for (kind, partner, account) in [
        ("personal_return", Some(r.partner), None),
        ("money_lent_return", None, Some(r.aslam)),
    ] {
        let v = V {
            kind,
            partner,
            account,
            ..V::default()
        };
        assert!(voucher(&conn, &n, v).is_ok(), "{kind} needs no reason");
    }
}

#[test]
fn the_partner_or_the_person_must_be_named() {
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    for kind in ["personal_expense", "personal_return"] {
        let v = V {
            kind,
            reason: Some("گھر کا خرچ"),
            ..V::default()
        };
        assert!(
            is_check(voucher(&conn, &n, v).unwrap_err()),
            "{kind} without a partner"
        );
    }
    for kind in ["money_lent", "money_lent_return"] {
        let v = V {
            kind,
            reason: Some("گھریلو ضرورت"),
            ..V::default()
        };
        // The trigger on the 0103 control fires before the CHECK; either way it is refused.
        assert!(voucher(&conn, &n, v).is_err(), "{kind} without a person");
    }
    // A partner who does not exist is refused by the foreign key.
    let v = V {
        kind: "personal_return",
        partner: Some(r.partner + 100),
        ..V::default()
    };
    assert!(voucher(&conn, &n, v).is_err());
}

#[test]
fn salary_mode_is_only_for_a_money_lent_return_with_a_salary_head_and_no_bank() {
    // PRD FR-16.4, E2E-30: 2,000 of Aslam's loan is deducted from his salary. No cash moves.
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    let salary = V {
        kind: "money_lent_return",
        mode: Some("salary"),
        account: Some(r.aslam),
        head: Some(r.salary),
        ..V::default()
    };
    assert!(voucher(&conn, &n, salary).is_ok());
    // No salary head named.
    let no_head = V {
        head: None,
        ..salary
    };
    assert!(is_check(voucher(&conn, &n, no_head).unwrap_err()));
    // A bank account on a salary deduction.
    let with_bank = V {
        bank: Some(r.bank),
        ..salary
    };
    assert!(is_check(voucher(&conn, &n, with_bank).unwrap_err()));
    // Salary is not a way to pay anything else.
    for kind in [
        "receipt",
        "payment",
        "advance",
        "shop_expense",
        "personal_expense",
        "personal_return",
        "money_lent",
        "general",
    ] {
        let v = V {
            kind,
            mode: Some("salary"),
            account: Some(r.aslam),
            partner: Some(r.partner),
            head: Some(r.salary),
            reason: Some("گھر کا خرچ"),
            ..V::default()
        };
        assert!(is_check(voucher(&conn, &n, v).unwrap_err()), "{kind}");
    }
    // Bank and cheque rules are as before: a bank voucher still needs its bank account.
    let bank_without = V {
        kind: "money_lent_return",
        mode: Some("bank"),
        account: Some(r.aslam),
        ..V::default()
    };
    assert!(is_check(voucher(&conn, &n, bank_without).unwrap_err()));
    let bank_with = V {
        bank: Some(r.bank),
        ..bank_without
    };
    assert!(voucher(&conn, &n, bank_with).is_ok());
}

#[test]
fn return_by_and_taken_by_belong_to_money_taken_and_money_lent() {
    // PRD 16.3: "Return by" and "Taken by" on money a partner takes; PRD 16.5: "Return by" on a loan.
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    let taken = V {
        kind: "personal_expense",
        partner: Some(r.partner),
        reason: Some("گھر کا خرچ"),
        return_by: Some("2026-10-20"),
        taken_by: Some("بیٹا"),
        ..V::default()
    };
    assert!(voucher(&conn, &n, taken).is_ok());
    let lent = V {
        kind: "money_lent",
        account: Some(r.aslam),
        reason: Some("گھریلو ضرورت"),
        return_by: Some("2026-10-15"),
        ..V::default()
    };
    assert!(voucher(&conn, &n, lent).is_ok());
    // A return by date is a real date.
    for bad in ["2026-13-01", "15-10-2026", "tomorrow"] {
        let v = V {
            return_by: Some(bad),
            ..lent
        };
        assert!(is_check(voucher(&conn, &n, v).unwrap_err()), "{bad}");
    }
    // Not on a return, a receipt, or the other types.
    for kind in ["personal_return", "money_lent_return", "receipt", "payment"] {
        let v = V {
            kind,
            partner: Some(r.partner),
            account: Some(r.aslam),
            return_by: Some("2026-10-20"),
            ..V::default()
        };
        assert!(is_check(voucher(&conn, &n, v).unwrap_err()), "{kind}");
    }
    // Taken by is only for the partner's money taken, and is not blank.
    let lent_taken_by = V {
        taken_by: Some("بیٹا"),
        ..lent
    };
    assert!(is_check(voucher(&conn, &n, lent_taken_by).unwrap_err()));
    let blank = V {
        taken_by: Some("  "),
        ..taken
    };
    assert!(is_check(voucher(&conn, &n, blank).unwrap_err()));
}

#[test]
fn the_new_posting_types_are_accepted() {
    // Architecture 15.7: personal_return, money_lent and money_lent_return are posting types.
    let (_dir, conn) = setup();
    for posting_type in ["personal_return", "money_lent", "money_lent_return"] {
        assert!(
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, total_paisa) VALUES ('2026-10-10', ?1, 'voucher', 1, 500000)",
                [posting_type],
            )
            .is_ok(),
            "{posting_type}"
        );
    }
    let unknown = conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id) VALUES ('2026-10-10', 'loan', 'voucher', 1)",
        [],
    );
    assert!(is_check(unknown.unwrap_err()));
}

#[test]
fn a_staff_member_links_to_one_money_lent_account() {
    // Architecture 15.4: staff.account_id is his ادھار رقم account (nullable, unique).
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let staff = |name: &str, account: Option<i64>| {
        conn.execute(
            "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role, account_id)
             VALUES ('اسلم', ?1, 'اسلم', lower(?1), lower(?1), 'labour', ?2)",
            params![name, account],
        )
    };
    assert!(
        staff("Aslam", None).is_ok(),
        "no account until he is lent money"
    );
    assert!(
        staff("Aslam Two", None).is_ok(),
        "several staff can have no account"
    );
    assert!(staff("Aslam Three", Some(r.aslam)).is_ok());
    let twice = staff("Aslam Four", Some(r.aslam));
    assert!(
        matches!(classify(twice.unwrap_err()), DbError::Constraint(m) if m.contains("UNIQUE")),
        "one account belongs to one person"
    );
    assert!(
        staff("Ghost", Some(r.aslam + 1000)).is_err(),
        "the account must exist"
    );
    // The control exists for the accounts of money lent (PRD FR-10.4 v1.2).
    let (head, name_en, party): (String, String, String) = conn
        .query_row(
            "SELECT h.code, c.name_en, c.party_type FROM account_controls c JOIN account_heads h ON h.id = c.head_id WHERE c.code = '0103'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (head.as_str(), name_en.as_str(), party.as_str()),
        ("01", "Money Lent", "other")
    );
}

/// A draft entry of this posting type with one credit line on `account_id`. The line is the
/// credit that an advance would be matched against.
fn credit_line_of(conn: &Connection, posting_type: &str, account_id: i64, amount: i64) -> i64 {
    // A reversal must name the entry it reverses (a CHECK); any entry will do for a draft.
    let reverses: Option<i64> = (posting_type == "reversal").then(|| draft_entry(conn, 100));
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id, total_paisa)
         VALUES ('2026-10-10', ?1, 'voucher', 1, ?2, ?3)",
        params![posting_type, reverses, amount],
    )
    .unwrap();
    let entry = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, 0, ?3)",
        params![entry, account_id, amount],
    )
    .unwrap();
    conn.last_insert_rowid()
}

/// An advance debit line (flagged `is_advance`) on `account_id`.
fn advance_line_of(conn: &Connection, account_id: i64, amount: i64) -> i64 {
    let entry = draft_entry_of(conn, "advance", amount);
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance) VALUES (?1, 1, ?2, ?3, 0, 1)",
        params![entry, account_id, amount],
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn an_advance_is_adjusted_by_a_bikri_credit_or_a_seller_receipt_only() {
    // PRD FR-15.8 (credit from a بکری posting), FR-15.12 (credit from the seller's own
    // receipt). A جنرل credit is refused until Umar decides (PRD 16.4 against Architecture 15.7).
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let insert = |advance: i64, credit: i64| {
        conn.execute(
            "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 100000)",
            params![advance, credit, f.seller],
        )
    };
    let advance = advance_line_of(&conn, f.seller, 1_500_000);
    for posting_type in ["bikri_first", "bikri_later", "receipt"] {
        let credit = credit_line_of(&conn, posting_type, f.seller, 2_431_600);
        assert!(insert(advance, credit).is_ok(), "{posting_type}");
    }
    for posting_type in [
        "general",
        "reversal",
        "payment",
        "advance",
        "opening",
        "shop_expense",
        "personal_expense",
        "personal_return",
        "money_lent",
        "money_lent_return",
        "month_close",
    ] {
        let credit = credit_line_of(&conn, posting_type, f.seller, 2_431_600);
        let refused = insert(advance, credit).unwrap_err();
        assert!(
            matches!(classify(refused), DbError::AllocationLines),
            "{posting_type} must not adjust an advance"
        );
    }
}

#[test]
fn an_advance_credit_must_be_on_the_same_seller_account() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let advance = advance_line_of(&conn, f.seller, 1_500_000);
    // A receipt credit of a buyer is never an advance adjustment.
    let buyer_receipt = credit_line_of(&conn, "receipt", f.buyer, 500_000);
    let refused = conn.execute(
        "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 100000)",
        params![advance, buyer_receipt, f.seller],
    );
    assert!(matches!(
        classify(refused.unwrap_err()),
        DbError::AllocationLines
    ));
    let refused = conn.execute(
        "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 100000)",
        params![advance, buyer_receipt, f.buyer],
    );
    assert!(matches!(
        classify(refused.unwrap_err()),
        DbError::AllocationLines
    ));
}

#[test]
fn only_an_advance_voucher_or_a_seller_opening_debit_can_carry_the_advance_flag() {
    // E2E-28: the 19,316 ادائیگی to رحیم is a plain debit, never a second open advance.
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let flagged = |posting_type: &str, account: i64| {
        let entry = draft_entry_of(&conn, posting_type, 1_931_600);
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance) VALUES (?1, 1, ?2, 1931600, 0, 1)",
            params![entry, account],
        )
    };
    let refused =
        |e: Error| matches!(classify(e), DbError::Constraint(m) if m.contains("ERR_ADVANCE_FLAG"));
    assert!(flagged("advance", f.seller).is_ok(), "a پیشگی voucher");
    assert!(
        flagged("opening", f.seller).is_ok(),
        "an opening debit marked as an advance"
    );
    for posting_type in [
        "payment",
        "general",
        "bikri_first",
        "bikri_later",
        "receipt",
        "shop_expense",
        "money_lent",
        "personal_expense",
    ] {
        assert!(
            refused(flagged(posting_type, f.seller).unwrap_err()),
            "{posting_type}"
        );
    }
    // A buyer, the cash account or an expense account never holds an advance.
    for account in [f.buyer, account_id(&conn, "01010001")] {
        assert!(refused(flagged("advance", account).unwrap_err()));
    }
    // The plain payment debit is fine without the flag.
    let entry = draft_entry_of(&conn, "payment", 1_931_600);
    assert!(line(&conn, entry, 1, "0010", 1_931_600, 0).is_ok());
}

#[test]
fn e2e_31_the_receipt_and_the_bikri_credit_use_up_one_advance_exactly() {
    // E2E-31: Rahim's advance is 15,000. He returns 10,000 by receipt; بکری 1 credits him
    // 24,316 and adjusts the other 5,000. Together 15,000: the advance is used up.
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let advance = advance_line_of(&conn, f.seller, 1_500_000);
    let receipt = credit_line_of(&conn, "receipt", f.seller, 1_000_000);
    let bikri = credit_line_of(&conn, "bikri_first", f.seller, 2_431_600);
    let alloc = |credit: i64, amount: i64| {
        conn.execute(
            "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, ?4)",
            params![advance, credit, f.seller, amount],
        )
    };
    assert!(alloc(receipt, 1_000_000).is_ok(), "10,000 returned in cash");
    assert!(alloc(bikri, 500_000).is_ok(), "5,000 adjusted by the بکری");
    let total: i64 = conn
        .query_row(
            "SELECT SUM(amount_paisa) FROM advance_allocations WHERE advance_line_id = ?1 AND is_active = 1",
            [advance],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(total, 1_500_000, "10,000 + 5,000 = 15,000");
    // One more rupee is refused: the advance is fully adjusted.
    assert!(matches!(
        classify(alloc(bikri, 10_000).unwrap_err()),
        DbError::AllocationAmount
    ));
    // The receipt itself is used up, too.
    assert!(matches!(
        classify(alloc(receipt, 100).unwrap_err()),
        DbError::AllocationAmount
    ));
}

#[test]
fn e2e_30_the_salary_deduction_rows_split_minus_800_600_600() {
    // E2E-30: 2,000 of Aslam's loan is deducted from his salary on 12-10-26. That part is a
    // shop expense, split 40 / 30 / 30 and written negative. (The test uses seeded accounts as
    // stand-ins for the partners' capital and drawings accounts.)
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    let salary = V {
        kind: "money_lent_return",
        mode: Some("salary"),
        account: Some(r.aslam),
        head: Some(r.salary),
        ..V::default()
    };
    assert!(voucher(&conn, &n, salary).is_ok());
    let entry = draft_entry_of(&conn, "money_lent_return", 500_000);
    let capital = account_id(&conn, "06010001");
    let drawings = account_id(&conn, "06019999");
    let mut partners = vec![r.partner];
    for name in ["Partner B", "Partner C"] {
        conn.execute(
            "INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id) VALUES ('ب', ?1, ?2, ?3)",
            params![name, capital, drawings],
        )
        .unwrap();
        partners.push(conn.last_insert_rowid());
    }
    for (partner, bps, amount) in zip3(&partners, [4000, 3000, 3000], [-80_000, -60_000, -60_000]) {
        conn.execute(
            "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa)
             VALUES (?1, '2026-10-12', ?2, 'expense', 200000, ?3, ?4)",
            params![entry, partner, bps, amount],
        )
        .unwrap();
    }
    let sum: i64 = conn
        .query_row(
            "SELECT SUM(amount_paisa) FROM partner_share_rows WHERE journal_entry_id = ?1",
            [entry],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(sum, -200_000, "the parts add back to minus 2,000");
}

fn zip3(partners: &[i64], bps: [i64; 3], amounts: [i64; 3]) -> Vec<(i64, i64, i64)> {
    partners
        .iter()
        .zip(bps)
        .zip(amounts)
        .map(|((p, b), a)| (*p, b, a))
        .collect()
}

#[test]
fn a_payment_to_a_buyer_needs_a_reason_but_a_payment_to_a_seller_does_not() {
    // PRD FR-15.11, E2E-32: 3,000 cash to اکرم, reason "نقد ادھار". A payment to رحیم (E2E, 4,316) has none.
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let pukhta_buyer = account_id(&conn, "0030");
    let n = Cell::new(0);
    let payment = |account: i64, reason: Option<&str>| {
        n.set(n.get() + 1);
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa, account_id, reason) VALUES ('payment', ?1, '2026-10-10', 300000, ?2, ?3)",
            params![n.get(), account, reason],
        )
    };
    let refused = |e: Error| matches!(classify(e), DbError::Constraint(m) if m.contains("ERR_REASON_REQUIRED"));
    for account in [f.buyer, pukhta_buyer] {
        for reason in [None, Some(""), Some("  ")] {
            assert!(
                refused(payment(account, reason).unwrap_err()),
                "{account} {reason:?}"
            );
        }
    }
    assert!(payment(f.buyer, Some("نقد ادھار")).is_ok());
    assert!(payment(f.seller, None).is_ok(), "a seller needs no reason");
    // The reason cannot be taken away later.
    let cleared = conn.execute(
        "UPDATE vouchers SET reason = NULL WHERE reason IS NOT NULL",
        [],
    );
    assert!(refused(cleared.unwrap_err()));
    let moved = conn.execute(
        "UPDATE vouchers SET account_id = ?1 WHERE account_id = ?2 AND reason IS NULL",
        params![f.buyer, f.seller],
    );
    assert!(refused(moved.unwrap_err()));
}

#[test]
fn money_lent_vouchers_name_an_account_under_control_0103() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let r = rows(&conn);
    let n = Cell::new(0);
    let refused = |e: Error| matches!(classify(e), DbError::Constraint(m) if m.contains("ERR_MONEY_LENT_ACCOUNT"));
    for kind in ["money_lent", "money_lent_return"] {
        for wrong in [f.buyer, f.seller, account_id(&conn, "01010001")] {
            let v = V {
                kind,
                account: Some(wrong),
                reason: Some("گھریلو ضرورت"),
                ..V::default()
            };
            assert!(
                refused(voucher(&conn, &n, v).unwrap_err()),
                "{kind} {wrong}"
            );
        }
        let v = V {
            kind,
            account: Some(r.aslam),
            reason: Some("گھریلو ضرورت"),
            ..V::default()
        };
        assert!(voucher(&conn, &n, v).is_ok(), "{kind}");
    }
    // A voucher cannot be moved to another account afterwards.
    let moved = conn.execute(
        "UPDATE vouchers SET account_id = ?1 WHERE type = 'money_lent'",
        [f.buyer],
    );
    assert!(refused(moved.unwrap_err()));
}

#[test]
fn only_receipts_payments_and_advances_take_a_cheque() {
    // PRD 15.1 has cheque fields; 16.2, 16.3 and 16.5 offer cash and bank only.
    let (_dir, conn) = setup();
    let r = rows(&conn);
    let n = Cell::new(0);
    let cheque = |kind: &str| {
        n.set(n.get() + 1);
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa, mode, bank_account_id, cheque_no, cheque_date, partner_id, account_id, reason)
             VALUES (?1, ?2, '2026-10-10', 500000, 'cheque', ?3, '000123', '2026-10-10', ?4, ?5, 'گھر کا خرچ')",
            params![kind, n.get(), r.bank, r.partner, r.aslam],
        )
    };
    for kind in ["receipt", "payment", "advance"] {
        // account_id is the money-lent person, which the payment trigger accepts (not a buyer).
        assert!(cheque(kind).is_ok(), "{kind}");
    }
    for kind in [
        "shop_expense",
        "personal_expense",
        "personal_return",
        "money_lent",
        "money_lent_return",
        "general",
    ] {
        assert!(is_check(cheque(kind).unwrap_err()), "{kind}");
    }
}
