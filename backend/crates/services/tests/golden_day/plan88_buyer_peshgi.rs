//! Plan 88 on the PRD worked day: a پیشگی can be given to a buyer, and it is saved exactly as cash
//! lent to him (PRD 15.4, FR-15.11 with its v1.51 line, FR-15.4, FR-6.7, FR-10.10). The expected
//! numbers are the PRD's E2E-32 (his dues 5,880, cash 55,484, R20 and the tile 24,564, the 3,000 a
//! bill dated after his بکری 1 bill). Nothing is changed to suit the code. A seller's پیشگی is
//! unchanged (E2E-7 and E2E-31 stay in their own tests).
use super::dead_amount::events_1_to_5;
use super::*;
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{ReportFilters, ReportId, ReportRunInput, run};
use mandi_services::ServiceError;
use mandi_services::core::accounts::{
    AccountScope, AccountsRecentInput, AccountsTouchInput, ResolveCodeInput, ResolveStatus,
    SearchAccountsInput,
};
use mandi_services::core::vouchers::{AdvancesInput, VoucherEnteredAs, VoucherSaved};
use mandi_services::{dead, search};

const REASON: &str = "نقد ادھار";

fn advance_to(account: i64, amount: &str, reason: Option<&str>) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: account,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
        reason: reason.map(str::to_owned),
    }
}

fn first_error(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => {
            let first = items.first().expect("a rejection has an item");
            (first.code, first.field.clone())
        }
        other => panic!("not a rejection: {other:?}"),
    }
}

fn cash_of(w: &support::World) -> (BalanceSide, i64) {
    balance(w, support::id_of(&w.env, "01010001"))
}

fn tile_dues(w: &support::World) -> i64 {
    dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
    .buyers_dues_paisa
    .rupees()
}

/// The balance column and the total of R20 (rupees): the first row's balance, then the total.
fn r20(w: &support::World) -> (i64, i64) {
    use mandi_reports::reports::Cell;
    let page = run(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R20,
            filters: ReportFilters::default(),
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    let money = |cell: &Cell| match cell {
        Cell::Money { paisa } | Cell::Balance { paisa, .. } => paisa.rupees(),
        other => panic!("not money: {other:?}"),
    };
    (
        money(&page.rows.first().expect("a buyer owes").cells[3]),
        money(page.totals.get(3).expect("a total")),
    )
}

fn voucher_count(w: &support::World) -> i64 {
    scalar(w, "SELECT COUNT(*) FROM vouchers")
}

/// Each partner's کل رقم in rupees, in the order the partners were made.
fn partner_totals(w: &support::World) -> Vec<i64> {
    let overview = mandi_services::partners::overview(&w.env.ctx).unwrap();
    w.partners
        .iter()
        .map(|id| {
            overview
                .iter()
                .find(|row| row.partner_id == *id)
                .expect("a partner row")
                .total_paisa
                .rupees()
        })
        .collect()
}

fn trial_totals(w: &support::World) -> (i64, i64) {
    let t = ledger::trial_balance(&w.env.ctx, None).unwrap();
    (t.total_dr.rupees(), t.total_cr.rupees())
}

#[test]
fn e2e_32_a_peshgi_of_3000_to_akram_is_cash_lent_to_him() {
    let w = events_1_to_5();
    assert_eq!(cash_of(&w), (BalanceSide::Dr, 58_484));
    assert_eq!(trial_totals(&w), (85_048, 85_048));
    // What a buyer's advance must NOT touch (FR-17.13: only a seller's advance moves the partners):
    // the advance lines, the advance matching, the partner share rows and the partners' totals.
    let untouched = |w: &support::World| {
        (
            scalar(w, "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1"),
            scalar(w, "SELECT COUNT(*) FROM advance_allocations"),
            scalar(w, "SELECT COUNT(*) FROM partner_share_rows"),
            partner_totals(w),
        )
    };
    let before = untouched(&w);
    assert_eq!(before.3, vec![-1_004, 746, 746]);

    let saved = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &advance_to(w.akram, "3,000", Some(REASON)),
    )
    .unwrap();
    // The voucher is a payment, from the ادائیگی series: ا-1 was رحیم's 4,316, so this is ا-2.
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Payment, 2)
    );
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(5_880))
    );

    // His dues 5,880: two open bills, the 3,000 dated 10-10-26 after his بکری 1 bill (E2E-32).
    let bills = open_bills_of(&w, w.akram);
    assert_eq!(bills.bills.len(), 2);
    assert_eq!(
        (
            bills.bills[0].source,
            bills.bills[0].number,
            bills.bills[0].open_paisa.rupees()
        ),
        (LineSource::Bikri, Some(1), 2_880)
    );
    assert_eq!(
        (
            bills.bills[1].source,
            bills.bills[1].number,
            bills.bills[1].entry_date.as_str(),
            bills.bills[1].open_paisa.rupees()
        ),
        (LineSource::CashLent, Some(2), "2026-10-10", 3_000)
    );
    assert_eq!(bills.open_paisa.rupees(), 5_880);

    // Cash 55,484; the tile, R20 and the ledgers 24,564. The trial balance lists the balance of each
    // account: cash goes down 3,000 and اکرم up 3,000, so it stays 85,048 on both sides.
    assert_eq!(cash_of(&w), (BalanceSide::Dr, 55_484));
    assert_eq!(tile_dues(&w), 24_564);
    assert_eq!(r20(&w), (5_880, 24_564));
    assert_eq!(
        balance(&w, w.akram).1 + balance(&w, w.arshad).1,
        2_880 + 3_000 + 18_684
    );
    assert_eq!(trial_totals(&w), (85_048, 85_048));
    assert_eq!(untouched(&w), before);

    // The row is what a payment to a buyer is: its reason is kept, its account is a buyer's.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(
        (row.voucher_type, row.reason.as_str(), row.account_type),
        (VoucherType::Payment, REASON, Some(AccountType::Buyer))
    );
}

#[test]
fn a_peshgi_to_a_buyer_is_the_same_save_as_the_payment_to_him() {
    // The same entry saved the two ways on two copies of the day gives the same books: the same
    // balances, the same cash, the same journal total, the same bills.
    let by_advance = events_1_to_5();
    vouchers::save_advance(
        &by_advance.env.ctx,
        &by_advance.owner,
        &advance_to(by_advance.akram, "3,000", Some(REASON)),
    )
    .unwrap();
    let by_payment = events_1_to_5();
    vouchers::save_payment(
        &by_payment.env.ctx,
        &by_payment.owner,
        &payment_input(by_payment.akram, "3,000", REASON),
    )
    .unwrap();
    let figures = |w: &support::World| {
        let bills = open_bills_of(w, w.akram);
        (
            balance(w, w.akram),
            cash_of(w),
            trial_totals(w),
            bills
                .bills
                .iter()
                .map(|b| (b.source, b.number, b.open_paisa.rupees()))
                .collect::<Vec<_>>(),
            scalar(w, "SELECT COUNT(*) FROM vouchers WHERE type = 'payment'"),
            scalar(w, "SELECT COUNT(*) FROM vouchers WHERE type = 'advance'"),
        )
    };
    assert_eq!(figures(&by_advance), figures(&by_payment));
}

#[test]
fn a_peshgi_to_a_buyer_saves_without_a_reason_and_is_remembered_as_a_peshgi() {
    // PRD FR-15.11 v1.60: the reason is optional; the پیشگی posts as cash lent and is marked.
    let w = events_1_to_5();
    for reason in [None, Some(""), Some("   ")] {
        let preview = preview_of(
            &w,
            VoucherEntry::Advance(advance_to(w.akram, "3,000", reason)),
        );
        assert!(preview.errors.is_empty(), "reason {reason:?}");
    }
    let saved =
        vouchers::save_advance(&w.env.ctx, &w.owner, &advance_to(w.akram, "3,000", None)).unwrap();
    assert_eq!(saved.voucher_type, VoucherType::Payment);
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(row.entered_as, Some(VoucherEnteredAs::Advance));
    assert_eq!(row.reason, "");
    // E2E-32's numbers: cash 58,484 − 3,000, اکرم 2,880 + 3,000, and the line is no advance.
    assert_eq!(cash_of(&w), (BalanceSide::Dr, 55_484));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 5_880));
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_lines WHERE entry_id = {} AND is_advance = 1",
                saved.entry_id
            )
        ),
        0
    );
    // Cash lent from the ادائیگی tab is not marked.
    let lent =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.akram, "1,000", "")).unwrap();
    assert_eq!(
        vouchers::get(&w.env.ctx, lent.voucher_id)
            .unwrap()
            .entered_as,
        None
    );
}

#[test]
fn the_preview_of_a_peshgi_to_a_buyer_is_the_preview_of_the_payment_to_him() {
    let w = events_1_to_5();
    let advance = preview_of(
        &w,
        VoucherEntry::Advance(advance_to(w.akram, "3,000", Some(REASON))),
    );
    let payment = preview_of(
        &w,
        VoucherEntry::Payment(payment_input(w.akram, "3,000", REASON)),
    );
    assert!(advance.errors.is_empty());
    assert_eq!(
        serde_json::to_value(&advance).unwrap(),
        serde_json::to_value(&payment).unwrap()
    );
    assert_eq!(
        (
            advance.current_balance.amount_paisa,
            advance.new_balance.amount_paisa
        ),
        (rupees(2_880), rupees(5_880))
    );
    // Previewing saves nothing.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
}

#[test]
fn cash_below_zero_asks_for_a_peshgi_to_a_buyer_as_for_every_payment() {
    // FR-6.7: cash is 58,484; 60,000 lent takes it to −1,516.
    let w = events_1_to_5();
    let before = voucher_count(&w);
    let refused = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &advance_to(w.akram, "60,000", Some(REASON)),
    )
    .unwrap_err();
    assert_eq!(
        first_error(&refused).0,
        ErrorCode::ErrCashBelowZeroNotConfirmed
    );
    assert_eq!(voucher_count(&w), before);
    let preview = preview_of(
        &w,
        VoucherEntry::Advance(advance_to(w.akram, "60,000", Some(REASON))),
    );
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa, rupees(-1_516));

    let mut confirmed = advance_to(w.akram, "60,000", Some(REASON));
    confirmed.confirm_cash_below_zero = true;
    let saved = vouchers::save_advance(&w.env.ctx, &w.owner, &confirmed).unwrap();
    assert_eq!(saved.voucher_type, VoucherType::Payment);
    assert_eq!(cash_of(&w), (BalanceSide::Cr, 1_516));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 62_880));
}

#[test]
fn a_buyer_in_the_dead_amount_is_refused_at_the_account_field() {
    // FR-10.10: any new entry on him but a وصولی is refused as on any inactive account.
    let w = events_1_to_5();
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &serde_json::from_value(serde_json::json!({
            "account_id": w.arshad,
            "entry_date": "11-10-2026",
            "reason": "died",
            "note": "فوت ہو گئے"
        }))
        .unwrap(),
    )
    .unwrap();
    let before = voucher_count(&w);
    let mut input = advance_to(w.arshad, "1,000", Some(REASON));
    input.date = "11-10-2026".into();
    let refused = vouchers::save_advance(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(
        first_error(&refused),
        (
            ErrorCode::ErrAccountInactive,
            Some("seller_account_id".to_owned())
        )
    );
    assert_eq!(voucher_count(&w), before);
    let preview = preview_of(&w, VoucherEntry::Advance(input));
    assert_eq!(
        (preview.errors[0].code, preview.errors[0].field.as_str()),
        (ErrorCode::ErrAccountInactive, "seller_account_id")
    );
}

#[test]
fn a_seller_peshgi_is_unchanged_and_ignores_a_reason() {
    // E2E-7 and E2E-31 are in their own tests. Here: a seller's advance is still an advance, from
    // the پ series (پ-1 was رحیم's 15,000), and a reason typed with it changes nothing.
    let w = events_1_to_5();
    let karim = support::id_of(&w.env, "0011");
    let saved = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &advance_to(karim, "5,000", Some("ignored")),
    )
    .unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Advance, 2)
    );
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(
        (row.reason.as_str(), row.account_type),
        ("", Some(AccountType::Seller))
    );
    // The advance is on his ledger and counted as an advance; no bill of a buyer.
    assert_eq!(cash_of(&w), (BalanceSide::Dr, 53_484));
    assert_eq!(tile_dues(&w), 21_564);
}

#[test]
fn the_accounts_without_a_ledger_keep_the_refusal_of_an_advance_to_a_non_seller() {
    let w = events_1_to_5();
    let naqdi = support::id_of(&w.env, "03040000");
    for account in [w.chukta_khata, naqdi] {
        let refused = vouchers::save_advance(
            &w.env.ctx,
            &w.owner,
            &advance_to(account, "1,000", Some(REASON)),
        )
        .unwrap_err();
        assert_eq!(
            first_error(&refused),
            (
                ErrorCode::ErrNotASeller,
                Some("seller_account_id".to_owned())
            ),
            "account {account}"
        );
    }
}

#[test]
fn a_saved_buyer_peshgi_opens_for_edit_as_a_payment_and_an_advance_entry_cannot_replace_it() {
    let w = events_1_to_5();
    let saved = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &advance_to(w.akram, "3,000", Some(REASON)),
    )
    .unwrap();
    let opened = vouchers::open_for_edit(
        &w.env.ctx,
        &mandi_services::core::vouchers::VoucherGetInput {
            id: saved.voucher_id,
        },
    )
    .unwrap();
    assert!(matches!(opened.entry, VoucherEntry::Payment(_)));
    // The preview of an edit keeps the type: an advance entry on a buyer account cannot take the
    // place of a saved voucher.
    let replaced = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Advance(advance_to(w.akram, "2,000", Some(REASON))),
            replacing_voucher_id: Some(saved.voucher_id),
        },
    )
    .unwrap();
    assert!(!replaced.errors.is_empty());
}

#[test]
fn the_running_list_takes_sellers_and_buyers_each_line_its_own_voucher() {
    let w = events_1_to_5();
    let karim = support::id_of(&w.env, "0011");
    let input = AdvancesInput {
        lines: vec![
            advance_to(w.akram, "3,000", None),
            advance_to(karim, "2,000", None),
            advance_to(w.arshad, "1,000", Some(REASON)),
        ],
    };
    // The preview: every line is fine, a buyer line with or without a reason (PRD FR-15.11 v1.60).
    // Balances before and after are his.
    let preview = vouchers::preview_advances(&w.env.ctx, &input).unwrap();
    assert!(preview.rows.iter().all(|r| r.errors.is_empty()));
    assert_eq!(
        (
            preview.rows[2].current_balance.amount_paisa,
            preview.rows[2].new_balance.amount_paisa
        ),
        (rupees(18_684), rupees(19_684))
    );
    assert_eq!(preview.total_paisa, rupees(6_000));

    let saved = vouchers::save_advances(&w.env.ctx, &w.owner, &input).unwrap();
    let kinds: Vec<VoucherType> = saved
        .rows
        .iter()
        .map(|r| {
            r.saved
                .as_ref()
                .map(|s: &VoucherSaved| s.voucher_type)
                .unwrap()
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            VoucherType::Payment,
            VoucherType::Advance,
            VoucherType::Payment
        ]
    );
    assert_eq!(saved.saved_paisa, rupees(6_000));
}

#[test]
fn the_peshgi_picker_lists_sellers_and_buyers_each_with_its_type() {
    let w = events_1_to_5();
    let sellers_and_buyers = |text: &str| {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                include_dead: Some(true),
                scope: AccountScope::SellerBuyer,
                control_id: None,
                text: text.into(),
                limit: 50,
                offset: 0,
                with_chukta: Some(true),
            },
        )
        .unwrap()
        .into_iter()
        .map(|h| (h.code, h.account_type))
        .collect::<Vec<_>>()
    };
    // Every seller and buyer, by code; nothing else: not چکتا کھاتہ, not the چکتا buyer شاہد, not
    // نقدی منڈی, not income or cash, not اپنا مال.
    assert_eq!(
        sellers_and_buyers(""),
        vec![
            ("0010".to_owned(), AccountType::Seller),
            ("0011".to_owned(), AccountType::Seller),
            ("0020".to_owned(), AccountType::Buyer),
            ("0021".to_owned(), AccountType::Buyer),
        ]
    );
    // A name finds the buyer, a code the seller.
    assert_eq!(
        sellers_and_buyers("akram"),
        vec![("0020".to_owned(), AccountType::Buyer)]
    );
    assert_eq!(
        sellers_and_buyers("0010"),
        vec![("0010".to_owned(), AccountType::Seller)]
    );
    // A short code: `0` is the seller 0010 and the buyer 0020, each with its type.
    let resolved = search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::SellerBuyer,
            control_id: None,
            text: "0".into(),
        },
    )
    .unwrap();
    assert_eq!(resolved.status, ResolveStatus::Several);
    let mut found: Vec<(String, AccountType)> = resolved
        .candidates
        .into_iter()
        .map(|h| (h.code, h.account_type))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        found,
        vec![
            ("0010".to_owned(), AccountType::Seller),
            ("0020".to_owned(), AccountType::Buyer)
        ]
    );
    // A recent list and a touch work for the field, with its own preference.
    search::touch_account(
        &w.env.ctx,
        &w.owner,
        &AccountsTouchInput {
            scope: AccountScope::SellerBuyer,
            control_id: None,
            id: w.arshad,
        },
    )
    .unwrap();
    let recent = search::recent_accounts(
        &w.env.ctx,
        &w.owner,
        &AccountsRecentInput {
            include_dead: None,
            scope: AccountScope::SellerBuyer,
            control_id: None,
            with_chukta: None,
        },
    )
    .unwrap();
    assert_eq!(recent.first().map(|h| h.id), Some(w.arshad));
    assert_eq!(recent.len(), 4);
}

#[test]
fn the_peshgi_picker_leaves_out_a_person_in_the_dead_amount() {
    let w = events_1_to_5();
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &serde_json::from_value(serde_json::json!({
            "account_id": w.arshad,
            "entry_date": "11-10-2026",
            "reason": "died",
            "note": "فوت ہو گئے"
        }))
        .unwrap(),
    )
    .unwrap();
    // Even when the caller asks for the dead (the receipt field does), the advance field never
    // offers him: nothing is given to a person in the dead amount.
    let codes: Vec<String> = search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &SearchAccountsInput {
            include_dead: Some(true),
            scope: AccountScope::SellerBuyer,
            control_id: None,
            text: String::new(),
            limit: 50,
            offset: 0,
            with_chukta: None,
        },
    )
    .unwrap()
    .into_iter()
    .map(|h| h.code)
    .collect();
    assert_eq!(codes, vec!["0010", "0011", "0020"]);
}

#[test]
fn the_peshgi_list_shows_the_buyer_ones_with_their_type() {
    let w = events_1_to_5();
    let list = |voucher_type: VoucherType, advance_view: Option<bool>| {
        vouchers::list(
            &w.env.ctx,
            &VouchersListInput {
                voucher_type: Some(voucher_type),
                from: String::new(),
                to: String::new(),
                account_id: None,
                offset: 0,
                limit: 0,
                show_finished: false,
                search: None,
                sort: None,
                group_by_month: false,
                advance_view,
            },
        )
        .unwrap()
    };
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &advance_to(w.akram, "3,000", Some(REASON)),
    )
    .unwrap();
    // The plain list of advances is as before: only رحیم's پیشگی of 01-09-26.
    let plain = list(VoucherType::Advance, None);
    assert_eq!(plain.total_count, 1);
    assert_eq!(plain.rows[0].account_type, Some(AccountType::Seller));
    // The پیشگی list: that, and the 3,000 lent to اکرم, but not the 4,316 paid to رحیم.
    let view = list(VoucherType::Advance, Some(true));
    assert_eq!(view.total_count, 2);
    let rows: Vec<(VoucherType, i64, Option<AccountType>)> = view
        .rows
        .iter()
        .map(|r| (r.voucher_type, r.amount_paisa.rupees(), r.account_type))
        .collect();
    assert_eq!(
        rows,
        vec![
            (VoucherType::Advance, 15_000, Some(AccountType::Seller)),
            (VoucherType::Payment, 3_000, Some(AccountType::Buyer)),
        ]
    );
    assert_eq!(view.totals.total_paisa, Some(rupees(18_000)));
    // The flag means nothing for another type: the two payments (to رحیم and to اکرم), as without it.
    let payments = list(VoucherType::Payment, Some(true));
    assert_eq!(payments.total_count, 2);
    assert_eq!(
        payments.total_count,
        list(VoucherType::Payment, None).total_count
    );
}
