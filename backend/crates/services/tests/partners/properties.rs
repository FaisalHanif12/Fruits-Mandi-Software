//! Property tests with random postings (Code Standards 9.3, PRD FR-6.5, FR-17.4):
//!
//! - every posting's share rows add up to its income base minus its expense base;
//! - over the whole book, the rows add up to income minus shop expenses (head 04 and head 05);
//! - a reversal plus its entry gives zero per partner and per kind;
//! - every row is a whole number of rupees.
//!
//! The shares change on 20-10-26, so both sets of shares are used.

use mandi_services::domain::ledger::PostingType;
use mandi_services::ledger;
use proptest::prelude::*;

use crate::common::{raw, set_today};
use crate::support::*;

#[derive(Debug, Clone)]
struct Step {
    kind: u8,
    first: i64,
    second: i64,
    late: bool,
    pick: usize,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    proptest::collection::vec(
        (
            0u8..7,
            1i64..60_000,
            1i64..60_000,
            any::<bool>(),
            0usize..64,
        )
            .prop_map(|(kind, first, second, late, pick)| Step {
                kind,
                first,
                second,
                late,
                pick,
            }),
        1..9,
    )
}

fn run(steps: &[Step]) {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "33.33"), (b, "33.33"), (c, "33.34")],
    )
    .unwrap();
    set_today(&env, 2026, 10, 20);
    let cash = id_of(&env, "01010001");
    let incomes: Vec<i64> = ["04010001", "04010002", "04010003", "04010004", "04010005"]
        .iter()
        .map(|code| id_of(&env, code))
        .collect();
    let head = expense_head(&env, &owner);
    let capital = id_of(&env, "06010001");
    let mut posted: Vec<i64> = Vec::new();
    let mut reversed: Vec<(i64, i64)> = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let day = if step.late {
            "2026-10-20"
        } else {
            "2026-10-10"
        };
        let income = incomes[index % incomes.len()];
        let lines = match step.kind {
            0 => vec![line(cash, Dr, step.first), line(income, Cr, step.first)],
            1 => vec![line(head, Dr, step.first), line(cash, Cr, step.first)],
            // A negative income (a rate difference below zero).
            2 => vec![line(incomes[4], Dr, step.first), line(cash, Cr, step.first)],
            // Income and the cost of goods sold in one entry (E2E-34 shape). The own-goods
            // accounts come with plan 16b; a shop expense head stands in for the cost.
            3 => vec![
                line(cash, Dr, step.first),
                line(head, Dr, step.second),
                line(income, Cr, step.first),
                line(cash, Cr, step.second),
            ],
            // An entry with no income or expense line.
            4 => vec![line(cash, Dr, step.first), line(capital, Cr, step.first)],
            // A refund credited to a shop expense account.
            5 => vec![line(cash, Dr, step.first), line(head, Cr, step.first)],
            // Reverse an earlier entry that is not a reversal and not reversed yet.
            _ => {
                let open: Vec<i64> = posted
                    .iter()
                    .copied()
                    .filter(|id| !reversed.iter().any(|(original, _)| original == id))
                    .collect();
                if open.is_empty() {
                    continue;
                }
                let original = open[step.pick % open.len()];
                let reversal = env
                    .ctx
                    .db()
                    .write(|tx| {
                        ledger::reverse(&env.ctx, tx, owner.user_id, original, "property")
                            .map(|p| p.entry_id)
                    })
                    .unwrap();
                reversed.push((original, reversal));
                continue;
            }
        };
        let entry = post(&env, &owner, day, PostingType::General, lines).unwrap();
        posted.push(entry);
    }

    let conn = raw(&dir);
    // The bases of every entry, worked out in SQL from its lines and the heads of its accounts.
    let mut bases = conn
        .prepare(
            "SELECT l.entry_id,
                    COALESCE(SUM(CASE WHEN h.code = '04' THEN l.cr_paisa - l.dr_paisa ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN h.code = '05' THEN l.dr_paisa - l.cr_paisa ELSE 0 END), 0)
             FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id
             JOIN account_controls ct ON ct.id = a.control_id
             JOIN account_heads h ON h.id = ct.head_id
             GROUP BY l.entry_id",
        )
        .unwrap();
    let entries: Vec<(i64, i64, i64)> = bases
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let row_sum = |entry: i64| -> i64 {
        conn.query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE journal_entry_id = ?1",
            [entry],
            |r| r.get(0),
        )
        .unwrap()
    };
    let mut book_bases = 0_i64;
    for (entry, income, expense) in &entries {
        assert_eq!(row_sum(*entry), income - expense, "entry {entry}");
        book_bases += income - expense;
    }
    let all_rows: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        all_rows, book_bases,
        "FR-6.5: shares = income - shop expenses"
    );
    let odd: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM partner_share_rows WHERE amount_paisa % 100 <> 0 OR base_paisa % 100 <> 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(odd, 0);
    // A reversal and its entry cancel, partner by partner and kind by kind.
    for (original, reversal) in &reversed {
        let leftover: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                     SELECT partner_id, kind FROM partner_share_rows
                     WHERE journal_entry_id IN (?1, ?2)
                     GROUP BY partner_id, kind
                     HAVING SUM(amount_paisa) <> 0 OR SUM(base_paisa) <> 0)",
                [original, reversal],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(leftover, 0, "reversal {reversal} of {original}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 14,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn rows_add_up_to_income_minus_expense_and_reversals_cancel(steps in steps()) {
        run(&steps);
    }
}
