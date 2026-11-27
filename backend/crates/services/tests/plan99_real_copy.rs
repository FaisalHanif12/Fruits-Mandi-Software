//! Plan 99 on a COPY of the Owner's real database: the one-time fix "پرانے ابتدائی بقایا روکڑ میں درج
//! کریں" (PRD FR-10.9 v1.60) and a garden's sale money (FR-12.53).
//!
//! It is `#[ignore]`: a plain `cargo test` never runs it. It needs `MANDI_REAL_DB`, the path of a
//! COPY of `mandi.db` (never the file the app has open). The copy and its `-wal` and `-shm` files
//! are copied once more into a temp folder, so the files named are never opened. In that temp copy
//! only, the Owner's password is set to the test one and his sign to the default, so the test can
//! sign in. The audit rows the fix writes use the test chain key; the temp copy is thrown away.
//!
//! Run (from `backend/`):
//! `MANDI_REAL_DB=<copy> cargo test -p mandi-services --test plan99_real_copy -- --ignored --nocapture`
//!
//! Worked numbers: openings بنام 125,000 and جمع 40,000 move; cash 1,057,840 becomes 972,840;
//! ابتدائی فرق 85,000 جمع becomes 0. Garden 2: bills 95,880, paid on the spot 64,000, still due
//! 31,880, the garden's credit 99,600 (its money in).
#![cfg(test)]

mod common;

use mandi_services::auth;
use mandi_services::core::accounts::{BalanceSide, OpeningsToCashInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::garden_sale_money::GardensSaleMoneyInput;
use mandi_services::core::gardens::GardensListInput;
use mandi_services::db::rusqlite::Connection;
use mandi_services::ports::PasswordHasher;
use mandi_services::testing::{TestEnv, fast_hasher};
use mandi_services::{Secret, ServiceError, accounts, gardens};

/// Σ بنام − Σ جمع of every account, by id, from the posted entries.
fn balances(conn: &Connection) -> Vec<(i64, i64)> {
    let mut statement = conn
        .prepare(
            "SELECT l.account_id, SUM(l.dr_paisa) - SUM(l.cr_paisa) FROM journal_lines l
             JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
             GROUP BY l.account_id ORDER BY l.account_id",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn net_of(all: &[(i64, i64)], id: i64) -> i64 {
    all.iter().find(|(a, _)| *a == id).map_or(0, |(_, n)| *n)
}

fn id_of(conn: &Connection, code: &str) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
        r.get(0)
    })
    .unwrap()
}

#[test]
#[ignore = "needs MANDI_REAL_DB, a COPY of the Owner's mandi.db"]
fn the_openings_fix_and_a_garden_sale_money_on_a_copy_of_the_real_database() {
    let Ok(source) = std::env::var("MANDI_REAL_DB") else {
        eprintln!("skipped: MANDI_REAL_DB (a COPY of mandi.db) is not set");
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("data").join("mandi.db");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::copy(&source, &path).expect("copy the database");
    for extra in ["-wal", "-shm"] {
        let from = std::path::PathBuf::from(format!("{source}{extra}"));
        if from.exists() {
            std::fs::copy(&from, format!("{}{extra}", path.display())).expect("copy beside it");
        }
    }
    let env = TestEnv::new(root.path()).expect("open the copy (migrations run on it)");
    common::set_today(&env, 2026, 11, 27);
    let raw = common::raw(&root);
    let hash = fast_hasher()
        .unwrap()
        .hash(&Secret::new("secret1"))
        .unwrap();
    raw.execute(
        "UPDATE users SET password_hash = ?1, failed_attempts = 0, locked_until_at = NULL",
        [hash],
    )
    .unwrap();
    raw.execute("DELETE FROM settings WHERE key = 'owner_sign'", [])
        .unwrap();
    let username: String = raw
        .query_row("SELECT username FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();
    auth::login(&env.ctx, &username, &Secret::new("secret1")).unwrap();
    let owner = auth::require(&env.ctx).unwrap();

    // ---- A garden's sale money, before anything changes ----
    let list = gardens::list(
        &env.ctx,
        &GardensListInput {
            from: None,
            to: None,
            status: None,
            partner_id: None,
            search: None,
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    let mut found_garden_2 = false;
    for row in &list.rows {
        let sale = gardens::sale_money(
            &env.ctx,
            &GardensSaleMoneyInput {
                garden_id: row.id,
                from: None,
                to: None,
                arrival_id: None,
                search: None,
                offset: 0,
                limit: 100,
            },
        )
        .unwrap();
        let t = &sale.totals;
        eprintln!(
            "garden {} {}: rows {}, bill {}, paid {}, collected {}, due {}, credit {}",
            row.number,
            row.name_en,
            sale.total_count,
            t.buyer_total_paisa.rupees(),
            t.paid_paisa.rupees(),
            t.collected_paisa.rupees(),
            t.still_due_paisa.rupees(),
            t.garden_credit_paisa.rupees()
        );
        for line in &sale.rows {
            eprintln!(
                "  {} {:?} bikri {:?} buyer {:?} shop {:?} qty {:?} value {:?} bill {:?} paid {:?} collected {:?} due {:?} credit {}",
                line.date,
                line.kind,
                line.bikri_number,
                line.buyer.as_ref().map(|b| b.name_en.clone()),
                line.shop_name,
                line.qty,
                line.value_paisa.map(|v| v.rupees()),
                line.buyer_total_paisa.map(|v| v.rupees()),
                line.paid_paisa.map(|v| v.rupees()),
                line.collected_paisa.map(|v| v.rupees()),
                line.still_due_paisa.map(|v| v.rupees()),
                line.garden_credit_paisa.rupees()
            );
        }
        // The garden's credit is its money in from sales: nothing else came in on these gardens.
        assert_eq!(
            t.garden_credit_paisa, row.money_in_paisa,
            "garden {}",
            row.number
        );
        if t.buyer_total_paisa.rupees() == 95_880 {
            found_garden_2 = true;
            assert_eq!(t.paid_paisa.rupees(), 64_000);
            assert_eq!(t.still_due_paisa.rupees(), 31_880);
            assert_eq!(t.garden_credit_paisa.rupees(), 99_600);
        }
    }
    assert!(found_garden_2, "the garden with bills of 95,880");

    // ---- The one-time fix ----
    let cash = id_of(&raw, "01010001");
    let difference = id_of(&raw, "06019999");
    let before = balances(&raw);
    assert_eq!(net_of(&before, cash), 105_784_000);
    assert_eq!(net_of(&before, difference), -8_500_000);
    let shares_before: i64 = raw
        .query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let closed_before: Option<String> = raw
        .query_row("SELECT closed_up_to_date FROM day_state", [], |r| r.get(0))
        .unwrap();

    let preview = accounts::openings_to_cash_preview(&env.ctx).unwrap();
    for row in &preview.rows {
        eprintln!(
            "move {} {} {} dr {} cr {} closed {}",
            row.date,
            row.account.code,
            row.account.name_en,
            row.dr_paisa.rupees(),
            row.cr_paisa.rupees(),
            row.day_closed
        );
    }
    assert_eq!(preview.rows.len(), 7);
    assert_eq!(preview.total_dr_paisa.rupees(), 125_000);
    assert_eq!(preview.total_cr_paisa.rupees(), 40_000);
    assert_eq!(preview.cash_before_paisa.rupees(), 1_057_840);
    assert_eq!(preview.cash_after_paisa.rupees(), 972_840);
    assert!(!preview.needs_cash_confirm);
    assert_eq!(
        preview.opening_difference_before.amount_paisa.rupees(),
        85_000
    );
    assert_eq!(preview.opening_difference_before.side, BalanceSide::Cr);
    assert_eq!(preview.opening_difference_after.amount_paisa.rupees(), 0);

    let input = |sign: &str| OpeningsToCashInput {
        reason: "openings through the cash book".into(),
        sign: sign.into(),
        confirm_cash_below_zero: false,
    };
    match accounts::openings_to_cash(&env.ctx, &owner, &input("munshi")) {
        Err(ServiceError::Rejected(items)) => assert_eq!(items[0].code, ErrorCode::ErrSignWrong),
        other => panic!("{other:?}"),
    }
    assert_eq!(balances(&raw), before, "a wrong sign changes nothing");

    let done = accounts::openings_to_cash(&env.ctx, &owner, &input("adnan")).unwrap();
    assert_eq!(done.cash_after_paisa.rupees(), 972_840);
    assert_eq!(done.opening_difference_after.amount_paisa.rupees(), 0);
    assert_eq!(done.closed_up_to, closed_before);

    let after = balances(&raw);
    assert_eq!(net_of(&after, cash), 97_284_000);
    assert_eq!(net_of(&after, difference), 0);
    // Nothing else moves: every other account keeps its balance.
    let others = |all: &[(i64, i64)]| -> Vec<(i64, i64)> {
        all.iter()
            .copied()
            .filter(|(id, n)| *id != cash && *id != difference && *n != 0)
            .collect()
    };
    assert_eq!(others(&after), others(&before));
    assert_eq!(
        after.iter().map(|(_, n)| n).sum::<i64>(),
        0,
        "trial balance"
    );
    let shares_after: i64 = raw
        .query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(shares_after, shares_before);

    match accounts::openings_to_cash(&env.ctx, &owner, &input("adnan")) {
        Err(ServiceError::Rejected(items)) => {
            assert_eq!(items[0].code, ErrorCode::ErrOpeningsAlreadyInCash);
        }
        other => panic!("{other:?}"),
    }
}
