//! The invariants of the whole books, asserted after EVERY step of a random run (plan 80, lane T1).
//!
//! Every check reads plain SQL on a second connection to the data file, a way apart from the one
//! the app uses, and compares it with what the services say where the services also say it.
//!
//! * A: every journal entry balances, nothing is left as a draft, the trial balance is 0;
//! * B: every stored amount is a whole rupee;
//! * C: an entry and its reversal are zero on every account; a cancelled بکری nets to 0;
//! * D: the partner share rows of an entry add back exactly to its income and shop expense lines,
//!   and a reversal's rows undo its entry's rows partner by partner (PRD FR-6.5, C10, C11);
//! * E: Σ کل رقم + ابتدائی سرمایہ (+ ابتدائی فرق) + open advances to sellers = equity (FR-17.9 as
//!   changed by FR-17.13), and each partner's کل رقم is the C22 formula on plain sums;
//! * F: the advance events never give a partner back more than his open part (C28);
//! * G: the stored bill and advance matching rows are the canonical ones, and consistent;
//! * H: trips: cost released never goes over the total cost, and equals it when nothing is left (C21);
//! * I: gardens: the parts of a payment add up to it (C23), and a closed garden's account is 0 (C26);
//! * J: fruit: the parts add up to the value (C27), and a lot's balance is never below 0 (C16);
//! * M: the daily books check of the app (`health::check_books`: the trial balance, both hash chains
//!   and every money table against the journal, NFR-SEC-6 v1.21) is clean after every step;
//! * N: the owner's refunds of a garden are never above what he could owe, `max(0, installments
//!   less rent)` (PRD FR-12.51, plan 80 R4-3);
//! * O: a closed day never changes: the journal entries dated on or before the closed-up-to day are
//!   the same as when it was closed, except through a reopen (PRD FR-6.6, FR-6.3, FR-10.11).

use mandi_services::db::rusqlite::Connection;
use mandi_services::{cash, ledger, matching, partners};

use crate::common::raw;
use crate::universe::U;

/// One number from a plain SQL statement.
pub fn q(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0))
        .unwrap_or_else(|e| panic!("{e}: {sql}"))
}

/// The tables a refused step must leave exactly as they were, and the money in the books.
const COUNTED: &[&str] = &[
    "journal_entries",
    "journal_lines",
    "partner_share_rows",
    "stock_moves",
    "analytics_facts",
    "bill_allocations",
    "advance_allocations",
    "vouchers",
    "bikri_entries",
    "bikri_lots",
    "bikri_sales",
    "venture_entries",
    "venture_entry_parts",
    "purchase_cost_parts",
    "purchases",
    "ventures",
    "fruit_takes",
    "accounts",
    "dead_marks",
    "partner_shares",
    "lot_counters",
];

/// Row counts of the tables above and the totals of the journal. A step that is refused must not
/// change this (NFR-REL-1: nothing half-saved).
pub fn fingerprint(u: &U) -> Vec<i64> {
    let conn = raw(&u.w.dir);
    let mut out: Vec<i64> = COUNTED
        .iter()
        .map(|t| q(&conn, &format!("SELECT COUNT(*) FROM {t}")))
        .collect();
    out.push(q(
        &conn,
        "SELECT COALESCE(SUM(dr_paisa), 0) + COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
    ));
    out.push(q(&conn, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"));
    // Soft state that a refused step could still flip: matching rows in force, parts in force.
    out.push(q(
        &conn,
        "SELECT COUNT(*) FROM bill_allocations WHERE is_active = 1",
    ));
    out.push(q(
        &conn,
        "SELECT COUNT(*) FROM advance_allocations WHERE is_active = 1",
    ));
    out.push(q(
        &conn,
        "SELECT COUNT(*) FROM venture_entry_parts WHERE is_active = 1",
    ));
    out.push(q(
        &conn,
        "SELECT COALESCE(SUM(qty_sold + qty_written_off + qty_taken), 0) FROM bikri_lots",
    ));
    out.push(q(
        &conn,
        "SELECT COALESCE(SUM(qty_sold + qty_written_off + cost_released_paisa), 0) FROM purchases",
    ));
    out
}

/// The condition on `journal_entries e` for an entry in force: posted, not a reversal, not reversed.
const LIVE: &str = "e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')";

/// What the advances to sellers still have out, in paisa, worked out here on plain sums: each live
/// advance line less what the active matching rows of posted entries settled.
pub fn open_advances(conn: &Connection) -> i64 {
    q(
        conn,
        &format!(
            "SELECT COALESCE(SUM(l.dr_paisa - COALESCE((
                 SELECT SUM(a.amount_paisa) FROM advance_allocations a
                 JOIN journal_lines cl ON cl.id = a.credit_line_id
                 JOIN journal_entries ce ON ce.id = cl.entry_id
                 WHERE a.advance_line_id = l.id AND a.is_active = 1 AND ce.status = 'posted'), 0)), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.is_advance = 1 AND l.dr_paisa > 0 AND {LIVE}"
        ),
    )
}

/// All the checks, after one step.
pub fn check_all(u: &U, at: &str) {
    let conn = raw(&u.w.dir);
    books_balance(u, &conn, at);
    whole_rupees(&conn, at);
    reversals(&conn, at);
    shares(u, &conn, at);
    equity_and_totals(u, &conn, at);
    matching_rows(u, &conn, at);
    trips(&conn, at);
    gardens(&conn, at);
    fruit_and_lots(&conn, at);
    workings(u, &conn, at);
    dead_amount(&conn, at);
    garden_rent(u, &conn, at);
    refunds_covered(&conn, at);
    books_check(u, at);
}

/// M: the books check of the app, after every step. It reads and writes nothing.
fn books_check(u: &U, at: &str) {
    let books = mandi_services::health::check_books(&u.w.env.ctx).expect("the books check runs");
    assert!(
        books.is_ok(),
        "{at}: M: the books check found a problem: difference {:?}, journal chain {:?}, audit chain {:?}, tables {:?}",
        books.difference,
        books.journal_chain,
        books.audit_chain,
        books.tables
    );
}

/// N: for every garden, the refunds of its owner are at most the installments over the rent
/// (PRD FR-12.51). A rent raised or an installment cancelled or lowered after a refund must be
/// refused, or the refunds would be above what he could owe (plan 80 R4-3).
fn refunds_covered(conn: &Connection, at: &str) {
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM ventures v WHERE v.kind = 'garden' AND
               COALESCE((SELECT SUM(e.amount_paisa) FROM venture_entries e
                         WHERE e.venture_id = v.id AND e.kind = 'refund' AND e.status = 'saved'), 0)
               > MAX(0, COALESCE((SELECT SUM(e.amount_paisa) FROM venture_entries e
                         WHERE e.venture_id = v.id AND e.kind = 'installment' AND e.status = 'saved'), 0)
                        - v.rent_paisa)"
        ),
        0,
        "{at}: N: the refunds of a garden's owner are above what he could owe (PRD FR-12.51, R4-3)"
    );
}

/// The `YYYY-MM-DD` day the service says is closed up to, if any.
pub fn closed_up_to(u: &U) -> Option<String> {
    mandi_services::day::status(&u.w.env.ctx)
        .expect("the day status reads")
        .closed_up_to
}

/// The closed-up-to day as a level of the run: -1 for none of its three days, 0 for the 10th, 1 for
/// the 11th, 2 for the 12th.
pub fn level_of(closed: Option<&str>) -> i8 {
    match closed {
        None => -1,
        Some(day) if day < "2026-10-10" => -1,
        Some("2026-10-10") => 0,
        Some("2026-10-11") => 1,
        Some(_) => 2,
    }
}

/// O: the posted journal entries dated on or before the closed-up-to day, an opening apart.
pub fn closed_entries(u: &U) -> std::collections::BTreeSet<i64> {
    let Some(closed) = closed_up_to(u) else {
        return std::collections::BTreeSet::new();
    };
    let conn = raw(&u.w.dir);
    let mut s = conn
        .prepare(
            "SELECT id FROM journal_entries WHERE status = 'posted' AND posting_type <> 'opening'
             AND entry_date <= ?1 ORDER BY id",
        )
        .unwrap();
    s.query_map([closed], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// How many audit rows there are (S7: every change writes one in its own save).
pub fn audit_rows(u: &U) -> i64 {
    q(&raw(&u.w.dir), "SELECT COUNT(*) FROM audit_log")
}

/// I: the rent still due, what the owner owes back and the relief follow the entries in force
/// (PRD FR-12.51): due = rent less the installments, never below 0; owes = installments over the
/// rent less the refunds, never below 0; relief = the original rent less the rent.
fn garden_rent(u: &U, conn: &Connection, at: &str) {
    use mandi_services::core::gardens::GardenGetInput;
    let row = mandi_services::gardens::get(
        &u.w.env.ctx,
        &GardenGetInput {
            id: u.garden,
            history_offset: 0,
            history_limit: 1,
        },
    )
    .expect("the garden reads")
    .row;
    let sum = |kind: &str| {
        q(
            conn,
            &format!(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entries
                 WHERE venture_id = {} AND kind = '{kind}' AND status = 'saved'",
                u.garden
            ),
        )
    };
    let (installments, refunds) = (sum("installment"), sum("refund"));
    let rent = row.rent_paisa.as_paisa();
    assert_eq!(
        row.rent_due_paisa.as_paisa(),
        (rent - installments).max(0),
        "{at}: I: rent still due is not the rent less the installments"
    );
    assert_eq!(
        row.owner_owes_paisa.as_paisa(),
        (installments - rent - refunds).max(0),
        "{at}: I: what the owner owes is not the installments over the rent less the refunds"
    );
    assert_eq!(
        row.relief_paisa.as_paisa(),
        // Only a rent BELOW the original is a relief (PRD FR-12.51); a raised rent is no relief.
        row.rent_original_paisa
            .map_or(0, |o| (o.as_paisa() - rent).max(0)),
        "{at}: I: the relief is not the original rent less the rent, or 0 when the rent was raised"
    );
}

fn books_balance(u: &U, conn: &Connection, at: &str) {
    assert_eq!(
        q(
            conn,
            "SELECT COALESCE(SUM(dr_paisa), 0) - COALESCE(SUM(cr_paisa), 0) FROM journal_lines"
        ),
        0,
        "{at}: A: the books do not balance"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM journal_entries e
             WHERE COALESCE((SELECT SUM(dr_paisa) FROM journal_lines WHERE entry_id = e.id), 0)
                <> COALESCE((SELECT SUM(cr_paisa) FROM journal_lines WHERE entry_id = e.id), 0)"
        ),
        0,
        "{at}: A: a journal entry does not balance"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"
        ),
        0,
        "{at}: A: a draft journal entry was left behind"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM journal_entries e
             WHERE e.total_paisa <> COALESCE((SELECT SUM(dr_paisa) FROM journal_lines WHERE entry_id = e.id), 0)"
        ),
        0,
        "{at}: A: an entry's stored total is not its debits"
    );
    let trial = ledger::trial_balance(&u.w.env.ctx, None).expect("the trial balance reads");
    assert_eq!(trial.total_dr, trial.total_cr, "{at}: A: the trial balance");
    // Cash in hand is the cash account, read two ways.
    let cash_sql = q(
        conn,
        "SELECT COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id WHERE a.code = '01010001'",
    );
    let cash_in_hand =
        u.w.env
            .ctx
            .db()
            .read(cash::cash_in_hand)
            .expect("cash reads")
            .as_paisa();
    assert_eq!(cash_in_hand, cash_sql, "{at}: A: cash in hand");
}

fn whole_rupees(conn: &Connection, at: &str) {
    assert_eq!(
        q(
            conn,
            "SELECT (SELECT COUNT(*) FROM journal_lines WHERE dr_paisa % 100 <> 0 OR cr_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM bikri_sales WHERE buyer_total_paisa % 100 <> 0
                        OR seller_comm_paisa % 100 <> 0 OR buyer_comm_paisa % 100 <> 0
                        OR laga_paisa % 100 <> 0 OR paid_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM partner_share_rows WHERE amount_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM venture_entry_parts WHERE amount_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM purchase_cost_parts WHERE amount_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM purchases WHERE cost_released_paisa % 100 <> 0
                        OR total_cost_paisa % 100 <> 0)"
        ),
        0,
        "{at}: B: an amount that is not a whole rupee"
    );
}

fn reversals(conn: &Connection, at: &str) {
    // C: an entry and its reversal are zero on every account.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (
               SELECT l.account_id FROM journal_entries r
               JOIN journal_lines l ON l.entry_id IN (r.id, r.reverses_entry_id)
               WHERE r.reverses_entry_id IS NOT NULL
               GROUP BY r.id, l.account_id, COALESCE(l.chukta_person_key, '')
               HAVING SUM(l.dr_paisa) - SUM(l.cr_paisa) <> 0)"
        ),
        0,
        "{at}: C: a reversal is not the mirror of its entry"
    );
    // C: a cancelled بکری nets to 0 on every account over all its postings.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (
               SELECT l.account_id FROM bikri_entries b
               JOIN journal_entries e ON e.source_type = 'bikri' AND e.source_id = b.id
               JOIN journal_lines l ON l.entry_id = e.id
               WHERE b.status = 'cancelled'
               GROUP BY b.id, l.account_id, COALESCE(l.chukta_person_key, '')
               HAVING SUM(l.dr_paisa) - SUM(l.cr_paisa) <> 0)"
        ),
        0,
        "{at}: C: a cancelled بکری does not net to 0"
    );
    // C: a cancelled voucher nets to 0 over its entries.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (
               SELECT l.account_id FROM vouchers v
               JOIN journal_entries e ON e.source_type = 'voucher' AND e.source_id = v.id
               JOIN journal_lines l ON l.entry_id = e.id
               WHERE v.status = 'cancelled'
               GROUP BY v.id, l.account_id, COALESCE(l.chukta_person_key, '')
               HAVING SUM(l.dr_paisa) - SUM(l.cr_paisa) <> 0)"
        ),
        0,
        "{at}: C: a cancelled voucher does not net to 0"
    );
    // D: the share rows of a reversal undo those of its entry, partner by partner.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (
               SELECT s.partner_id FROM journal_entries r
               JOIN partner_share_rows s ON s.journal_entry_id IN (r.id, r.reverses_entry_id)
               WHERE r.reverses_entry_id IS NOT NULL
               GROUP BY r.id, s.partner_id, s.kind
               HAVING SUM(s.amount_paisa) <> 0)"
        ),
        0,
        "{at}: D: the share rows of a reversal do not undo those of its entry"
    );
}

fn shares(u: &U, conn: &Connection, at: &str) {
    // D: the share rows of every entry add up to its income and shop expense lines (FR-6.5).
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM journal_entries e WHERE
               COALESCE((SELECT SUM(amount_paisa) FROM partner_share_rows WHERE journal_entry_id = e.id), 0)
               <> COALESCE((SELECT SUM(l.cr_paisa) - SUM(l.dr_paisa) FROM journal_lines l
                            JOIN accounts a ON a.id = l.account_id
                            JOIN account_controls c ON c.id = a.control_id
                            JOIN account_heads h ON h.id = c.head_id
                            WHERE l.entry_id = e.id AND h.code IN ('04', '05')), 0)"
        ),
        0,
        "{at}: D: an entry's share rows are not its income less its shop expenses"
    );
    let rows = q(
        conn,
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows",
    );
    let net = q(
        conn,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id WHERE h.code IN ('04', '05')",
    );
    assert_eq!(
        rows, net,
        "{at}: D: Σ share rows is not income less expenses"
    );
    // Every share row names an active partner of this world, and an income row is plus.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM partner_share_rows WHERE partner_id NOT IN (SELECT id FROM partners)"
        ),
        0,
        "{at}: D: a share row of an unknown partner"
    );
    // Posting rows of one entry: one row per partner, and the shares of its day total 100%.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (SELECT journal_entry_id, kind, SUM(share_bps) AS s
                FROM partner_share_rows GROUP BY journal_entry_id, kind HAVING s <> 10000)"
        ),
        0,
        "{at}: D: the shares of an entry do not total 100%"
    );
    let _ = u;
}

/// A partner's کل رقم from plain sums (PRD C22), without the advance part.
fn before_advances(conn: &Connection, partner_id: i64) -> i64 {
    let account = |column: &str| -> i64 {
        q(
            conn,
            &format!(
                "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
                 JOIN partners p ON p.{column} = l.account_id WHERE p.id = {partner_id}"
            ),
        )
    };
    let capital = account("capital_account_id");
    let drawings = account("drawings_account_id");
    let ventures = account("venture_account_id");
    let linked = account("linked_account_id");
    let shares = q(
        conn,
        &format!(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE partner_id = {partner_id}"
        ),
    );
    // `drawings` is credits less debits, so money taken is minus.
    capital + shares + drawings + ventures + linked
}

fn equity_and_totals(u: &U, conn: &Connection, at: &str) {
    let overview = partners::overview(&u.w.env.ctx).expect("the partner overview reads");
    let total_of = |id: i64| -> i64 {
        overview
            .iter()
            .find(|r| r.partner_id == id)
            .map_or(0, |r| r.total_paisa.as_paisa())
    };
    let sum: i64 = u.w.partners.iter().map(|p| total_of(*p)).sum();
    let opening = q(
        conn,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id WHERE a.code IN ('06010001', '06019999')",
    );
    let equity = q(
        conn,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id WHERE h.code IN ('04', '05', '06')",
    );
    let advances = open_advances(conn);
    assert!(
        advances >= 0,
        "{at}: E: open advances are below 0: {advances}"
    );
    assert_eq!(
        sum + opening + advances,
        equity,
        "{at}: E: Σ کل رقم {sum} + ابتدائی سرمایہ and فرق {opening} + open advances {advances} is not equity {equity}"
    );

    // F: each partner's part of the open advances is never negative, and together they are the
    // advance still out. The per-partner کل رقم is C22 on plain sums less that part.
    let all = mandi_services::db::advances::read_all(conn).expect("the advance events read");
    let mut parts = 0_i64;
    for p in u.w.partners {
        let net = all.net(p, None, None).expect("the net reads").as_paisa();
        assert!(
            net >= 0,
            "{at}: F: partner {p} has {net} of advances out (below 0)"
        );
        parts += net;
        assert_eq!(
            total_of(p),
            before_advances(conn, p) - net,
            "{at}: E: partner {p}: کل رقم is not the C22 formula on plain sums"
        );
    }
    assert_eq!(
        parts, advances,
        "{at}: F: the partners' parts of the advances are not the advance still out"
    );
    // F: no advance gives back more than it took, partner by partner (C28).
    let mut per: std::collections::BTreeMap<(i64, i64), (i64, i64)> = Default::default();
    for e in &all.events {
        let slot = per.entry((e.line_id, e.partner_id)).or_default();
        match e.kind {
            mandi_services::core::advances::AdvanceEventKind::Out => slot.0 += e.part.as_paisa(),
            mandi_services::core::advances::AdvanceEventKind::Back => slot.1 += e.part.as_paisa(),
        }
    }
    for ((line, partner), (out, back)) in per {
        assert!(
            back <= out,
            "{at}: F: advance line {line} gave partner {partner} back {back} of {out}"
        );
    }
}

fn matching_rows(u: &U, conn: &Connection, at: &str) {
    let w = &u.w;
    // G: a bill allocation pairs a debit with a credit of one account, both in force, and nothing
    // is matched for more than its line holds.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM bill_allocations a
             JOIN journal_lines bl ON bl.id = a.bill_line_id
             JOIN journal_lines pl ON pl.id = a.payment_line_id
             WHERE a.is_active = 1 AND (bl.dr_paisa <= 0 OR pl.cr_paisa <= 0
                   OR bl.account_id <> pl.account_id OR bl.account_id <> a.account_id
                   OR COALESCE(bl.chukta_person_key, '') <> COALESCE(pl.chukta_person_key, ''))"
        ),
        0,
        "{at}: G: a bill allocation does not pair a debit with a credit of one account"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (SELECT l.id FROM journal_lines l
               JOIN bill_allocations a ON a.is_active = 1 AND (a.bill_line_id = l.id OR a.payment_line_id = l.id)
               GROUP BY l.id HAVING SUM(a.amount_paisa) > MAX(l.dr_paisa, l.cr_paisa))"
        ),
        0,
        "{at}: G: a line is matched for more than it holds"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM advance_allocations a
             JOIN journal_lines al ON al.id = a.advance_line_id
             JOIN journal_lines cl ON cl.id = a.credit_line_id
             WHERE a.is_active = 1 AND (al.is_advance <> 1 OR al.dr_paisa <= 0 OR cl.cr_paisa <= 0
                   OR al.account_id <> cl.account_id OR al.account_id <> a.account_id)"
        ),
        0,
        "{at}: G: an advance allocation does not pair an advance with a credit of one seller"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (SELECT l.id FROM journal_lines l
               JOIN advance_allocations a ON a.is_active = 1 AND (a.advance_line_id = l.id OR a.credit_line_id = l.id)
               GROUP BY l.id HAVING SUM(a.amount_paisa) > MAX(l.dr_paisa, l.cr_paisa))"
        ),
        0,
        "{at}: G: an advance or a credit is settled for more than it holds"
    );
    // G: the matching in force belongs to entries in force.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM bill_allocations a
             JOIN journal_lines l ON l.id IN (a.bill_line_id, a.payment_line_id)
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE a.is_active = 1 AND e.status <> 'posted'"
        ),
        0,
        "{at}: G: a matching row of an unposted entry is in force"
    );

    // G: working the matching out again changes nothing, for every buyer that has a line (a seller's
    // bills are not matched; his payables are worked out at read time, PRD FR-7.59) and for the
    // advances of every seller. All the
    // parties are worked out in ONE transaction that is rolled back at the end (an error from the
    // closure), so nothing is committed and no step pays for many fsyncs.
    let now = w.env.ctx.now_text();
    let keyed: Vec<(i64, Option<String>)> = {
        let mut s = conn
            .prepare(
                "SELECT DISTINCT l.account_id, l.chukta_person_key FROM journal_lines l
                 JOIN accounts a ON a.id = l.account_id
                 JOIN account_controls c ON c.id = a.control_id
                 WHERE c.party_type = 'buyer'",
            )
            .unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let sellers: Vec<i64> = {
        let mut s = conn
            .prepare(
                "SELECT DISTINCT l.account_id FROM journal_lines l
                 JOIN accounts a ON a.id = l.account_id
                 JOIN account_controls c ON c.id = a.control_id WHERE c.party_type = 'seller'",
            )
            .unwrap();
        s.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let outcome: Result<(), mandi_services::ServiceError> = w.env.ctx.db().write(|tx| {
        let mut bad: Vec<String> = Vec::new();
        for (account, key) in &keyed {
            let again = matching::rematch(tx, w.owner.user_id, &now, *account, key.as_deref())?;
            if (again.switched_off, again.added) != (0, 0) {
                bad.push(format!("bills of account {account} {key:?}: {again:?}"));
            }
        }
        for seller in &sellers {
            let again = matching::rematch_advances(tx, w.owner.user_id, &now, *seller)?;
            if (again.switched_off, again.added) != (0, 0) {
                bad.push(format!("advances of seller {seller}: {again:?}"));
            }
        }
        Err(mandi_services::ServiceError::Data(format!(
            "rolled back|{}",
            bad.join("; ")
        )))
    });
    match outcome {
        Err(mandi_services::ServiceError::Data(report)) if report.starts_with("rolled back|") => {
            let bad = &report["rolled back|".len()..];
            assert!(
                bad.is_empty(),
                "{at}: G: the stored matching is not the canonical one: {bad}"
            );
        }
        other => panic!("{at}: G: the matching check failed to run: {other:?}"),
    }
}

fn trips(conn: &Connection, at: &str) {
    // H: cost released never goes over the total cost; it is the total when nothing is left (C21).
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM purchases WHERE status = 'saved' AND
               (cost_released_paisa > total_cost_paisa OR cost_released_paisa < 0
                OR qty_sold + qty_written_off > qty
                OR (qty_sold + qty_written_off = qty AND cost_released_paisa <> total_cost_paisa))"
        ),
        0,
        "{at}: H: a purchase released more than its cost, or not all of it when nothing is left"
    );
    // The stock moves of a purchase add up to the stock left.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM purchases p WHERE p.status = 'saved' AND
               (SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE purchase_id = p.id)
               <> p.qty - p.qty_sold - p.qty_written_off"
        ),
        0,
        "{at}: H: the stock moves of a purchase are not its stock left"
    );
    // C24: a live trip cost is spread over the purchases, and the parts add back to it.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM venture_entries e WHERE e.kind = 'cost' AND e.status = 'saved'
               AND (SELECT COALESCE(SUM(amount_paisa), 0) FROM purchase_cost_parts
                    WHERE entry_id = e.id AND is_active = 1) <> e.amount_paisa
               AND EXISTS (SELECT 1 FROM purchase_cost_parts WHERE entry_id = e.id)"
        ),
        0,
        "{at}: H: the parts of a trip cost do not add up to it (C24)"
    );
    // A trip is closed exactly when it has a live purchase and every one has nothing left.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM ventures v WHERE v.kind = 'trip' AND (
               (v.status = 'closed') <>
               (EXISTS (SELECT 1 FROM purchases p WHERE p.venture_id = v.id AND p.status = 'saved')
                AND NOT EXISTS (SELECT 1 FROM purchases p WHERE p.venture_id = v.id AND p.status = 'saved'
                                AND p.qty - p.qty_sold - p.qty_written_off > 0)))"
        ),
        0,
        "{at}: H: a trip is open or closed against what its stock says (FR-12.33)"
    );
}

fn gardens(conn: &Connection, at: &str) {
    // C23, C25: the active parts of a live payment add up to it; so do the parts of a trip's sale in
    // another shop. A garden's sale in another shop and a refund credit the garden's account, not
    // the partners, so they have no parts.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM venture_entries e JOIN ventures v ON v.id = e.venture_id
             WHERE e.status = 'saved'
               AND (e.kind IN ('installment', 'cost', 'purchase') OR (e.kind = 'other_sale' AND v.kind = 'trip'))
               AND e.amount_paisa <> (SELECT COALESCE(SUM(p.amount_paisa), 0) FROM venture_entry_parts p
                                      WHERE p.entry_id = e.id AND p.is_active = 1)"
        ),
        0,
        "{at}: I: the parts of an entry do not add up to it (C23, C25)"
    );
    // One active part per partner per entry.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM (SELECT entry_id, partner_id FROM venture_entry_parts
                WHERE is_active = 1 GROUP BY entry_id, partner_id HAVING COUNT(*) > 1)"
        ),
        0,
        "{at}: I: two parts in force for one partner of one entry"
    );
    // The shop parts of a live payment are what left the cash into the partners' 0604 accounts.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM venture_entries e
             WHERE e.kind IN ('installment', 'cost') AND e.status = 'saved' AND e.journal_entry_id IS NOT NULL
               AND (SELECT COALESCE(SUM(p.amount_paisa), 0) FROM venture_entry_parts p
                    WHERE p.entry_id = e.id AND p.is_active = 1 AND p.source = 'shop')
                <> (SELECT COALESCE(SUM(l.dr_paisa), 0) FROM journal_lines l WHERE l.entry_id = COALESCE(e.current_entry_id, e.journal_entry_id)
                    AND l.account_id IN (SELECT venture_account_id FROM partners))"
        ),
        0,
        "{at}: I: the shop parts of a garden payment are not what the partners' accounts paid"
    );
    // C26: a closed garden's account is 0.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM ventures v WHERE v.kind = 'garden' AND v.status = 'closed' AND
               COALESCE((SELECT SUM(l.cr_paisa - l.dr_paisa) FROM journal_lines l WHERE l.account_id = v.account_id), 0) <> 0"
        ),
        0,
        "{at}: I: a closed garden's account is not 0 (C26)"
    );
}

fn fruit_and_lots(conn: &Connection, at: &str) {
    // C16: no lot has a balance below 0, and its taken quantity is its live fruit entries.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
             WHERE b.status = 'saved' AND (l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken < 0
               OR l.qty_taken <> (SELECT COALESCE(SUM(f.qty), 0) FROM fruit_takes f
                                  WHERE f.lot_id = l.id AND f.status = 'saved'))"
        ),
        0,
        "{at}: J: a lot has a balance below 0 or a taken quantity that is not its fruit entries (C16)"
    );
    // C27: the parts of a live fruit entry add up to its value (the 0604 debits of its journal
    // entry), and the credited account gets the whole value.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM fruit_takes f WHERE f.status = 'saved' AND f.journal_entry_id IS NOT NULL AND (
               (SELECT COALESCE(SUM(l.dr_paisa), 0) FROM journal_lines l WHERE l.entry_id = COALESCE(f.current_entry_id, f.journal_entry_id)
                  AND l.account_id IN (SELECT venture_account_id FROM partners)) <> f.value_paisa
               OR (SELECT COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l WHERE l.entry_id = COALESCE(f.current_entry_id, f.journal_entry_id)
                  AND l.account_id = f.credit_account_id) <> f.value_paisa)"
        ),
        0,
        "{at}: J: the parts of a fruit entry do not add up to its value (C27)"
    );
    // C27: each guest part is at least the value ÷ the partners, rounded down.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM fruit_takes f WHERE f.status = 'saved' AND f.partner_id IS NULL
               AND f.journal_entry_id IS NOT NULL AND EXISTS (
                 SELECT 1 FROM journal_lines l WHERE l.entry_id = COALESCE(f.current_entry_id, f.journal_entry_id)
                   AND l.account_id IN (SELECT venture_account_id FROM partners)
                   AND l.dr_paisa / 100 < (f.value_paisa / 100) / (SELECT COUNT(*) FROM partners))"
        ),
        0,
        "{at}: J: a guest fruit part is below the value divided by the partners (C27)"
    );
}

/// The sum of one account's debits less credits, in paisa.
fn net_dr(conn: &Connection, account: i64) -> i64 {
    q(
        conn,
        &format!(
            "SELECT COALESCE(SUM(dr_paisa - cr_paisa), 0) FROM journal_lines WHERE account_id = {account}"
        ),
    )
}

/// K: the read-time workings of FR-7.59, FR-15.4 and FR-15.8 agree with the ledger, for every buyer
/// and every seller, whatever happened to them (openings, advances, edits, cancels, a dead mark).
fn workings(u: &U, conn: &Connection, at: &str) {
    use mandi_services::core::accounts::BalanceSide;
    use mandi_services::core::vouchers::{OpenBillsInput, OpenPayablesInput};
    use mandi_services::vouchers;
    let signed = |b: mandi_services::core::accounts::Balance| match b.side {
        BalanceSide::Cr => -b.amount_paisa.as_paisa(),
        _ => b.amount_paisa.as_paisa(),
    };
    for buyer in u.buyers() {
        let bills = vouchers::open_bills(&u.w.env.ctx, &OpenBillsInput { account_id: buyer })
            .expect("open bills read");
        assert_eq!(
            bills.open_paisa.as_paisa() - bills.unmatched_credit_paisa.as_paisa(),
            net_dr(conn, buyer),
            "{at}: K: buyer {buyer}: open bills less unmatched credit is not his ledger balance"
        );
        assert_eq!(
            signed(bills.balance),
            net_dr(conn, buyer),
            "{at}: K: buyer {buyer}: balance"
        );
        assert!(
            bills.unmatched_credit_paisa.as_paisa() == 0 || bills.open_paisa.as_paisa() == 0,
            "{at}: K: buyer {buyer}: a credit waits next to an open bill"
        );
    }
    for seller in u.sellers() {
        let p = vouchers::open_payables(
            &u.w.env.ctx,
            &OpenPayablesInput {
                seller_id: seller,
                include_settled: true,
            },
        )
        .expect("open payables read");
        assert_eq!(
            p.open_paisa.as_paisa()
                - p.advances_open_paisa.as_paisa()
                - p.owes_firm_paisa.as_paisa(),
            -net_dr(conn, seller),
            "{at}: K: seller {seller}: open payable less advances less owed is not his ledger"
        );
        assert!(
            p.open_paisa.as_paisa() == 0 || p.owes_firm_paisa.as_paisa() == 0,
            "{at}: K: seller {seller}: he is owed and owes at once"
        );
        let advance = q(
            conn,
            &format!(
                "SELECT COALESCE(SUM(l.dr_paisa - COALESCE((
                     SELECT SUM(a.amount_paisa) FROM advance_allocations a
                     JOIN journal_lines cl ON cl.id = a.credit_line_id
                     JOIN journal_entries ce ON ce.id = cl.entry_id
                     WHERE a.advance_line_id = l.id AND a.is_active = 1 AND ce.status = 'posted'), 0)), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.is_advance = 1 AND l.dr_paisa > 0 AND l.account_id = {seller} AND {LIVE}"
            ),
        );
        assert_eq!(
            p.advances_open_paisa.as_paisa(),
            advance,
            "{at}: K: seller {seller}: advance open is not given less settled"
        );
    }
    // Stock: the moves of a lot add up to received less sold, written off and taken; the facts are
    // the lines in force.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
             WHERE b.status = 'saved' AND (
               l.qty_sold <> (SELECT COALESCE(SUM(qty), 0) FROM bikri_sales WHERE lot_id = l.id AND is_active = 1)
               OR (SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE lot_id = l.id)
                    <> l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken)"
        ),
        0,
        "{at}: K: a lot differs from its lines or its stock moves"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0) FROM analytics_facts"
        ),
        q(
            conn,
            "SELECT COALESCE(SUM(seller_amount_paisa), 0) FROM bikri_sales WHERE is_active = 1"
        ),
        "{at}: K: the analytics facts are not the lines in force"
    );
    // H: a purchase's sold quantity is its own-lot lines in force plus its sales in other shops.
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM purchases p WHERE p.status = 'saved' AND p.qty_sold <>
               COALESCE((SELECT SUM(s.qty) FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
                         WHERE l.purchase_id = p.id AND s.is_active = 1), 0)
             + COALESCE((SELECT SUM(e.qty) FROM venture_entries e
                         WHERE e.purchase_id = p.id AND e.kind = 'other_sale' AND e.status = 'saved'), 0)"
        ),
        0,
        "{at}: H: a purchase's sold quantity is not its own-lot lines plus its other-shop sales"
    );
}

/// L: a person moved to the dead amount is inactive; a mark that was undone leaves him active.
fn dead_amount(conn: &Connection, at: &str) {
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM dead_marks m JOIN accounts a ON a.id = m.account_id
             WHERE m.is_active = 1 AND a.is_active <> 0"
        ),
        0,
        "{at}: L: a person in the dead amount is still active"
    );
    assert_eq!(
        q(
            conn,
            "SELECT COUNT(*) FROM dead_marks m JOIN accounts a ON a.id = m.account_id
             WHERE m.is_active = 0 AND a.is_active <> 1
               AND NOT EXISTS (SELECT 1 FROM dead_marks m2 WHERE m2.account_id = m.account_id AND m2.is_active = 1)"
        ),
        0,
        "{at}: L: a person whose mark was undone is still inactive"
    );
}
