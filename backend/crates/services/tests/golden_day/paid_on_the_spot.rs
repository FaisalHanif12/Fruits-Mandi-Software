//! Plan 65 T2 (D8 Feature 4, PRD FR-12.44, FR-12.9, FR-15.4), E2E-65: money paid on the spot, on the real
//! services. The lot is the one of E2E-64 (200 آم from احسان, کرایہ 10,000 paid by the firm, مزدوری 1,000,
//! no نقد) sold to مانی, انس and ہارون, and ہارون holds 10,000 جمع before the sale (his card in image 1 of
//! D8). The expected numbers are the PRD's (D8 Feature 4, plan 65 Section 2) and the money agent's hand
//! check (plan 65a point 9). They are never changed to suit the code.
//!
//! | Buyer | Buyer total | Paid on the spot | Left on account |
//! |---|---|---|---|
//! | مانی | 21,360 | 21,360 | 0 |
//! | انس | 48,100 | 10,000 | 38,100 |
//! | ہارون | 125,060 | 0 | 125,060 |
//! | Total | 194,520 | 31,360 | 163,160 |
//!
//! Journal: the sale part is that of E2E-64 (194,520 on each side). The cash part is روکڑ بنام 21,360 and
//! مانی جمع 21,360, then روکڑ بنام 10,000 and انس جمع 10,000: 31,360 on each side, so the whole entry is
//! 194,520 + 31,360 = 225,880 on each side. Cash goes from 50,000 to 81,360.
//!
//! ہارون: his earlier 10,000 جمع settles the new bill (125,060 - 10,000 = 115,060 بنام, open 115,060), so
//! what the three of them owe is 38,100 + 115,060 = 153,160, not 163,160.
use super::*;
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_services::core::bikri::{BikriDraft, SaleInput};

struct Case {
    w: support::World,
    seller: i64,
    mani: i64,
    anas: i64,
    haroon: i64,
    mango: i64,
}

/// The worked day's opening position, the lot's parties, and ہارون's opening credit of 10,000.
fn case() -> Case {
    let w = worked_day();
    let seller = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "احسان",
        "Ehsan",
        true,
        "",
    );
    let buyer = |ur: &str, en: &str| {
        support::add_account(&w.env, &w.owner, AccountType::Buyer, ur, en, true, "")
    };
    let mani = buyer("مانی", "Mani");
    let anas = buyer("انس", "Anas");
    let haroon = buyer("ہارون", "Haroon");
    let mango = support::add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    // His 10,000 جمع: the shop owes him (an opening credit, posted against ابتدائی فرق).
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: haroon,
                version: accounts::get(&w.env.ctx, haroon).unwrap().version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "10,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    Case {
        w,
        seller,
        mani,
        anas,
        haroon,
        mango,
    }
}

impl Case {
    /// The lot of 200 with کرایہ 10,000 (paid by the firm) and مزدوری 1,000, no نقد.
    fn draft(&self, paid: [&str; 3]) -> BikriDraft {
        let mut lot = support::bare_lot(&self.w, "lot-1", self.seller, self.mango, "200");
        lot.freight = Some("10,000".into());
        lot.labour = support::manual("1,000");
        let mut rows: Vec<SaleInput> = vec![
            support::sale("line-1", self.mani, "20", "1000"),
            support::sale("line-2", self.anas, "50", "900"),
            support::sale("line-3", self.haroon, "130", "900"),
        ];
        for (row, paid) in rows.iter_mut().zip(paid) {
            row.paid = (!paid.is_empty()).then(|| paid.to_owned());
        }
        lot.sales = rows;
        support::draft(vec![lot])
    }

    fn code(&self, id: i64) -> String {
        raw(&self.w.dir)
            .query_row("SELECT code FROM accounts WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap()
    }
}

fn cash(w: &support::World) -> (BalanceSide, i64) {
    balance(w, support::id_of(&w.env, "01010001"))
}

#[test]
fn e2e_65_paid_21360_10000_and_0_journal_cash_matching_and_dues() {
    let c = case();
    let w = &c.w;
    assert_eq!(balance(w, c.haroon), (BalanceSide::Cr, 10_000));
    assert_eq!(cash(w), (BalanceSide::Dr, 50_000));
    let draft = c.draft(["21,360", "10,000", ""]);

    // The preview: line by line, the totals of the lot, and what it does to cash.
    let preview = bikri::preview(&w.env.ctx, &draft).unwrap();
    assert!(preview.errors.is_empty(), "errors: {:?}", preview.errors);
    let shown: Vec<(i64, i64, i64, bool)> = (0..3)
        .map(|j| {
            let l = support::calc(&preview.lots[0], j);
            (
                support::r(l.buyer_total),
                support::r(l.paid_paisa),
                support::r(l.left_paisa),
                l.paid_fixed,
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (21_360, 21_360, 0, false),
            (48_100, 10_000, 38_100, false),
            (125_060, 0, 125_060, false)
        ]
    );
    let s = &preview.summary;
    assert_eq!(
        (
            support::r(s.buyer_total),
            support::r(s.buyer_cash),
            support::r(s.buyer_credit)
        ),
        (194_520, 31_360, 163_160)
    );
    // E2E-64 figures do not move: gross, deductions, net and the shared income.
    assert_eq!(
        (
            support::r(s.gross),
            support::r(s.deductions),
            support::r(s.net),
            support::r(s.shared_income)
        ),
        (182_000, 21_920, 160_080, 23_440)
    );
    assert_eq!(support::r(preview.cash.change_paisa), 31_360);
    assert_eq!(support::r(preview.cash.cash_after_paisa), 81_360);

    // The save is the same calculation.
    let saved = save_ok(w, &draft, false);
    assert_eq!(saved.preview, preview);

    // The stored lines: paid and left, and the old word of the column.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let stored: Vec<(i64, i64, i64)> = detail.lots[0]
        .sales
        .iter()
        .map(|s| {
            (
                s.buyer_total.rupees(),
                s.paid_paisa.rupees(),
                s.left_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        stored,
        [
            (21_360, 21_360, 0),
            (48_100, 10_000, 38_100),
            (125_060, 0, 125_060)
        ]
    );
    let words: Vec<String> = raw(&w.dir)
        .prepare("SELECT payment_type FROM bikri_sales ORDER BY line_no")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        words,
        ["cash", "credit", "credit"],
        "nothing decides money from it"
    );
    let t = detail.lots[0].totals;
    assert_eq!(
        (
            t.buyer_total.rupees(),
            t.buyer_cash.rupees(),
            t.buyer_credit.rupees()
        ),
        (194_520, 31_360, 163_160)
    );

    // The journal: the sale part of E2E-64, then the cash part.
    let journal = entry_lines(w, saved.entry_id);
    assert_eq!(
        journal,
        vec![
            (c.code(c.mani), 21_360, 0, None),
            (c.code(c.anas), 48_100, 0, None),
            (c.code(c.haroon), 125_060, 0, None),
            (c.code(c.seller), 0, 160_080, None),
            ("04010001".to_owned(), 0, 10_920, None),
            ("04010002".to_owned(), 0, 10_920, None),
            ("04010003".to_owned(), 0, 1_600, None),
            ("02010001".to_owned(), 0, 1_000, None),
            ("02010002".to_owned(), 0, 10_000, None),
            ("01010001".to_owned(), 21_360, 0, None),
            (c.code(c.mani), 0, 21_360, None),
            ("01010001".to_owned(), 10_000, 0, None),
            (c.code(c.anas), 0, 10_000, None),
        ]
    );
    assert_eq!(journal.iter().map(|l| l.1).sum::<i64>(), 225_880);
    assert_eq!(journal.iter().map(|l| l.2).sum::<i64>(), 225_880);
    // Cash is up by 31,360.
    assert_eq!(cash(w), (BalanceSide::Dr, 81_360));
    // The partner split is that of E2E-64.
    assert_eq!(shares_of(w, saved.entry_id), [9_376, 7_032, 7_032]);

    // Balances: مانی is square, انس owes 38,100, ہارون owes 125,060 - 10,000 = 115,060.
    assert_eq!(balance(w, c.mani).1, 0);
    assert_eq!(balance(w, c.anas), (BalanceSide::Dr, 38_100));
    assert_eq!(balance(w, c.haroon), (BalanceSide::Dr, 115_060));
    assert_eq!(balance(w, c.seller), (BalanceSide::Cr, 160_080));

    // Matching: the paid part of each bill is pinned to its own credit; what is left is a normal open
    // bill, and ہارون's old credit settles his new bill first.
    let open = |id: i64| open_bills_of(w, id);
    let mani = open(c.mani);
    assert_eq!((mani.bills.len(), mani.open_paisa.rupees()), (0, 0));
    let anas = open(c.anas);
    assert_eq!(anas.bills.len(), 1);
    assert_eq!(anas.open_paisa.rupees(), 38_100);
    assert_eq!(anas.bills[0].open_paisa.rupees(), 38_100);
    let haroon = open(c.haroon);
    assert_eq!(haroon.bills.len(), 1);
    assert_eq!(haroon.open_paisa.rupees(), 115_060);
    assert_eq!(haroon.unmatched_credit_paisa.rupees(), 0);
    let allocated = |id: i64| {
        scalar(
            w,
            &format!(
                "SELECT COALESCE(SUM(amount_paisa), 0) / 100 FROM bill_allocations WHERE account_id = {id} AND is_active = 1"
            ),
        )
    };
    assert_eq!(allocated(c.mani), 21_360);
    assert_eq!(allocated(c.anas), 10_000);
    assert_eq!(allocated(c.haroon), 10_000);

    // Dues: what the three owe is 38,100 + 115,060 = 153,160, not 163,160 (his credit is not owed).
    let tiles = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles;
    assert_eq!(tiles.buyers_dues_paisa.rupees(), 153_160);

    // The books balance. The trial balance adds the net balance of each account, so the debit side is
    // cash 81,360 + رحیم 15,000 + انس 38,100 + ہارون 115,060 + ابتدائی فرق 10,000 = 259,520, and the credit
    // side is capital 65,000 + احسان 160,080 + commission 10,920 + 10,920 + لاگا 1,600 + مزدوری 1,000 +
    // کرایہ 10,000 = 259,520.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    assert_eq!(trial.total_dr, rupees(259_520));
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn e2e_65_nothing_paid_and_everything_paid_post_what_credit_and_cash_lines_posted() {
    // Paid 0 on every line is the old all-credit بکری (no cash part), and paid in full on every line is
    // the old all-cash بکری (a cash part of 194,520): plan 60 P6, so no saved number changes.
    let credit = case();
    let saved = save_ok(&credit.w, &credit.draft(["", "", ""]), false);
    let lines = entry_lines(&credit.w, saved.entry_id);
    assert_eq!(lines.len(), 9, "no cash part");
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 194_520);
    assert_eq!(cash(&credit.w), (BalanceSide::Dr, 50_000));
    assert_eq!(
        balance(&credit.w, credit.haroon),
        (BalanceSide::Dr, 115_060)
    );

    let all = case();
    let saved = save_ok(&all.w, &all.draft(["21,360", "48,100", "125,060"]), false);
    let lines = entry_lines(&all.w, saved.entry_id);
    assert_eq!(lines.len(), 15, "nine lines and three cash pairs");
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 194_520 + 194_520);
    assert_eq!(cash(&all.w), (BalanceSide::Dr, 50_000 + 194_520));
    // ہارون paid all 125,060 and keeps his 10,000 جمع.
    assert_eq!(balance(&all.w, all.haroon), (BalanceSide::Cr, 10_000));
    assert_eq!(balance(&all.w, all.mani).1, 0);
}

#[test]
fn e2e_65_one_rupee_over_the_buyer_total_is_refused_and_nothing_is_saved() {
    // PRD FR-12.44: 21,361 on a bill of 21,360 is ERR_PAID_EXCEEDS_TOTAL on the paid field, with the
    // most allowed. The save writes nothing.
    let c = case();
    let draft = c.draft(["21,361", "", ""]);
    let preview = bikri::preview(&c.w.env.ctx, &draft).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrPaidExceedsTotal,
        "lots[0].sales[0].paid"
    ));
    let refused = bikri::save(
        &c.w.env.ctx,
        &c.w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    let mandi_services::ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    let item = items
        .iter()
        .find(|i| i.code == ErrorCode::ErrPaidExceedsTotal)
        .expect("refused with ERR_PAID_EXCEEDS_TOTAL");
    assert_eq!(item.field.as_deref(), Some("lots[0].sales[0].paid"));
    assert_eq!(
        item.values.get("max"),
        Some(&mandi_services::core::error::ErrorValue::Number(21_360))
    );
    assert_eq!(scalar(&c.w, "SELECT COUNT(*) FROM bikri_sales"), 0);
    assert_eq!(cash(&c.w), (BalanceSide::Dr, 50_000));
}

/// `{ "draft": ... }`, what `bikri_preview` takes inside `input`.
fn preview_fixture(
    name: &str,
    draft: &BikriDraft,
    answer: &mandi_services::core::bikri::BikriPreview,
) {
    super::day_close::write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(&serde_json::json!({ "draft": draft })).unwrap() + "\n"),
    );
    super::day_close::write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

#[test]
fn e2e_65_preview_fixtures_for_the_screen() {
    // The real answers of `bikri_preview` for the three cases the screen shows (plan 65): the D8
    // example with a part payment, a چکتا buyer whose paid cell is fixed, and a paid amount over the
    // buyer total with its error. The frontend copies the files.
    let c = case();
    let w = &c.w;
    let part = c.draft(["21,360", "10,000", ""]);
    let p = bikri::preview(&w.env.ctx, &part).unwrap();
    preview_fixture("bikri_preview_paid_d8", &part, &p);

    let over = c.draft(["21,361", "", ""]);
    let p = bikri::preview(&w.env.ctx, &over).unwrap();
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrPaidExceedsTotal,
        "lots[0].sales[0].paid"
    ));
    // The line keeps its figures, counts as paid 0 and is not fixed.
    let l = support::calc(&p.lots[0], 0);
    assert_eq!(
        (
            support::r(l.buyer_total),
            support::r(l.paid_paisa),
            support::r(l.left_paisa),
            l.paid_fixed
        ),
        (21_360, 0, 21_360, false)
    );
    preview_fixture("bikri_preview_paid_over_total", &over, &p);

    // Event 2: شاہد is a چکتا buyer; whatever is typed, the backend sets paid = 12,800.
    let event2 = event2(w);
    let p = bikri::preview(&w.env.ctx, &event2).unwrap();
    let l = support::calc(&p.lots[0], 0);
    assert_eq!(
        (
            support::r(l.paid_paisa),
            support::r(l.left_paisa),
            l.paid_fixed
        ),
        (12_800, 0, true)
    );
    preview_fixture("bikri_preview_paid_fixed_chukta", &event2, &p);
}
