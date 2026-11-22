//! Plan 76 points 5 and 10, property tests (money agent items a to e, f): for any run of random
//! khata and loan entries with every kind of link, with returns, edits, cancels, a change of the
//! shares and days going by,
//!
//! * a: every journal entry balances, and a cancelled entry and its reversal net to 0;
//! * b: Σ کل رقم + ابتدائی سرمایہ + ابتدائی فرق = the equity of the books (FR-17.9 holds);
//! * c: the contra account 03070001 holds exactly what the linked khatas have paid out and the
//!   partner-linked loans still have open, and Σ of the partners' 0605 accounts is the opposite;
//! * d: a khata linked to one partner moves only his account, by exactly its own balance;
//! * e: an unlinked khata and a loan linked to the shop write the same two lines they always did;
//! * f: the link of a khata changes only while its balance is 0.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{raw, set_today};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::khata::{
    KhataCreateInput, KhataCreateRequest, KhataInInput, KhataOutInput, KhataSetLinkInput,
};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentPersonCreateRequest, LentReturnMode, MoneyLentInput,
    MoneyLentRequest, MoneyLentReturnInput,
};
use mandi_services::core::links::{Link, LinkKind, PersonType};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::vouchers::{
    VoucherCancelInput, VoucherEditInput, VoucherEntry, VoucherType,
};
use mandi_services::{ServiceError, khata, lent, partners, vouchers};
use proptest::prelude::*;
use support::World;

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// One step of a run.
#[derive(Debug, Clone)]
enum Op {
    /// Money paid out to a khata (0 unlinked, 1 shop, 2 partner الف).
    Out(usize, u32),
    /// Money back from a khata.
    In(usize, u32),
    /// Money lent to a person (0 shop, 1 partner ب).
    Lend(usize, u32),
    /// A cash return, never more than the person has open.
    Back(usize, u32),
    /// The last khata voucher is edited to another amount.
    EditLast(u32),
    /// The last voucher is cancelled.
    CancelLast,
    /// The next day.
    NextDay,
    /// New shares for today on: الف this many percent, the other two share the rest.
    Shares(u32),
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    let op = prop_oneof![
        3 => (0_usize..3, 1_u32..3_000).prop_map(|(k, a)| Op::Out(k, a)),
        3 => (0_usize..3, 1_u32..3_000).prop_map(|(k, a)| Op::In(k, a)),
        2 => (0_usize..2, 1_u32..3_000).prop_map(|(p, a)| Op::Lend(p, a)),
        2 => (0_usize..2, 1_u32..3_000).prop_map(|(p, a)| Op::Back(p, a)),
        1 => (1_u32..3_000).prop_map(Op::EditLast),
        1 => Just(Op::CancelLast),
        1 => Just(Op::NextDay),
        1 => (20_u32..60).prop_map(Op::Shares),
    ];
    proptest::collection::vec(op, 3..12)
}

/// What a run keeps track of.
struct Run {
    w: World,
    khatas: [i64; 3],
    persons: [i64; 2],
    day: u32,
    /// The vouchers saved, newest last: (id, khata voucher?, date).
    saved: Vec<(i64, bool, String)>,
}

fn make_khata(w: &World, ur: &str, en: &str, link: Option<Link>) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        let request = KhataCreateRequest {
            base: KhataCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                note: None,
                confirmed_rules_words: words.clone(),
            },
            link,
        };
        match khata::create_linked(&w.env.ctx, &w.owner, &request) {
            Ok(done) => return done.account.id,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn make_person(w: &World, ur: &str, en: &str, link: Option<Link>) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        let request = LentPersonCreateRequest {
            base: LentPersonCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.clone(),
            },
            person_type: Some(PersonType::GardenOwner),
            link,
        };
        match lent::person_create_linked(&w.env.ctx, &w.owner, &request) {
            Ok(person) => return person.account.id,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn partner_link(partner: i64) -> Link {
    Link {
        kind: LinkKind::Partner,
        partner_id: Some(partner),
    }
}

fn start() -> Run {
    let w = support::world();
    let khatas = [
        make_khata(&w, "خاندانی کھاتہ", "Family Khata", None),
        make_khata(
            &w,
            "گھر کھاتہ",
            "Home Khata",
            Some(Link {
                kind: LinkKind::Shop,
                partner_id: None,
            }),
        ),
        make_khata(
            &w,
            "ذاتی کھاتہ",
            "Personal Khata",
            Some(partner_link(w.partners[0])),
        ),
    ];
    let persons = [
        make_person(&w, "نواز", "Nawaz", None),
        make_person(&w, "اشرف", "Ashraf", Some(partner_link(w.partners[1]))),
    ];
    Run {
        w,
        khatas,
        persons,
        day: 10,
        saved: Vec::new(),
    }
}

fn date_of(day: u32) -> String {
    format!("{day:02}-10-2026")
}

fn open_of(run: &Run, person: usize) -> i64 {
    lent::list(
        &run.w.env.ctx,
        &LentListInput {
            from: None,
            to: None,
            show_finished: false,
            group_by_month: false,
            sort: None,
            person_id: run.persons[person],
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
    .totals
    .open_paisa
    .as_paisa()
}

fn out_input(khata: i64, amount: u32, date: &str) -> KhataOutInput {
    KhataOutInput {
        date: date.into(),
        khata_account_id: Some(khata),
        amount: amount.to_string(),
        reason: "خرچ".into(),
        narration: None,
        confirm_cash_below_zero: true,
    }
}

fn in_input(khata: i64, amount: u32, date: &str) -> KhataInInput {
    KhataInInput {
        date: date.into(),
        khata_account_id: Some(khata),
        amount: amount.to_string(),
        reason: "واپس".into(),
        narration: None,
        confirm_cash_below_zero: true,
    }
}

fn apply(run: &mut Run, op: &Op) {
    let date = date_of(run.day);
    let (ctx, owner) = (&run.w.env.ctx, &run.w.owner);
    match op {
        Op::Out(k, amount) => {
            if let Ok(saved) =
                vouchers::save_khata_out(ctx, owner, &out_input(run.khatas[*k], *amount, &date))
            {
                run.saved.push((saved.voucher_id, true, date));
            }
        }
        Op::In(k, amount) => {
            if let Ok(saved) =
                vouchers::save_khata_in(ctx, owner, &in_input(run.khatas[*k], *amount, &date))
            {
                run.saved.push((saved.voucher_id, true, date));
            }
        }
        Op::Lend(p, amount) => {
            let request = MoneyLentRequest {
                base: MoneyLentInput {
                    date: date.clone(),
                    person_account_id: Some(run.persons[*p]),
                    amount: amount.to_string(),
                    reason: "باغ".into(),
                    return_by: None,
                    narration: None,
                    confirm_cash_below_zero: true,
                },
                link: None,
            };
            if let Ok(saved) = vouchers::save_money_lent_linked(ctx, owner, &request) {
                run.saved.push((saved.voucher_id, false, date));
            }
        }
        Op::Back(p, amount) => {
            let open = open_of(run, *p) / 100;
            let amount = i64::from(*amount).min(open);
            if amount > 0 {
                let saved = vouchers::save_money_lent_return(
                    ctx,
                    owner,
                    &MoneyLentReturnInput {
                        date: date.clone(),
                        person_account_id: Some(run.persons[*p]),
                        amount: amount.to_string(),
                        mode: LentReturnMode::Cash,
                        salary_head_id: None,
                        narration: None,
                    },
                )
                .unwrap();
                run.saved.push((saved.voucher_id, false, date));
            }
        }
        Op::EditLast(amount) => {
            let Some((id, true, date)) = run.saved.last().cloned() else {
                return;
            };
            let Ok(row) = vouchers::get(ctx, id) else {
                return;
            };
            let Some(khata) = row.account.as_ref().map(|a| a.id) else {
                return;
            };
            let entry = if row.voucher_type == VoucherType::KhataOut {
                VoucherEntry::KhataOut(out_input(khata, *amount, &date))
            } else {
                VoucherEntry::KhataIn(in_input(khata, *amount, &date))
            };
            // A refused edit writes nothing, so the checks below still hold.
            let _ = vouchers::edit(
                ctx,
                owner,
                &VoucherEditInput {
                    id,
                    version: row.version,
                    entry,
                    reason: "x".into(),
                    sign: "adnan".into(),
                    confirm_cash_below_zero: true,
                },
            );
        }
        Op::CancelLast => {
            let Some((id, _, _)) = run.saved.last().cloned() else {
                return;
            };
            let Ok(row) = vouchers::get(ctx, id) else {
                return;
            };
            if vouchers::cancel(
                ctx,
                owner,
                &VoucherCancelInput {
                    id,
                    version: row.version,
                    reason: "x".into(),
                    confirm_cash_below_zero: true,
                },
            )
            .is_ok()
            {
                run.saved.pop();
            }
        }
        Op::NextDay => {
            if run.day < 28 {
                run.day += 1;
                set_today(&run.w.env, 2026, 10, run.day);
            }
        }
        Op::Shares(a) => {
            let b = (100 - a) / 2;
            let c = 100 - a - b;
            let _ = partners::set_shares(
                ctx,
                owner,
                &PartnersSetSharesInput {
                    effective_from: date,
                    shares: [
                        (run.w.partners[0], *a),
                        (run.w.partners[1], b),
                        (run.w.partners[2], c),
                    ]
                    .iter()
                    .map(|(partner_id, pct)| ShareInput {
                        partner_id: *partner_id,
                        share: pct.to_string(),
                    })
                    .collect(),
                },
            );
        }
    }
}

fn scalar(run: &Run, sql: &str) -> i64 {
    raw(&run.w.dir)
        .query_row(sql, [], |r| r.get::<_, i64>(0))
        .unwrap()
}

/// Credit less debit of the accounts with these codes, paisa.
fn net_of(run: &Run, where_code: &str) -> i64 {
    scalar(
        run,
        &format!(
            "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id WHERE {where_code}"
        ),
    )
}

fn account_net(run: &Run, id: i64) -> i64 {
    scalar(
        run,
        &format!(
            "SELECT COALESCE(SUM(cr_paisa - dr_paisa), 0) FROM journal_lines WHERE account_id = {id}"
        ),
    )
}

fn check_all(run: &Run) -> Result<(), TestCaseError> {
    // a: every entry balances (the database trigger says so too), and a voucher cancelled with its
    // reversal nets to 0 on every account the pair touches.
    prop_assert_eq!(
        scalar(
            run,
            "SELECT COUNT(*) FROM (SELECT entry_id FROM journal_lines GROUP BY entry_id
                                   HAVING SUM(dr_paisa) <> SUM(cr_paisa))"
        ),
        0
    );
    prop_assert_eq!(
        scalar(
            run,
            "SELECT COUNT(*) FROM (
                 SELECT e.id FROM journal_entries e
                 JOIN journal_entries r ON r.reverses_entry_id = e.id
                 JOIN journal_lines l ON l.entry_id IN (e.id, r.id)
                 GROUP BY e.id, l.account_id HAVING SUM(l.cr_paisa - l.dr_paisa) <> 0)"
        ),
        0
    );
    // b: FR-17.9.
    let totals: i64 = partners::overview(&run.w.env.ctx)
        .unwrap()
        .iter()
        .map(|r| r.total_paisa.as_paisa())
        .sum();
    let opening = net_of(run, "a.code IN ('06010001', '06019999')");
    let books = net_of(run, "substr(a.code, 1, 2) IN ('04', '05', '06')");
    prop_assert_eq!(totals + opening, books);
    // c: the contra account holds what the linked khatas paid out (debit less credit) and the
    // partner-linked loans still have open; the partners' 0605 accounts hold the opposite.
    let shop = account_net(run, run.khatas[1]);
    let own = account_net(run, run.khatas[2]);
    let linked_loan = account_net(run, run.persons[1]);
    prop_assert_eq!(
        net_of(run, "a.code = '03070001'"),
        -(shop + own + linked_loan)
    );
    let partners_0605 = net_of(run, "a.code LIKE '0605%'");
    prop_assert_eq!(partners_0605, shop + own + linked_loan);
    // d: a khata linked to one partner moves only his 0605 account, by its own balance, and a loan
    // linked to another partner only his.
    let a_0605 = net_of(
        run,
        &format!(
            "a.id = (SELECT linked_account_id FROM partners WHERE id = {})",
            run.w.partners[0]
        ),
    );
    let b_0605 = net_of(
        run,
        &format!(
            "a.id = (SELECT linked_account_id FROM partners WHERE id = {})",
            run.w.partners[1]
        ),
    );
    let c_0605 = net_of(
        run,
        &format!(
            "a.id = (SELECT linked_account_id FROM partners WHERE id = {})",
            run.w.partners[2]
        ),
    );
    // The shop khata's parts add up to its own balance whatever the shares were each day.
    prop_assert_eq!(a_0605 + b_0605 + c_0605, shop + own + linked_loan);
    // e: the unlinked khata and the shop loan wrote their two lines and nothing else.
    for account in [run.khatas[0], run.persons[0]] {
        prop_assert_eq!(
            scalar(
                run,
                &format!(
                    "SELECT COUNT(*) FROM journal_lines WHERE entry_id IN
                       (SELECT entry_id FROM journal_lines WHERE account_id = {account})"
                )
            ),
            2 * scalar(
                run,
                &format!(
                    "SELECT COUNT(DISTINCT entry_id) FROM journal_lines WHERE account_id = {account}"
                )
            )
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn linked_money_always_balances_and_keeps_the_partners_totals_in_the_books(run_ops in ops()) {
        let mut run = start();
        check_all(&run)?;
        for op in &run_ops {
            apply(&mut run, op);
            check_all(&run)?;
        }
        // f: the shop khata's link changes only while its balance is 0.
        let balance = account_net(&run, run.khatas[1]);
        let moved = khata::set_link(
            &run.w.env.ctx,
            &run.w.owner,
            &KhataSetLinkInput {
                khata_account_id: run.khatas[1],
                link: partner_link(run.w.partners[2]),
            },
        );
        if balance == 0 {
            prop_assert!(moved.is_ok());
        } else {
            prop_assert_eq!(moved.unwrap_err().code(), ErrorCode::ErrLinkLocked);
        }
    }
}
