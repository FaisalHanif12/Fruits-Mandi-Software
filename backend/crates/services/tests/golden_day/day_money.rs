//! Plan 17 T2a on the PRD worked day: the one خرچہ screen, ادھار رقم and the روکڑ بک (PRD 16.1 to
//! 16.5, FR-6.7, FR-16.8, FR-16.9; E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-29, E2E-30, E2E-30b
//! and the preview numbers of E2E-39). Every expected number is from the PRD and is never changed
//! to suit the code.
use super::*;
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::error::ErrorValue;
use mandi_services::core::expenses::{
    ExpenseEntry, ExpensePreviewInput, PartnerTakesInput, PersonalExpenseInput,
    PersonalReturnInput, ShopExpenseInput, TakeSource,
};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentReportInput, LentReturnMode, MoneyLentInput,
    MoneyLentReturnInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::{ServiceError, lent, masters, partners};

/// Events 1 to 4 of the PRD (بکری 1, بکری 2, the وصولی of 10,000 and the ادائیگی of 4,316).
pub fn events_1_to_4(w: &support::World) {
    save_ok(w, &support::event1(w), false);
    save_ok(w, &event2(w), false);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt_input(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.rahim, "4,316", "")).unwrap();
}

/// Makes an expense head, confirming the spellings the engine is unsure of, and returns its id.
pub fn expense_head(w: &support::World, name_ur: &str, name_en: &str) -> i64 {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = ExpenseHeadInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            confirmed_rules_words: words.clone(),
        };
        match masters::save(&w.env.ctx, &w.owner, &MasterInput::ExpenseHead(input)) {
            Ok(_) => break,
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
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == name_en)
        .unwrap()
        .id
}

pub fn shop_input(head: i64, amount: &str, date: &str) -> ShopExpenseInput {
    ShopExpenseInput {
        date: date.into(),
        expense_head_id: Some(head),
        amount: amount.into(),
        paid_to: None,
        narration: None,
        confirm_cash_below_zero: false,
    }
}

pub fn personal_input(partner: i64, amount: &str, date: &str) -> PersonalExpenseInput {
    PersonalExpenseInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        reason: "گھر کا خرچ".into(),
        return_by: None,
        taken_by: None,
        narration: None,
        confirm_cash_below_zero: false,
    }
}

pub fn return_input(partner: i64, amount: &str, date: &str) -> PersonalReturnInput {
    PersonalReturnInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
    }
}

/// Event 5 on the خرچہ screen (PRD 4.8): electricity 3,000 as دکان خرچہ and الف's 2,000.
pub fn event5(w: &support::World, electricity: i64) {
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop_input(electricity, "3,000", "10-10-2026"),
    )
    .unwrap();
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &personal_input(w.partners[0], "2,000", "10-10-2026"),
    )
    .unwrap();
}

pub fn cash_now(w: &support::World) -> i64 {
    balance(w, support::id_of(&w.env, "01010001")).1
}

fn share_sum(w: &support::World, partner: i64, kind: &str) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
             WHERE partner_id = ?1 AND kind = ?2",
            (partner, kind),
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
        / 100
}

#[test]
fn e2e_8_9_10_12_event_5_on_the_khercha_screen_completes_the_day() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);

    // E2E-8: cash 50,000 - 5,000 + 12,800 + 10,000 - 4,316 - 3,000 - 2,000.
    assert_eq!(cash_now(&w), 58_484);

    // E2E-9: income 2,196 / 1,646 / 1,646; expense -1,200 / -900 / -900; profit 996 / 746 / 746.
    let income: Vec<i64> = w
        .partners
        .iter()
        .map(|p| share_sum(&w, *p, "income"))
        .collect();
    let expense: Vec<i64> = w
        .partners
        .iter()
        .map(|p| share_sum(&w, *p, "expense"))
        .collect();
    assert_eq!(income, vec![2_196, 1_646, 1_646]);
    assert_eq!(expense, vec![-1_200, -900, -900]);
    let profit: Vec<i64> = income.iter().zip(&expense).map(|(i, e)| i + e).collect();
    assert_eq!(profit, vec![996, 746, 746]);
    // Partner A's drawings account holds the 2,000 he took, never split.
    assert_eq!(
        balance(&w, support::id_of(&w.env, "06030001")),
        (BalanceSide::Dr, 2_000)
    );

    // E2E-10: the trial balance is 85,048 on both sides.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, rupees(85_048));
    assert_eq!(trial.total_cr, rupees(85_048));

    // E2E-12: everything is sold.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_lots
             WHERE qty_received - qty_sold - qty_written_off - qty_taken > 0"
        ),
        0
    );

    // Each type has its own number series (PRD 15.1): خ-1 and ذ-1.
    let list = vouchers::list(
        &w.env.ctx,
        &VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            voucher_type: Some(VoucherType::ShopExpense),
            from: String::new(),
            to: String::new(),
            account_id: None,
            person_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        (
            list.rows.len(),
            list.rows[0].number,
            list.totals.total_paisa
        ),
        (1, 1, Some(rupees(3_000)))
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn the_cash_book_after_the_whole_day_has_no_cash_count_and_pages_add_up() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let all = mandi_services::cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            group_by_month: false,
            sort: None,
            from: None,
            to: None,
            offset: 0,
            limit: 1_000,
        },
    )
    .unwrap();
    // 10-10-26 only: opening 50,000 (the opening entry and the advance are older days).
    let day = mandi_services::cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            group_by_month: false,
            sort: None,
            from: Some("10-10-2026".into()),
            to: Some("10-10-2026".into()),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(day.opening_paisa, rupees(50_000));
    assert_eq!(day.closing_paisa, rupees(58_484));
    // In: 12,800 + 10,000. Out: 5,000 + 4,316 + 3,000 + 2,000 (R34, E2E-36).
    assert_eq!(
        (day.totals.in_paisa, day.totals.out_paisa),
        (rupees(22_800), rupees(14_316))
    );
    assert_eq!(all.closing_paisa, rupees(58_484));
    assert_eq!(all.opening_paisa, Paisa::ZERO);

    // Pages of 3: the rows of every page add up to the totals of the whole range, and the running
    // cash of the last row of the last page is the closing cash.
    let mut in_sum = Paisa::ZERO;
    let mut out_sum = Paisa::ZERO;
    let mut last = Paisa::ZERO;
    let mut seen = 0_i64;
    let mut offset = 0;
    loop {
        let page = mandi_services::cashbook::get(
            &w.env.ctx,
            &CashbookInput {
                group_by_month: false,
                sort: None,
                from: None,
                to: None,
                offset,
                limit: 3,
            },
        )
        .unwrap();
        assert_eq!(page.total_count, all.total_count);
        assert_eq!(page.totals, all.totals);
        if page.rows.is_empty() {
            break;
        }
        for row in &page.rows {
            in_sum = in_sum.checked_add(row.in_paisa).unwrap();
            out_sum = out_sum.checked_add(row.out_paisa).unwrap();
            last = row.running_paisa.unwrap();
            seen += 1;
        }
        offset += 3;
    }
    assert_eq!(seen, all.total_count);
    assert_eq!(
        (in_sum, out_sum),
        (all.totals.in_paisa, all.totals.out_paisa)
    );
    assert_eq!(last, all.closing_paisa);
    // The pages together are the rows of the single big page, with the same running cash.
    let running: Vec<Paisa> = all.rows.iter().map(|r| r.running_paisa.unwrap()).collect();
    assert_eq!(running.last().copied(), Some(rupees(58_484)));
    // The shop expense shows the account of its head and its voucher number.
    let expense_row = all
        .rows
        .iter()
        .find(|r| r.posting_type == "shop_expense")
        .unwrap();
    assert_eq!(
        (
            expense_row.number,
            expense_row.out_paisa,
            expense_row.account.as_ref().map(|a| a.code.as_str())
        ),
        (Some(1), rupees(3_000), Some("05010001"))
    );
}

#[test]
fn e2e_20_a_shop_expense_of_3384_splits_with_the_leftover_to_the_largest_share() {
    let w = support::world();
    // The world's date is 10-10-2026 and the partners have 40 / 30 / 30.
    let head = expense_head(&w, "بجلی", "Electricity");
    let saved = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop_input(head, "3,384", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(shares_of(&w, saved.entry_id), vec![-1_354, -1_015, -1_015]);
}

#[test]
fn e2e_39_the_preview_shows_the_split_and_then_only_the_chosen_partner() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");

    // Before the save: 3,000 shows الف 1,200, ب 900, ج 900 (PRD C11).
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::ShopExpense(shop_input(electricity, "3,000", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let parts: Vec<i64> = preview
        .shares
        .iter()
        .map(|s| s.amount_paisa.rupees())
        .collect();
    assert_eq!(parts, vec![1_200, 900, 900]);
    assert!(!preview.others_not_affected && preview.partner_effect.is_none());
    // The cash note: 58,484 after the 3,000 and the 2,000 of the day, 63,484 - 3,000 now.
    assert_eq!(preview.cash.cash_after_paisa, rupees(63_484 - 3_000));

    // The saved share rows are exactly the previewed parts.
    let saved = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop_input(electricity, "3,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(shares_of(&w, saved.entry_id), vec![-1_200, -900, -900]);

    // Then 2,000 typed as ذاتی خرچہ for الف: his کل رقم 2,196 - 1,200 = 996 now, -1,004 after.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::PersonalExpense(personal_input(
                w.partners[0],
                "2,000",
                "10-10-2026",
            )),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let effect = preview.partner_effect.as_ref().unwrap();
    assert_eq!(effect.partner.id, w.partners[0]);
    assert_eq!(
        (
            effect.total_before_paisa.rupees(),
            effect.total_after_paisa.rupees(),
            effect.open_before_paisa.rupees(),
            effect.open_after_paisa.rupees()
        ),
        (996, -1_004, 0, 2_000)
    );
    assert!(preview.others_not_affected && preview.shares.is_empty());

    // Saving it gives what the preview showed, and the cash of E2E-8.
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &personal_input(w.partners[0], "2,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(cash_now(&w), 58_484);
    // B and C still have their profit: 746 each (not affected by the personal entry).
    assert_eq!(
        (
            share_sum(&w, w.partners[1], "income") + share_sum(&w, w.partners[1], "expense"),
            share_sum(&w, w.partners[2], "income") + share_sum(&w, w.partners[2], "expense")
        ),
        (746, 746)
    );
}

#[test]
fn e2e_29_a_partner_returns_2000_and_the_take_shows_fully_returned() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let a = w.partners[0];

    let takes = |w: &support::World| {
        partners::takes(
            &w.env.ctx,
            &PartnerTakesInput {
                show_finished: true,
                group_by_month: false,
                sort: None,
                partner_id: a,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let before = takes(&w);
    assert_eq!(before.rows.len(), 1);
    let row = &before.rows[0];
    assert_eq!(row.source, TakeSource::Voucher);
    assert_eq!(
        (
            row.amount_paisa.rupees(),
            row.returned_paisa.rupees(),
            row.open_paisa.rupees(),
            row.reason.as_str()
        ),
        (2_000, 0, 2_000, "گھر کا خرچ")
    );
    assert_eq!(before.totals.open_paisa, rupees(2_000));

    // A return above the open amount is refused, with the open amount (FR-16.2).
    set_today(&w.env, 2026, 10, 11);
    let too_much = vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &return_input(a, "2,001", "11-10-2026"),
    )
    .unwrap_err();
    assert_eq!(
        code_and_value(&too_much, "open_paisa"),
        (
            ErrorCode::ErrReturnExceedsOpen,
            Some(rupees(2_000).as_paisa())
        )
    );

    // 11-10-26: A returns 2,000. Net taken 0, cash 60,484, the take is fully returned.
    let saved = vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &return_input(a, "2,000", "11-10-2026"),
    )
    .unwrap();
    assert_eq!(saved.voucher_type, VoucherType::PersonalReturn);
    assert_eq!(cash_now(&w), 60_484);
    let after = takes(&w);
    assert_eq!(
        (
            after.rows[0].returned_paisa.rupees(),
            after.rows[0].open_paisa.rupees(),
            after.totals.open_paisa.rupees()
        ),
        (2_000, 0, 0)
    );
    // His کل رقم is back to 996 (خالص 996 - (2,000 - 2,000)): the next preview starts from it.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::PersonalExpense(personal_input(a, "1,000", "11-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        preview.partner_effect.unwrap().total_before_paisa.rupees(),
        996
    );
    // A return by a partner with nothing open is refused.
    let none_open =
        vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_input(a, "1", "11-10-2026"))
            .unwrap_err();
    assert_eq!(none_open.code(), ErrorCode::ErrReturnExceedsOpen);
}

/// Makes the person اسلم and returns his account id.
fn aslam(w: &support::World, staff: Option<i64>) -> i64 {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: Some("0300-1234567".into()),
            staff_id: staff,
            confirmed_rules_words: words.clone(),
        };
        match lent::person_create(&w.env.ctx, &w.owner, &input) {
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
}

fn lent_input(person: i64, amount: &str, date: &str, return_by: Option<&str>) -> MoneyLentInput {
    MoneyLentInput {
        date: date.into(),
        person_account_id: Some(person),
        amount: amount.into(),
        reason: "گھریلو ضرورت".into(),
        return_by: return_by.map(str::to_owned),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn lent_return(
    person: i64,
    amount: &str,
    date: &str,
    mode: LentReturnMode,
) -> MoneyLentReturnInput {
    MoneyLentReturnInput {
        date: date.into(),
        person_account_id: Some(person),
        amount: amount.into(),
        mode,
        salary_head_id: None,
        narration: None,
    }
}

fn open_of_person(w: &support::World, person: i64) -> i64 {
    lent::list(
        &w.env.ctx,
        &LentListInput {
            show_finished: true,
            group_by_month: false,
            sort: None,
            person_id: person,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
    .totals
    .open_paisa
    .rupees()
}

#[test]
fn e2e_30_money_lent_to_aslam_is_returned_in_cash_and_from_salary() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    let salary = expense_head(&w, "تنخواہ", "Salaries");
    event5(&w, electricity);
    assert_eq!(cash_now(&w), 58_484);
    let person = aslam(&w, None);

    // 10-10-26: 5,000 lent, return by 15-10-26. Cash 53,484, open 5,000.
    let lent_saved = vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &lent_input(person, "5,000", "10-10-2026", Some("15-10-2026")),
    )
    .unwrap();
    assert_eq!(lent_saved.voucher_type, VoucherType::MoneyLent);
    assert_eq!(cash_now(&w), 53_484);
    assert_eq!(open_of_person(&w, person), 5_000);

    // 11-10-26: 3,000 back in cash. Open 2,000, cash 56,484.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_return(person, "3,000", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
    assert_eq!(open_of_person(&w, person), 2_000);
    assert_eq!(cash_now(&w), 56_484);

    // A return above the open amount is refused with the open amount.
    set_today(&w.env, 2026, 10, 12);
    let too_much = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_return(person, "2,001", "12-10-2026", LentReturnMode::Cash),
    )
    .unwrap_err();
    assert_eq!(
        code_and_value(&too_much, "open_paisa"),
        (
            ErrorCode::ErrReturnExceedsOpen,
            Some(rupees(2_000).as_paisa())
        )
    );

    // 12-10-26: 2,000 from salary (the head called تنخواہ is the default). Open 0, cash the same,
    // the salary is a shop expense split -800 / -600 / -600.
    let saved = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_return(person, "2,000", "12-10-2026", LentReturnMode::Salary),
    )
    .unwrap();
    assert_eq!(open_of_person(&w, person), 0);
    assert_eq!(cash_now(&w), 56_484);
    assert_eq!(shares_of(&w, saved.entry_id), vec![-800, -600, -600]);
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[("05010002", 2_000, 0), ("01030001", 0, 2_000)])
    );
    // The head the deduction used is the salary head, saved with the voucher (mode salary).
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM vouchers WHERE id = {} AND mode = 'salary' AND expense_head_id = {salary}",
                saved.voucher_id
            )
        ),
        1
    );

    // R32: lent 5,000, returned 5,000 (cash 3,000, salary 2,000), open 0.
    let report = lent::report(
        &w.env.ctx,
        &LentReportInput {
            search: None,
            sort: None,
            as_of: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(report.rows.len(), 1);
    let row = &report.rows[0];
    assert_eq!(
        (
            row.lent_paisa.rupees(),
            row.returned_cash_paisa.rupees(),
            row.returned_salary_paisa.rupees(),
            row.open_paisa.rupees()
        ),
        (5_000, 3_000, 2_000, 0)
    );
    assert_eq!(report.totals.open_paisa, Paisa::ZERO);
    // The trial balance still balances.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn e2e_30b_an_overdue_loan_is_in_the_attention_data_and_the_report() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let person = aslam(&w, None);
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &lent_input(person, "5,000", "10-10-2026", Some("15-10-2026")),
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_return(person, "3,000", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap();

    // On the return-by day itself nothing is overdue yet.
    set_today(&w.env, 2026, 10, 15);
    assert!(
        lent::attention_overdue(&w.env.ctx)
            .unwrap()
            .items
            .is_empty()
    );

    // "Today" 16-10-26: اسلم's 2,000 is 1 day past 15-10-26.
    set_today(&w.env, 2026, 10, 16);
    let attention = lent::attention_overdue(&w.env.ctx).unwrap();
    assert_eq!(attention.total_count, 1);
    let item = &attention.items[0];
    assert_eq!(
        (
            item.who.id,
            item.open_paisa.rupees(),
            item.days_past,
            item.return_by.as_str(),
            item.number
        ),
        (person, 2_000, 1, "2026-10-15", 1)
    );
    let report = lent::report(
        &w.env.ctx,
        &LentReportInput {
            search: None,
            sort: None,
            as_of: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(report.rows[0].days_past_return_by, Some(1));
    assert_eq!(report.totals.overdue_paisa, rupees(2_000));
    assert!(!attention.truncated);
}

#[test]
fn a_partner_take_past_its_return_by_day_is_in_the_attention_data() {
    let w = worked_day();
    events_1_to_4(&w);
    let mut input = personal_input(w.partners[1], "1,500", "10-10-2026");
    input.return_by = Some("12-10-2026".into());
    input.taken_by = Some("his son".into());
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &input).unwrap();
    set_today(&w.env, 2026, 10, 14);
    let attention = lent::attention_overdue(&w.env.ctx).unwrap();
    assert_eq!(attention.items.len(), 1);
    let item = &attention.items[0];
    assert_eq!(
        (item.who.id, item.open_paisa.rupees(), item.days_past),
        (w.partners[1], 1_500, 2)
    );
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            show_finished: true,
            group_by_month: false,
            sort: None,
            partner_id: w.partners[1],
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(takes.rows[0].taken_by, "his son");
    assert_eq!(takes.rows[0].days_past_return_by, Some(2));
}

#[test]
fn d4_a30_the_return_by_day_cannot_be_before_the_entry_day() {
    let w = worked_day();
    set_today(&w.env, 2026, 10, 15);
    let person = aslam(&w, None);
    // Money lent on 15-10-26 with return by 10-10-26 is refused.
    let refused = vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &lent_input(person, "1,000", "15-10-2026", Some("10-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnByBeforeEntry);
    assert_eq!(first_field(&refused).as_deref(), Some("return_by"));
    assert_eq!(open_of_person(&w, person), 0);
    // The same day and a later day save.
    for return_by in ["15-10-2026", "20-10-2026"] {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &lent_input(person, "1,000", "15-10-2026", Some(return_by)),
        )
        .unwrap();
    }
    assert_eq!(open_of_person(&w, person), 2_000);
    // The same rule on a partner's personal expense.
    let mut input = personal_input(w.partners[0], "100", "15-10-2026");
    input.return_by = Some("14-10-2026".into());
    let refused = vouchers::save_personal_expense(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnByBeforeEntry);
}
