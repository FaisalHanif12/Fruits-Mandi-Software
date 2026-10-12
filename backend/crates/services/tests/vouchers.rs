//! The voucher framework and the پیشگی voucher (plan 10, PRD 15.1, 15.4, FR-15.1, FR-7.60,
//! acceptance D2-A14): the checks, the posting, the numbering, the running list, cash and bank,
//! reversal and the open advances.
#![cfg(test)]

mod common;

use std::time::Instant;

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType, Balance, BalanceSide};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::ids::AccountId;
use mandi_services::core::masters::{BankAccountInput, MasterInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesInput, VoucherMode, VoucherType, VouchersListInput,
};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, assist, auth, masters, vouchers};
use tempfile::TempDir;

struct Fixture {
    dir: TempDir,
    env: TestEnv,
    owner: auth::Actor,
    rahim: i64,
    karim: i64,
    cash: i64,
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn account_input(kind: AccountType, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
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
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    }
}

/// Rahim and Karim, cash 50,000 (an opening of 50,000 against ابتدائی فرق), today 10-10-26.
fn fixture() -> Fixture {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(
        &env.ctx,
        &owner,
        &account_input(AccountType::Seller, "رحیم", "Rahim"),
    )
    .unwrap()
    .id;
    let karim = accounts::save(
        &env.ctx,
        &owner,
        &account_input(AccountType::Seller, "کریم", "Karim"),
    )
    .unwrap()
    .id;
    let cash = id_of(&env, "01010001");
    set_opening(&env, &owner, cash, "50,000", "", false);
    Fixture {
        dir,
        env,
        owner,
        rahim,
        karim,
        cash,
    }
}

fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap_or_else(|| panic!("no account {code}"))
}

fn set_opening(env: &TestEnv, owner: &auth::Actor, id: i64, dr: &str, cr: &str, advance: bool) {
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    accounts::openings_save(
        &env.ctx,
        owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: id,
                version: accounts::get(&env.ctx, id).unwrap().version,
                opening_date: "01-09-2026".into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: advance,
            }],
        },
    )
    .unwrap();
}

fn advance(seller: i64, amount: &str) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: seller,
        amount: amount.into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: String::new(),
    }
}

fn first_error(error: ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn balance(env: &TestEnv, id: i64) -> (BalanceSide, i64) {
    let b: Balance = ledger::account_balance(&env.ctx, id).unwrap();
    (b.side, b.amount_paisa.rupees())
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn an_advance_debits_the_seller_as_an_advance_and_credits_cash() {
    // PRD 15.4 and 4.3: رحیم takes 15,000 cash. Cash 50,000 becomes 35,000.
    let f = fixture();
    let saved = vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "15,000")).unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Advance, 1)
    );
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(15_000))
    );
    assert_eq!(balance(&f.env, f.rahim), (BalanceSide::Dr, 15_000));
    assert_eq!(balance(&f.env, f.cash), (BalanceSide::Dr, 35_000));
    // The entry: posted, type advance, source voucher; the seller line is the advance line.
    let (kind, source, source_id, total, status): (String, String, i64, i64, String) = raw(&f.dir)
        .query_row(
            "SELECT posting_type, source_type, source_id, total_paisa, status FROM journal_entries WHERE id = ?1",
            [saved.entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();
    assert_eq!(
        (
            kind.as_str(),
            source.as_str(),
            source_id,
            total,
            status.as_str()
        ),
        ("advance", "voucher", saved.voucher_id, 1_500_000, "posted")
    );
    let lines: Vec<(i64, i64, i64, i64)> = raw(&f.dir)
        .prepare("SELECT account_id, dr_paisa, cr_paisa, is_advance FROM journal_lines WHERE entry_id = ?1 ORDER BY line_no")
        .unwrap()
        .query_map([saved.entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        lines,
        [(f.rahim, 1_500_000, 0, 1), (f.cash, 0, 1_500_000, 0)]
    );
    // The voucher row holds the entry; the audit row says who and what.
    let (current, number, mode): (i64, i64, String) = raw(&f.dir)
        .query_row(
            "SELECT current_entry_id, number, mode FROM vouchers WHERE id = ?1",
            [saved.voucher_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (current, number, mode.as_str()),
        (saved.entry_id, 1, "cash")
    );
    let new_json: String = raw(&f.dir)
        .query_row(
            "SELECT new_json FROM audit_log WHERE entity = 'voucher' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        new_json.contains("\"amount_paisa\":1500000") && new_json.contains("\"number\":1"),
        "{new_json}"
    );
    // No income or expense account is touched, so no partner share (PRD FR-6.5).
    assert_eq!(count(&f.dir, "SELECT COUNT(*) FROM partner_share_rows"), 0);
    ledger::verify_journal_chain(&f.env.ctx).unwrap();
    assert_chain_ok(&f.dir);
    // Numbers go on: the next voucher is پ-2.
    assert_eq!(
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "1000"))
            .unwrap()
            .number,
        2
    );
}

#[test]
fn every_mistake_names_its_field_and_nothing_is_written() {
    let f = fixture();
    let before = (
        count(&f.dir, "SELECT COUNT(*) FROM vouchers"),
        count(&f.dir, "SELECT COUNT(*) FROM journal_entries"),
        audit_total(&f.dir),
    );
    let pukhta = id_of(&f.env, "030119999");
    let buyer = accounts::save(
        &f.env.ctx,
        &f.owner,
        &account_input(AccountType::Buyer, "اکرم ٹول ٹیکس", "Akram Toll Tax"),
    )
    .unwrap()
    .id;
    let off = accounts::save(
        &f.env.ctx,
        &f.owner,
        &account_input(AccountType::Seller, "ناصر", "Nasir"),
    )
    .unwrap();
    accounts::set_active(&f.env.ctx, &f.owner, off.id, false).unwrap();
    let cases: Vec<(AdvanceInput, ErrorCode, &str)> = vec![
        (
            advance(f.rahim, "0"),
            ErrorCode::ErrMustBeMoreThanZero,
            "amount",
        ),
        (
            advance(f.rahim, "-5"),
            ErrorCode::ErrNegativeNotAllowed,
            "amount",
        ),
        (
            advance(f.rahim, "15000.50"),
            ErrorCode::ErrDecimalsNotAllowed,
            "amount",
        ),
        (advance(f.rahim, "abc"), ErrorCode::ErrNotANumber, "amount"),
        (advance(f.rahim, ""), ErrorCode::ErrRequired, "amount"),
        (
            advance(f.rahim, "10,000,000,001"),
            ErrorCode::ErrAmountTooLarge,
            "amount",
        ),
        (
            AdvanceInput {
                date: "31-02-2026".into(),
                ..advance(f.rahim, "100")
            },
            ErrorCode::ErrDateInvalid,
            "date",
        ),
        (
            AdvanceInput {
                date: "11-10-2026".into(),
                ..advance(f.rahim, "100")
            },
            ErrorCode::ErrDateInFuture,
            "date",
        ),
        (
            advance(buyer, "100"),
            ErrorCode::ErrNotASeller,
            "seller_account_id",
        ),
        (
            advance(pukhta, "100"),
            ErrorCode::ErrNotASeller,
            "seller_account_id",
        ),
        (
            advance(off.id, "100"),
            ErrorCode::ErrAccountInactive,
            "seller_account_id",
        ),
        (
            advance(999_999, "100"),
            ErrorCode::ErrNotFound,
            "seller_account_id",
        ),
        // Cash with a bank account; bank with none; a bank that does not exist.
        (
            AdvanceInput {
                bank_account_id: Some(1),
                ..advance(f.rahim, "100")
            },
            ErrorCode::ErrFormatInvalid,
            "bank_account_id",
        ),
        (
            AdvanceInput {
                mode: VoucherMode::Bank,
                ..advance(f.rahim, "100")
            },
            ErrorCode::ErrRequired,
            "bank_account_id",
        ),
        (
            AdvanceInput {
                mode: VoucherMode::Bank,
                bank_account_id: Some(999),
                ..advance(f.rahim, "100")
            },
            ErrorCode::ErrNotFound,
            "bank_account_id",
        ),
    ];
    for (input, code, field) in cases {
        let (got, at) =
            first_error(vouchers::save_advance(&f.env.ctx, &f.owner, &input).unwrap_err());
        assert_eq!(
            (got, at.as_deref()),
            (code, Some(field)),
            "{} {}",
            input.amount,
            input.date
        );
    }
    let after = (
        count(&f.dir, "SELECT COUNT(*) FROM vouchers"),
        count(&f.dir, "SELECT COUNT(*) FROM journal_entries"),
        audit_total(&f.dir),
    );
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    // The only audit rows written are those of the setup steps in this test (the buyer, the
    // seller and its switch off), none from the refused vouchers.
    assert_eq!(after.2, before.2 + 3);
    // Several mistakes come back together.
    let ServiceError::Rejected(items) = vouchers::save_advance(
        &f.env.ctx,
        &f.owner,
        &AdvanceInput {
            date: "x".into(),
            ..advance(buyer, "")
        },
    )
    .unwrap_err() else {
        panic!("not a rejection")
    };
    assert!(items.len() >= 3);
}

#[test]
fn a_closed_day_refuses_a_voucher_until_it_is_reopened() {
    let f = fixture();
    raw(&f.dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-09' WHERE id = 1",
            [],
        )
        .unwrap();
    let (code, field) = first_error(
        vouchers::save_advance(
            &f.env.ctx,
            &f.owner,
            &AdvanceInput {
                date: "09-10-2026".into(),
                ..advance(f.rahim, "100")
            },
        )
        .unwrap_err(),
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDateClosed, Some("date"))
    );
    vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "100")).unwrap();
}

#[test]
fn bank_and_cheque_post_to_the_bank_account() {
    // PRD 15.1: bank money posts to the bank account at once; a cheque needs its number and date.
    let f = fixture();
    let bank = masters::save(
        &f.env.ctx,
        &f.owner,
        &MasterInput::BankAccount(BankAccountInput {
            id: None,
            version: None,
            bank_name: "HBL".into(),
            title: "Maher Adnan".into(),
            number: "1234-5678".into(),
            name_ur: "ایچ بی ایل".into(),
            name_en: "HBL Bank".into(),
            confirmed_rules_words: vec!["hbl".into(), "ایچ".into(), "بی".into(), "ایل".into()],
        }),
    );
    let bank = match bank {
        Ok(row) => row,
        Err(ServiceError::Rejected(items)) => {
            // The names may ask for the spelling check; confirm every word asked about.
            let words: Vec<String> = items
                .iter()
                .filter_map(|i| match i.values.get("words") {
                    Some(mandi_services::core::error::ErrorValue::Text(w)) => Some(w.clone()),
                    _ => None,
                })
                .flat_map(|w| w.split(' ').map(str::to_owned).collect::<Vec<_>>())
                .collect();
            masters::save(
                &f.env.ctx,
                &f.owner,
                &MasterInput::BankAccount(BankAccountInput {
                    id: None,
                    version: None,
                    bank_name: "HBL".into(),
                    title: "Maher Adnan".into(),
                    number: "1234-5678".into(),
                    name_ur: "ایچ بی ایل".into(),
                    name_en: "HBL Bank".into(),
                    confirmed_rules_words: words,
                }),
            )
            .unwrap()
        }
        Err(other) => panic!("{other:?}"),
    };
    let bank_row = bank.id();
    let bank_ledger: i64 = raw(&f.dir)
        .query_row(
            "SELECT account_id FROM bank_accounts WHERE id = ?1",
            [bank_row],
            |r| r.get(0),
        )
        .unwrap();
    set_opening(&f.env, &f.owner, bank_ledger, "20,000", "", false);
    let by_bank = AdvanceInput {
        mode: VoucherMode::Bank,
        bank_account_id: Some(bank_row),
        ..advance(f.rahim, "5,000")
    };
    let saved = vouchers::save_advance(&f.env.ctx, &f.owner, &by_bank).unwrap();
    assert_eq!(balance(&f.env, bank_ledger), (BalanceSide::Dr, 15_000));
    assert_eq!(
        balance(&f.env, f.cash),
        (BalanceSide::Dr, 50_000),
        "cash is not touched"
    );
    let (mode, bank_id): (String, Option<i64>) = raw(&f.dir)
        .query_row(
            "SELECT mode, bank_account_id FROM vouchers WHERE id = ?1",
            [saved.voucher_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((mode.as_str(), bank_id), ("bank", Some(bank_row)));
    // A cheque needs its number and its date.
    let cheque = AdvanceInput {
        mode: VoucherMode::Cheque,
        ..by_bank.clone()
    };
    let (code, field) =
        first_error(vouchers::save_advance(&f.env.ctx, &f.owner, &cheque).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrRequired, Some("cheque_no"))
    );
    let (code, field) = first_error(
        vouchers::save_advance(
            &f.env.ctx,
            &f.owner,
            &AdvanceInput {
                cheque_no: "000123".into(),
                ..cheque.clone()
            },
        )
        .unwrap_err(),
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrRequired, Some("cheque_date"))
    );
    let ok = vouchers::save_advance(
        &f.env.ctx,
        &f.owner,
        &AdvanceInput {
            cheque_no: "000123".into(),
            cheque_date: "12-10-2026".into(),
            ..cheque
        },
    )
    .unwrap();
    assert_eq!(ok.number, 2);
    assert_eq!(
        balance(&f.env, bank_ledger),
        (BalanceSide::Dr, 10_000),
        "a cheque posts to the bank at once"
    );
    let (no, date): (String, String) = raw(&f.dir)
        .query_row(
            "SELECT cheque_no, cheque_date FROM vouchers WHERE id = ?1",
            [ok.voucher_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((no.as_str(), date.as_str()), ("000123", "2026-10-12"));
    // A bank that is switched off cannot be used.
    masters::set_active(
        &f.env.ctx,
        &f.owner,
        mandi_services::core::masters::MasterKind::BankAccount,
        bank_row,
        false,
    )
    .unwrap();
    let (code, field) =
        first_error(vouchers::save_advance(&f.env.ctx, &f.owner, &by_bank).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrAccountInactive, Some("bank_account_id"))
    );
}

#[test]
fn the_running_list_shows_balances_and_saves_each_voucher_alone() {
    // PRD FR-15.1 and the money review: two lines for the same seller, a failing line in the
    // middle that moves nothing and takes no number.
    let f = fixture();
    set_opening(&f.env, &f.owner, f.rahim, "15,000", "", true);
    let list = AdvancesInput {
        lines: vec![
            advance(f.rahim, "5,000"),
            advance(f.rahim, "5,000.50"),
            advance(f.rahim, "3,000"),
            advance(f.karim, "1,000"),
        ],
    };
    let preview = vouchers::preview_advances(&f.env.ctx, &list).unwrap();
    let shown: Vec<(BalanceSide, i64, BalanceSide, i64)> = preview
        .rows
        .iter()
        .map(|r| {
            (
                r.current_balance.side,
                r.current_balance.amount_paisa.rupees(),
                r.new_balance.side,
                r.new_balance.amount_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (BalanceSide::Dr, 15_000, BalanceSide::Dr, 20_000),
            (BalanceSide::Dr, 20_000, BalanceSide::Dr, 20_000), // refused: moves nothing
            (BalanceSide::Dr, 20_000, BalanceSide::Dr, 23_000),
            (BalanceSide::Zero, 0, BalanceSide::Dr, 1_000),
        ]
    );
    assert!(preview.rows[0].errors.is_empty());
    assert_eq!(preview.rows[1].errors.len(), 1);
    assert_eq!(
        (
            preview.rows[1].errors[0].code,
            preview.rows[1].errors[0].field.as_str()
        ),
        (ErrorCode::ErrDecimalsNotAllowed, "lines[1].amount")
    );
    assert_eq!(
        preview.rows[0].account.as_ref().map(|a| a.code.as_str()),
        Some("030110001")
    );
    assert_eq!(
        preview.total_paisa,
        rupees(9_000),
        "only the lines without a mistake"
    );
    assert_eq!(
        count(&f.dir, "SELECT COUNT(*) FROM vouchers"),
        0,
        "the preview writes nothing"
    );

    let saved = vouchers::save_advances(&f.env.ctx, &f.owner, &list).unwrap();
    assert_eq!(
        saved
            .rows
            .iter()
            .map(|r| r.saved.as_ref().map(|s| s.number))
            .collect::<Vec<_>>(),
        [Some(1), None, Some(2), Some(3)]
    );
    assert_eq!(saved.rows[1].errors[0].field, "lines[1].amount");
    assert_eq!(saved.saved_paisa, rupees(9_000));
    // The saved balances are the previewed ones.
    for (row, preview_row) in saved.rows.iter().zip(&preview.rows) {
        if let Some(s) = &row.saved {
            assert_eq!(s.new_balance, preview_row.new_balance);
        }
    }
    assert_eq!(balance(&f.env, f.rahim), (BalanceSide::Dr, 23_000));
    assert_eq!(balance(&f.env, f.cash), (BalanceSide::Dr, 41_000));
    // The list is bounded.
    let too_many = AdvancesInput {
        lines: (0..501).map(|_| advance(f.rahim, "1")).collect(),
    };
    assert_eq!(
        first_error(vouchers::preview_advances(&f.env.ctx, &too_many).unwrap_err()).0,
        ErrorCode::ErrOutOfRange
    );
}

#[test]
fn a_seller_with_a_credit_balance_gets_the_right_side_and_amount() {
    // The money review: Karim has 10,160 جمع (PRD 4.5). 4,000 leaves 6,160 جمع; 10,160 leaves
    // nothing; 12,000 puts him 1,840 بنام.
    let f = fixture();
    set_opening(&f.env, &f.owner, f.karim, "", "10,160", false);
    let preview = vouchers::preview_advances(
        &f.env.ctx,
        &AdvancesInput {
            lines: vec![advance(f.karim, "4,000")],
        },
    )
    .unwrap();
    assert_eq!(
        (
            preview.rows[0].current_balance.side,
            preview.rows[0].current_balance.amount_paisa
        ),
        (BalanceSide::Cr, rupees(10_160))
    );
    assert_eq!(
        (
            preview.rows[0].new_balance.side,
            preview.rows[0].new_balance.amount_paisa
        ),
        (BalanceSide::Cr, rupees(6_160))
    );
    let saved = vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "4,000")).unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Cr, rupees(6_160))
    );
    // The advance stays open at 4,000 while his ledger is 6,160 جمع (payable 10,160 less 4,000).
    assert_eq!(
        assist::advances(&f.env.ctx, f.karim).unwrap().open_paisa,
        rupees(4_000)
    );
    let saved = vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "6,160")).unwrap();
    assert_eq!(saved.new_balance.side, BalanceSide::Zero);
    let saved = vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "1,840")).unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(1_840))
    );
}

#[test]
fn an_advance_bigger_than_the_cash_in_hand_is_allowed() {
    // No PRD rule refuses it (FR-15.7 warns only on a payment over the payable): cash 50,000,
    // advance 60,000 leaves cash 10,000 جمع and the books still balance.
    let f = fixture();
    vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "60,000")).unwrap();
    assert_eq!(balance(&f.env, f.cash), (BalanceSide::Cr, 10_000));
    let trial = ledger::trial_balance(&f.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn the_amount_limits() {
    let f = fixture();
    // 10,00,00,00,000 rupees (ten billion, 1,000 crore) is the largest one value (Code Standards
    // 4.2); one more is refused (see the mistakes test). Urdu digits and a simple sum work.
    let big =
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "10,000,000,000")).unwrap();
    assert_eq!(big.new_balance.amount_paisa, rupees(10_000_000_000));
    let second =
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "10,000,000,000")).unwrap();
    assert_eq!(
        second.new_balance.amount_paisa,
        rupees(20_000_000_000),
        "sums are checked, not capped"
    );
    let urdu = vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "۱۵۰۰۰")).unwrap();
    assert_eq!(urdu.new_balance.amount_paisa, rupees(15_000));
    let sum =
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "10000+5000")).unwrap();
    assert_eq!(sum.new_balance.amount_paisa, rupees(30_000));
}

#[test]
fn the_open_advances_follow_the_journal_and_reversals() {
    // PRD FR-7.60, FR-15.8: opening advance 15,000 and a voucher of 5,000; reversing the voucher
    // takes it out of the list and the card; a second reversal is refused.
    let f = fixture();
    set_opening(&f.env, &f.owner, f.rahim, "15,000", "", true);
    let voucher = vouchers::save_advance(
        &f.env.ctx,
        &f.owner,
        &AdvanceInput {
            narration: "کیلا کا موسم".into(),
            expected_goods: "کیلا".into(),
            ..advance(f.rahim, "5,000")
        },
    )
    .unwrap();
    let info = assist::advances(&f.env.ctx, f.rahim).unwrap();
    assert_eq!(info.open_paisa, rupees(20_000));
    assert_eq!(
        info.open
            .iter()
            .map(|o| (o.source.as_str(), o.voucher_number, o.open_paisa.rupees()))
            .collect::<Vec<_>>(),
        [("opening", None, 15_000), ("voucher", Some(1), 5_000)]
    );
    assert_eq!(
        assist::party_card(&f.env.ctx, f.rahim)
            .unwrap()
            .advance_open,
        Some(rupees(20_000)),
        "the card and the screen agree"
    );
    let last = info.last.unwrap();
    assert_eq!((last.voucher_number, last.amount_paisa), (1, rupees(5_000)));

    // Reverse the voucher's entry (the same date, the day is open).
    let reversal = f
        .env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&f.env.ctx, tx, f.owner.user_id, voucher.entry_id, "cancel"))
        .unwrap();
    let info = assist::advances(&f.env.ctx, f.rahim).unwrap();
    assert_eq!(info.open_paisa, rupees(15_000));
    assert_eq!(info.open.len(), 1);
    assert_eq!(
        assist::party_card(&f.env.ctx, f.rahim)
            .unwrap()
            .advance_open,
        Some(rupees(15_000))
    );
    assert_eq!(balance(&f.env, f.rahim), (BalanceSide::Dr, 15_000));
    assert_eq!(
        balance(&f.env, f.cash),
        (BalanceSide::Dr, 50_000),
        "cash is back"
    );
    // The reversal's lines carry no advance flag.
    assert_eq!(
        count(
            &f.dir,
            &format!(
                "SELECT COUNT(*) FROM journal_lines WHERE entry_id = {} AND is_advance = 1",
                reversal.entry_id
            )
        ),
        0
    );
    assert!(
        f.env
            .ctx
            .db()
            .write(|tx| ledger::reverse(&f.env.ctx, tx, f.owner.user_id, voucher.entry_id, "again"))
            .is_err()
    );
    // A reversed voucher keeps its number: the next is پ-2.
    assert_eq!(
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "100"))
            .unwrap()
            .number,
        2
    );
}

#[test]
fn reversing_a_line_switches_off_the_matching_that_rests_on_it() {
    // Plan 10, from plan 07: a reversal sets the related matching rows inactive, with who and
    // when. Matching again, oldest first, is the matching plan's job.
    let f = fixture();
    set_opening(&f.env, &f.owner, f.rahim, "15,000", "", true);
    let advance_entry: i64 = raw(&f.dir)
        .query_row(
            "SELECT opening_entry_id FROM accounts WHERE id = ?1",
            [f.rahim],
            |r| r.get(0),
        )
        .unwrap();
    // A credit on Rahim (10,000 back from him), then a matching row by hand: 10,000 adjusted.
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(f.cash), Side::Dr, rupees(10_000)),
        LedgerLine::new(AccountId::new(f.rahim), Side::Cr, rupees(10_000)),
    ])
    .unwrap();
    let credit = f
        .env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &f.env.ctx,
                tx,
                f.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10")
                        .unwrap(),
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
    let conn = raw(&f.dir);
    let line = |entry: i64| -> i64 {
        conn.query_row(
            "SELECT id FROM journal_lines WHERE entry_id = ?1 AND account_id = ?2",
            (entry, f.rahim),
            |r| r.get(0),
        )
        .unwrap()
    };
    conn.execute(
        "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 1000000)",
        (line(advance_entry), line(credit.entry_id), f.rahim),
    )
    .unwrap();
    assert_eq!(
        assist::advances(&f.env.ctx, f.rahim).unwrap().open_paisa,
        rupees(5_000)
    );
    let reversed = f
        .env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&f.env.ctx, tx, f.owner.user_id, credit.entry_id, "wrong"))
        .unwrap();
    assert_eq!(
        reversed.matching_off,
        (1, 0),
        "one advance row switched off, no bill row"
    );
    let (active, undone_at, undone_by): (i64, Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT is_active, undone_at, undone_by FROM advance_allocations",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(active, 0);
    assert!(undone_at.is_some());
    assert_eq!(undone_by, Some(f.owner.user_id));
    assert_eq!(
        assist::advances(&f.env.ctx, f.rahim).unwrap().open_paisa,
        rupees(15_000)
    );
    // The same for a bill row on both ends.
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM advance_allocations WHERE is_active = 1"
        ),
        0
    );
}

#[test]
fn the_advance_screen_reads_only_a_seller() {
    let f = fixture();
    let buyer = accounts::save(
        &f.env.ctx,
        &f.owner,
        &account_input(AccountType::Buyer, "اکرم ٹول ٹیکس", "Akram Toll Tax"),
    )
    .unwrap()
    .id;
    assert_eq!(
        first_error(assist::advances(&f.env.ctx, buyer).unwrap_err()).0,
        ErrorCode::ErrNotASeller
    );
    assert_eq!(
        first_error(assist::advances(&f.env.ctx, 999_999).unwrap_err()).0,
        ErrorCode::ErrNotFound
    );
    let empty = assist::advances(&f.env.ctx, f.rahim).unwrap();
    assert!(empty.open.is_empty() && empty.last.is_none());
    assert_eq!(empty.open_paisa, Paisa::ZERO);
}

#[test]
fn the_voucher_list_filters_by_kind_and_date() {
    let f = fixture();
    vouchers::save_advance(
        &f.env.ctx,
        &f.owner,
        &AdvanceInput {
            date: "01-10-2026".into(),
            ..advance(f.rahim, "1,000")
        },
    )
    .unwrap();
    vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.karim, "2,000")).unwrap();
    let ask = |kind: Option<VoucherType>, from: &str, to: &str| {
        vouchers::list(
            &f.env.ctx,
            &VouchersListInput {
                voucher_type: kind,
                from: from.into(),
                to: to.into(),
            },
        )
        .unwrap()
    };
    assert_eq!(ask(None, "", "").len(), 2);
    assert_eq!(ask(Some(VoucherType::Receipt), "", "").len(), 0);
    let mid = ask(Some(VoucherType::Advance), "05-10-2026", "");
    assert_eq!(
        mid.iter()
            .map(|v| (v.number, v.amount_paisa.rupees()))
            .collect::<Vec<_>>(),
        [(2, 2_000)]
    );
    assert_eq!(
        mid[0].account.as_ref().map(|a| a.code.as_str()),
        Some("030110002")
    );
    let early = ask(None, "", "01-10-2026");
    assert_eq!(early.iter().map(|v| v.number).collect::<Vec<_>>(), [1]);
    assert_eq!(
        first_error(
            vouchers::list(
                &f.env.ctx,
                &VouchersListInput {
                    voucher_type: None,
                    from: "99-99-2026".into(),
                    to: String::new()
                }
            )
            .unwrap_err()
        )
        .1
        .as_deref(),
        Some("from")
    );
}

#[test]
fn saving_a_voucher_is_fast() {
    // NFR-SPD-9: under 150 ms on the reference computer. A generous bound in a debug build.
    let f = fixture();
    let started = Instant::now();
    for _ in 0..20 {
        vouchers::save_advance(&f.env.ctx, &f.owner, &advance(f.rahim, "1,000")).unwrap();
    }
    let each = started.elapsed() / 20;
    eprintln!("a voucher saved in {each:?} (debug build)");
    assert!(each.as_millis() < 300, "{each:?}");
}

#[test]
fn random_advances_and_reversals_keep_the_books_exact() {
    // Property (money review): after every step, the books balance, cash is its opening less the
    // advances in force, each seller's open advance is what he was given and not reversed, and
    // the numbers run 1..n with no gap.
    let f = fixture();
    let sellers = [f.rahim, f.karim];
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    // (seller index, amount in rupees, entry id, still in force)
    let mut given: Vec<(usize, i64, i64, bool)> = Vec::new();
    let mut saved_numbers = 0_i64;
    for _ in 0..80 {
        let step = next() % 5;
        if step == 0 && !given.is_empty() {
            // Reverse one that is still in force.
            let candidates: Vec<usize> = given
                .iter()
                .enumerate()
                .filter(|(_, g)| g.3)
                .map(|(i, _)| i)
                .collect();
            if let Some(i) =
                candidates.get(usize::try_from(next()).unwrap() % candidates.len().max(1))
            {
                let entry = given[*i].2;
                f.env
                    .ctx
                    .db()
                    .write(|tx| ledger::reverse(&f.env.ctx, tx, f.owner.user_id, entry, "random"))
                    .unwrap();
                given[*i].3 = false;
            }
        } else {
            let who = usize::try_from(next() % 2).unwrap();
            let amount = i64::try_from(1 + next() % 90_000).unwrap();
            let saved = vouchers::save_advance(
                &f.env.ctx,
                &f.owner,
                &advance(sellers[who], &amount.to_string()),
            )
            .unwrap();
            saved_numbers += 1;
            assert_eq!(saved.number, saved_numbers);
            given.push((who, amount, saved.entry_id, true));
        }
        let trial = ledger::trial_balance(&f.env.ctx, None).unwrap();
        assert_eq!(trial.total_dr, trial.total_cr);
        let in_force: i64 = given.iter().filter(|g| g.3).map(|g| g.1).sum();
        let cash = ledger::account_balance(&f.env.ctx, f.cash).unwrap();
        let cash_now = match cash.side {
            BalanceSide::Dr => cash.amount_paisa.rupees(),
            BalanceSide::Cr => -cash.amount_paisa.rupees(),
            BalanceSide::Zero => 0,
        };
        assert_eq!(cash_now, 50_000 - in_force);
        for (index, seller) in sellers.iter().enumerate() {
            let want: i64 = given
                .iter()
                .filter(|g| g.3 && g.0 == index)
                .map(|g| g.1)
                .sum();
            assert_eq!(
                assist::advances(&f.env.ctx, *seller).unwrap().open_paisa,
                rupees(want)
            );
            assert_eq!(
                assist::party_card(&f.env.ctx, *seller)
                    .unwrap()
                    .advance_open,
                Some(rupees(want))
            );
        }
    }
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1 AND cr_paisa > 0"
        ),
        0
    );
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM journal_lines WHERE dr_paisa % 100 <> 0 OR cr_paisa % 100 <> 0"
        ),
        0
    );
    assert_eq!(count(&f.dir, "SELECT COUNT(*) FROM partner_share_rows"), 0);
    ledger::verify_journal_chain(&f.env.ctx).unwrap();
    assert_chain_ok(&f.dir);
}

#[test]
fn a_refused_line_shows_no_new_balance() {
    // A line that will not be saved moves nothing, for its own display too: its new balance is
    // its current balance (money review of plan 10).
    let f = fixture();
    raw(&f.dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-05' WHERE id = 1",
            [],
        )
        .unwrap();
    let list = AdvancesInput {
        lines: vec![
            advance(f.rahim, "1,000"),
            AdvanceInput {
                date: "03-10-2026".into(),
                ..advance(f.rahim, "2,000")
            },
            advance(f.rahim, "3,000"),
        ],
    };
    let preview = vouchers::preview_advances(&f.env.ctx, &list).unwrap();
    let shown: Vec<(i64, i64)> = preview
        .rows
        .iter()
        .map(|r| {
            (
                r.current_balance.amount_paisa.rupees(),
                r.new_balance.amount_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(shown, [(0, 1_000), (1_000, 1_000), (1_000, 4_000)]);
    assert_eq!(preview.rows[1].errors[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(preview.rows[1].errors[0].field, "lines[1].date");
    // The values for the message come with the error, as in any error (Code Standards 4.3).
    assert_eq!(
        preview.rows[1].errors[0].values.get("closed_up_to"),
        Some(&mandi_services::core::error::ErrorValue::Text(
            "05-10-2026".into()
        ))
    );
    assert_eq!(preview.total_paisa, rupees(4_000));
}

#[test]
fn a_failure_that_is_not_a_mistake_keeps_the_results_of_the_lines_already_saved() {
    // Money review of plan 10: the answer must not lose the vouchers that are already saved,
    // or "save all" would be repeated and the seller debited twice.
    let f = fixture();
    // The database refuses the second voucher of the list (as a full disk or a locked file
    // would): a trigger stands in for the failure.
    raw(&f.dir)
        .execute_batch(
            "CREATE TRIGGER fail_the_second BEFORE INSERT ON vouchers WHEN NEW.number = 2
             BEGIN SELECT RAISE(ABORT, 'simulated failure'); END;",
        )
        .unwrap();
    let list = AdvancesInput {
        lines: vec![
            advance(f.rahim, "1,000"),
            advance(f.rahim, "2,000"),
            advance(f.karim, "3,000"),
        ],
    };
    let saved = vouchers::save_advances(&f.env.ctx, &f.owner, &list).unwrap();
    assert_eq!(saved.rows.len(), 3);
    assert_eq!(saved.rows[0].saved.as_ref().map(|s| s.number), Some(1));
    assert!(saved.rows[1].saved.is_none());
    assert_eq!(saved.rows[1].errors.len(), 1);
    assert_eq!(saved.rows[1].errors[0].field, "lines[1]");
    assert!(saved.rows[2].saved.is_none(), "number 2 is still refused");
    assert_eq!(saved.saved_paisa, rupees(1_000));
    // Only the first voucher exists, once; nothing of the failed lines is left behind.
    assert_eq!(count(&f.dir, "SELECT COUNT(*) FROM vouchers"), 1);
    assert_eq!(balance(&f.env, f.rahim), (BalanceSide::Dr, 1_000));
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"
        ),
        0
    );
    ledger::verify_journal_chain(&f.env.ctx).unwrap();
}
