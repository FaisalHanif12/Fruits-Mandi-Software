//! Plan 16b, market trips (PRD 12.10, v1.7): the goods of a trip sold in بکری by an own lot
//! (seller اپنا مال). The expected numbers are the PRD's (E2E-34, E2E-34b, E2E-41, C21, C23 to C25)
//! and the money agent's extra cases. They are never changed to suit the code.
//!
//! The trip goods belong to the trip's partners, not to the firm. The seller credit of an own lot
//! (C8) goes to the partners' 0604 accounts by the trip's shares (C25). The cost of the goods (C21)
//! is stored on each line and shown with the result. It is never posted.

use mandi_services::core::bikri::{
    BikriEdited, CancelInput, EditOpenInput, EditPreviewInput, EditSaveInput, EditableBikri,
    LotInput, StockListInput,
};
use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseGetInput, PurchaseInput, PurchaseRow, PurchaseWriteOffDone,
    PurchaseWriteOffInput, PurchaseWriteOffUndoInput,
};
use mandi_services::core::trips::{TripCostInput, TripGetInput, TripInput};
use mandi_services::core::ventures::{
    EntryPartInput, MoneySource, VenturePartnerInput, VentureStatus,
};
use mandi_services::{partners, purchases, trips};

use super::*;

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

/// A trip on 10-10-26 of the three partners at 40 / 30 / 30, all paying from the shop's cash.
fn trip_of_three(w: &support::World) -> i64 {
    let [a, b, c] = w.partners;
    new_trip(w, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")])
}

fn new_trip(w: &support::World, partners: Vec<VenturePartnerInput>) -> i64 {
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners,
        },
    )
    .unwrap()
    .id
}

fn orange(w: &support::World) -> i64 {
    support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit")
}

fn purchase_input(
    w: &support::World,
    trip: i64,
    item: i64,
    qty: &str,
    rate: &str,
    freight: &str,
) -> PurchaseInput {
    PurchaseInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: Some("Orange Traders".into()),
        item_id: Some(item),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        freight: (!freight.is_empty()).then(|| freight.to_owned()),
        labour: None,
        other_cost: None,
        note: None,
        parts: Vec::<EntryPartInput>::new(),
        confirm_cash_below_zero: false,
    }
}

/// A purchase of the trip, saved. Returns its id.
fn buy(w: &support::World, trip: i64, item: i64, qty: &str, rate: &str, freight: &str) -> i64 {
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(w, trip, item, qty, rate, freight),
    )
    .unwrap()
    .id
}

fn get_purchase(w: &support::World, id: i64) -> PurchaseRow {
    purchases::get(&w.env.ctx, &PurchaseGetInput { id }).unwrap()
}

/// An own lot taking `qty` from a purchase, with no lot charges (PRD E2E-34).
fn own_lot(w: &support::World, key: &str, purchase_id: i64, qty: &str) -> LotInput {
    LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, key, w.rahim, w.banana, qty)
    }
}

/// Rupees of the partner share rows of one entry of one kind, in partner order.
fn shares_by_kind(w: &support::World, entry_id: i64, kind: &str) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT amount_paisa FROM partner_share_rows WHERE journal_entry_id = ?1 AND kind = ?2
             ORDER BY partner_id",
        )
        .unwrap();
    statement
        .query_map((entry_id, kind), |r| Ok(r.get::<_, i64>(0)? / 100))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn venture_account(w: &support::World, partner_id: i64) -> i64 {
    partners::get(&w.env.ctx, partner_id)
        .unwrap()
        .row
        .venture_account_id
}

fn venture_code(w: &support::World, partner_id: i64) -> String {
    partners::get(&w.env.ctx, partner_id)
        .unwrap()
        .row
        .venture_account_code
}

/// Σ credit − Σ debit of an account, in rupees.
fn credit_balance(w: &support::World, account_id: i64) -> i64 {
    let (dr, cr): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines
             WHERE account_id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    (cr - dr) / 100
}

/// What a partner has from commission and trips so far (PRD C22, without capital and drawings):
/// his income and expense shares plus his gardens, trips and fruit account.
fn partner_total(w: &support::World, index: usize) -> i64 {
    let partner = w.partners[index];
    let shares = scalar(
        w,
        &format!(
            "SELECT COALESCE(SUM(amount_paisa), 0) / 100 FROM partner_share_rows
             WHERE partner_id = {partner}"
        ),
    );
    shares + credit_balance(w, venture_account(w, partner))
}

fn totals_of_three(w: &support::World) -> Vec<i64> {
    (0..3).map(|i| partner_total(w, i)).collect()
}

/// The firm's equity worked out from the books alone (PRD FR-17.9): Σ (credit - debit) of every
/// income (04), expense (05) and capital (06) account, which holds the partners' capital, drawings,
/// and gardens, trips and fruit accounts. It does not use the share rows, so it is a separate proof
/// of the partner totals.
fn equity(w: &support::World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100 FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id
         WHERE substr(a.code, 1, 2) IN ('04', '05', '06')",
    )
}

/// Σ of the partners' totals + the firm's opening capital 65,000 is the equity in the books.
fn assert_equity_is_the_partner_totals(w: &support::World, expected: i64) {
    let sum: i64 = totals_of_three(w).iter().sum();
    assert_eq!(sum + 65_000, expected, "the partner totals add up");
    assert_eq!(equity(w), expected, "the equity worked out from the books");
}

/// The trial balance totals in rupees. The books balance: both sides are equal.
fn trial(w: &support::World) -> i64 {
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr, "the books balance");
    trial.total_dr.rupees()
}

fn trip_detail(w: &support::World, trip: i64) -> mandi_services::core::trips::TripDetail {
    trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap()
}

fn trip_status(w: &support::World, trip: i64) -> VentureStatus {
    trip_detail(w, trip).row.status
}

fn open(w: &support::World, bikri_id: i64, entry_id: i64) -> EditableBikri {
    bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap()
}

fn edit(w: &support::World, opened: &EditableBikri, draft: BikriDraft) -> BikriEdited {
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let saved = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(saved.preview, preview);
    saved
}

fn write_off(w: &support::World, purchase_id: i64, qty: &str) -> PurchaseWriteOffDone {
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id,
            qty: qty.into(),
            reason: "خراب".into(),
        },
    )
    .unwrap()
}

fn cancel_bikri(w: &support::World, bikri_id: i64) {
    let detail = bikri::get(&w.env.ctx, bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// The costs stored on the lines in force, by lot and line number.
fn line_costs(w: &support::World) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT cost_paisa / 100 FROM bikri_sales WHERE is_active = 1 ORDER BY lot_id, line_no",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn cash_balance(w: &support::World) -> i64 {
    balance(w, support::id_of(&w.env, "01010001")).1
}

/// E2E-34 up to the sale: a trip of the three partners, 20 پیٹی مالٹا at 500 with freight 400,
/// and an own lot that sells 15 of them to اکرم at 600 on credit.
struct Sale34 {
    trip: i64,
    purchase: i64,
    saved: BikriSaved,
}

fn sale_34(w: &support::World) -> Sale34 {
    let item = orange(w);
    let trip = trip_of_three(w);
    let purchase = buy(w, trip, item, "20", "500", "400");
    let draft = support::draft(vec![support::with_sales(
        own_lot(w, "own-1", purchase, "15"),
        vec![support::sale("s-1", w.akram, "15", "600")],
    )]);
    let saved = save_ok(w, &draft, false);
    Sale34 {
        trip,
        purchase,
        saved,
    }
}

/// E2E-34, the sale: 15 of the 20 are sold by an own lot to اکرم at 600 on credit.
#[test]
fn e2e_34_the_own_lot_sells_15_and_the_partners_get_the_seller_credit() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    assert_eq!(cash_balance(&w), 39_600);

    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "15"),
        vec![support::sale("s-1", w.akram, "15", "600")],
    )]);
    let p = support::preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let lot = &p.lots[0];
    assert!(lot.is_own);
    let t = lot.totals.as_ref().unwrap();
    assert_eq!(
        (
            support::r(t.gross),
            support::r(t.seller_comm),
            support::r(t.buyer_comm),
            support::r(t.laga),
            support::r(t.buyer_total),
            support::r(t.seller_credit),
            support::r(t.shared_income)
        ),
        (9_000, 540, 540, 120, 9_660, 8_460, 1_200)
    );
    // The cost of the goods sold (C21), the result and the partners' parts (C25).
    let own = lot.own.as_ref().unwrap();
    assert_eq!(
        (own.purchase_id, own.purchase_number, own.trip.number),
        (purchase, 1, 1)
    );
    assert_eq!(
        (
            own.cost_paisa.rupees(),
            own.credit_paisa.rupees(),
            own.result_paisa.rupees()
        ),
        (7_800, 8_460, 660)
    );
    let shown: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, vec![3_384, 2_538, 2_538]);
    assert_eq!(support::calc(lot, 0).cost_of_goods.rupees(), 7_800);
    let split: Vec<i64> = p.split.iter().map(|s| s.income_paisa.rupees()).collect();
    assert_eq!(
        split,
        vec![480, 360, 360],
        "commission 1,200 by the firm's shares"
    );

    let saved = save_ok(&w, &draft, false);
    assert_eq!(
        saved.preview, p,
        "the save works out what the preview showed"
    );
    // The entry: اکرم بنام 9,660; the partners' 0604 جمع 3,384 / 2,538 / 2,538; the income accounts.
    // 9,660 = 540 + 540 + 120 + 8,460. The cost is not in it.
    let [a, b, c] = w.partners;
    let (code_a, code_b, code_c) = (
        venture_code(&w, a),
        venture_code(&w, b),
        venture_code(&w, c),
    );
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[
            ("03040001", 9_660, 0),
            (&code_a, 0, 3_384),
            (&code_b, 0, 2_538),
            (&code_c, 0, 2_538),
            ("04010001", 0, 540),
            ("04010002", 0, 540),
            ("04010003", 0, 120),
        ])
    );
    assert_eq!(
        shares_by_kind(&w, saved.entry_id, "income"),
        vec![480, 360, 360]
    );
    assert!(shares_by_kind(&w, saved.entry_id, "expense").is_empty());
    // Nothing is owed to the اپنا مال account: it never holds a balance.
    assert_eq!(balance(&w, support::id_of(&w.env, "030119998")).1, 0);
    // Partner totals: -4,160 + 480 + 3,384 = -296, and -3,120 + 360 + 2,538 = -222 twice.
    assert_eq!(totals_of_three(&w), vec![-296, -222, -222]);
    // Σ -740 + 65,000 = 64,260 = the equity in the books (capital 65,000, the partners' trip
    // accounts -1,940, the income 1,200).
    assert_equity_is_the_partner_totals(&w, 64_260);
    // Cash did not move for the sale on credit.
    assert_eq!(cash_balance(&w), 39_600);
    // The saved lot shows the stored cost, credit and result.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let saved_own = detail.lots[0].own.as_ref().unwrap();
    assert_eq!(
        (
            saved_own.cost_paisa.rupees(),
            saved_own.credit_paisa.rupees(),
            saved_own.result_paisa.rupees()
        ),
        (7_800, 8_460, 660)
    );
    let stored: Vec<i64> = saved_own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(stored, vec![3_384, 2_538, 2_538]);
    let from = detail.lots[0].purchase.as_ref().unwrap();
    assert_eq!((from.number, from.trip_number), (1, 1));
    // The purchase: 15 sold, 5 left with a cost of 2,600; the trip stays open.
    let row = get_purchase(&w, purchase);
    assert_eq!((row.qty_sold, row.qty_left), (15, 5));
    assert_eq!(
        (
            row.cost_sold_paisa.rupees(),
            row.cost_left_paisa.rupees(),
            row.sales_paisa.rupees(),
            row.result_paisa.rupees()
        ),
        (7_800, 2_600, 8_460, 660)
    );
    assert_eq!(trip_status(&w, trip), VentureStatus::Open);
    // The stock list shows the 5 left, with their trip.
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(stock.own_rows.len(), 1);
    assert_eq!(
        (
            stock.own_rows[0].balance,
            stock.own_rows[0].cost_left_paisa.rupees(),
            stock.own_rows[0].trip.id
        ),
        (5, 2_600, trip)
    );
    // The analytics fact carries the trip and the cost.
    assert_eq!(
        scalar(
            &w,
            &format!("SELECT COUNT(*) FROM analytics_facts WHERE venture_id = {trip}")
        ),
        1
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT SUM(sign * cost_paisa) / 100 FROM analytics_facts"
        ),
        7_800
    );
    // Dr: cash 39,600 + رحیم 15,000 + اکرم 9,660 + the partners 1,940 = Cr: 65,000 + 1,200.
    assert_eq!(trial(&w), 66_200);
}

/// E2E-34b: on 11-10-26 the last 5 spoil. Nothing posts. The trip closes by itself.
#[test]
fn e2e_34b_the_last_5_spoil_and_the_trip_closes() {
    let w = worked_day();
    let s = sale_34(&w);
    set_today(&w.env, 2026, 10, 11);
    let entries = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    let off = write_off(&w, s.purchase, "5");
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
        entries,
        "a write-off posts nothing"
    );
    assert_eq!(
        off.cost_paisa.rupees(),
        2_600,
        "the last nag take the cost left"
    );
    assert!(off.trip_closed);
    // The result of the trip: 8,460 - 7,800 - 2,600 = -1,940; each partner's part -776 / -582 / -582.
    let detail = trip_detail(&w, s.trip);
    assert_eq!(detail.row.status, VentureStatus::Closed);
    assert_eq!(detail.row.result_paisa.rupees(), -1_940);
    let nets: Vec<i64> = detail
        .partner_results
        .iter()
        .map(|p| p.net_paisa.rupees())
        .collect();
    assert_eq!(nets, vec![-776, -582, -582]);
    // The partner totals did not move: the loss was in them already.
    assert_eq!(totals_of_three(&w), vec![-296, -222, -222]);
    // Σ -740 + 65,000 = 64,260 = the firm's equity: capital 65,000 less the partners' trip accounts
    // 1,940 plus the income 1,200. Worked out from the books too, not only from the totals.
    assert_equity_is_the_partner_totals(&w, 64_260);
    assert_eq!(trial(&w), 66_200);
    let row = get_purchase(&w, s.purchase);
    assert_eq!((row.qty_left, row.cost_left_paisa.rupees()), (0, 0));
    assert_eq!(row.result_paisa.rupees(), -1_940);
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert!(stock.own_rows.is_empty());
}

/// E2E-41: الف 50% from the shop, ب 50% from his pocket, ج not in the trip. 80 پیٹی مالٹا at 1,200
/// and 4,000 for the truck. An own lot sells all 80 to ارشد at 1,400 on credit.
#[test]
fn e2e_41_a_pocket_source_a_truck_and_one_own_lot_that_sells_everything() {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")]);
    let purchase = buy(&w, trip, item, "80", "1,200", "");
    trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            amount: "4,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash_balance(&w), 0, "الف paid 48,000 + 2,000 from the shop");
    let bought = get_purchase(&w, purchase);
    assert_eq!(
        (
            bought.total_cost_paisa.rupees(),
            bought.avg_cost_paisa.rupees()
        ),
        (100_000, 1_250)
    );

    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "80"),
        vec![support::sale("s-1", w.arshad, "80", "1,400")],
    )]);
    let p = support::preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let lot = &p.lots[0];
    let t = lot.totals.as_ref().unwrap();
    assert_eq!(
        (
            support::r(t.gross),
            support::r(t.seller_comm),
            support::r(t.buyer_comm),
            support::r(t.laga),
            support::r(t.buyer_total),
            support::r(t.seller_credit),
            support::r(t.shared_income)
        ),
        (112_000, 6_720, 6_720, 640, 119_360, 105_280, 14_080)
    );
    let own = lot.own.as_ref().unwrap();
    assert_eq!(
        (
            own.cost_paisa.rupees(),
            own.credit_paisa.rupees(),
            own.result_paisa.rupees()
        ),
        (100_000, 105_280, 5_280),
        "the last nag take all the cost left"
    );
    let shown: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, vec![52_640, 52_640], "ج is not in the trip");
    let split: Vec<i64> = p.split.iter().map(|s| s.income_paisa.rupees()).collect();
    assert_eq!(split, vec![5_632, 4_224, 4_224]);

    let saved = save_ok(&w, &draft, false);
    let (code_a, code_b) = (venture_code(&w, a), venture_code(&w, b));
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[
            ("03040002", 119_360, 0),
            (&code_a, 0, 52_640),
            (&code_b, 0, 52_640),
            ("04010001", 0, 6_720),
            ("04010002", 0, 6_720),
            ("04010003", 0, 640),
        ])
    );
    assert_eq!(
        shares_by_kind(&w, saved.entry_id, "income"),
        vec![5_632, 4_224, 4_224]
    );
    // Everything is sold: the trip closes by itself.
    assert_eq!(trip_status(&w, trip), VentureStatus::Closed);
    // الف: -50,000 + 5,632 + 52,640 = 8,272. ب paid from his pocket: 4,224 + 52,640 = 56,864.
    assert_eq!(totals_of_three(&w), vec![8_272, 56_864, 4_224]);
    // Σ 69,360 + 65,000 = 134,360 = the equity in the books (ب's pocket part is not in the books).
    assert_equity_is_the_partner_totals(&w, 134_360);
    let detail = trip_detail(&w, trip);
    assert_eq!(detail.row.result_paisa.rupees(), 5_280);
    let nets: Vec<i64> = detail
        .partner_results
        .iter()
        .map(|p| p.net_paisa.rupees())
        .collect();
    assert_eq!(
        nets,
        vec![2_640, 2_640],
        "الف +2,640, ب his 50,000 back + 2,640"
    );
    assert_eq!(trial(&w), 134_360);
}

/// The partners' parts follow an edit: the old posting is reversed with the opposite rows, and the
/// new posting writes new ones (PRD FR-12.26, C25). The cost stays as it was (PRD FR-12.20).
/// Cost 10 over 6 نگ: a line of 3 takes 5, and a line of 1 out of the 3 left takes 1.
#[test]
fn an_edit_that_changes_only_a_rate_keeps_the_stored_costs_and_moves_the_credit() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "6", "1", "4");
    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "4"),
        vec![
            support::sale("s-1", w.akram, "3", "100"),
            support::sale("s-2", w.arshad, "1", "100"),
        ],
    )]);
    let saved = save_ok(&w, &draft, false);
    assert_eq!(line_costs(&w), vec![5, 1]);
    // Credit 400 - 24 = 376: 152 / 112 / 112 (the leftover rupees go to the largest share).
    let before = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let own = before.lots[0].own.as_ref().unwrap();
    assert_eq!(own.credit_paisa.rupees(), 376);
    let parts: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(parts, vec![152, 112, 112]);

    let opened = open(&w, saved.bikri_id, saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[0].sales[0].seller_rate = "110".into();
    let edited = edit(&w, &opened, changed);
    assert_eq!(line_costs(&w), vec![5, 1], "the stored costs stay");
    let row = get_purchase(&w, purchase);
    assert_eq!(
        (row.qty_sold, row.cost_released_paisa.rupees(), row.qty_left),
        (4, 6, 2)
    );
    // Gross 330 + 100 = 430, commission 20 + 6 = 26, credit 404: 162 / 121 / 121.
    let after = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let own = after.lots[0].own.as_ref().unwrap();
    assert_eq!(
        (
            own.cost_paisa.rupees(),
            own.credit_paisa.rupees(),
            own.result_paisa.rupees()
        ),
        (6, 404, 398)
    );
    let parts: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(parts, vec![162, 121, 121]);
    // The posting rows: 3 first, 3 opposite, 3 new.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM lot_credit_parts"), 9);
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM lot_credit_parts WHERE journal_entry_id = {}",
                edited.entry_id
            )
        ),
        3
    );
    // Edit the rate back: every number returns.
    let reopened = open(&w, saved.bikri_id, edited.entry_id);
    let mut back = reopened.draft.clone();
    back.lots[0].sales[0].seller_rate = "100".into();
    edit(&w, &reopened, back);
    let again = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let own = again.lots[0].own.as_ref().unwrap();
    assert_eq!(own.credit_paisa.rupees(), 376);
    assert_eq!(line_costs(&w), vec![5, 1]);
    trial(&w);
}

/// After E2E-34b the rate of اکرم's line is edited to 610. The cost stays 7,800. The old posting
/// gave its 15 نگ back (the trip opens), the new posting takes them again (the trip closes).
#[test]
fn an_edit_after_the_write_off_gives_back_the_exact_stock_and_the_trip_closes_again() {
    let w = worked_day();
    let s = sale_34(&w);
    set_today(&w.env, 2026, 10, 11);
    write_off(&w, s.purchase, "5");
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Closed);

    let opened = open(&w, s.saved.bikri_id, s.saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[0].sales[0].seller_rate = "610".into();
    let edited = edit(&w, &opened, changed);
    let lot = &edited.preview.lots[0];
    let t = lot.totals.as_ref().unwrap();
    // 15 x 610 = 9,150: commission 549 each side, لاگا 120, the bill 9,819, the credit 8,601.
    assert_eq!(
        (
            support::r(t.seller_comm),
            support::r(t.seller_credit),
            support::r(t.buyer_comm),
            support::r(t.buyer_total),
            support::r(t.shared_income)
        ),
        (549, 8_601, 549, 9_819, 1_218)
    );
    let own = lot.own.as_ref().unwrap();
    assert_eq!(
        (own.cost_paisa.rupees(), own.result_paisa.rupees()),
        (7_800, 801)
    );
    let shown: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, vec![3_441, 2_580, 2_580]);
    assert_eq!(
        shares_by_kind(&w, edited.entry_id, "income"),
        vec![488, 365, 365]
    );
    let after = get_purchase(&w, s.purchase);
    assert_eq!(
        (
            after.cost_released_paisa.rupees(),
            after.result_paisa.rupees(),
            after.qty_left
        ),
        (10_400, -1_799, 0)
    );
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Closed);
    let nets: Vec<i64> = trip_detail(&w, s.trip)
        .partner_results
        .iter()
        .map(|p| p.net_paisa.rupees())
        .collect();
    assert_eq!(nets, vec![-719, -540, -540]);
    trial(&w);
}

/// Cancelling the sale gives back exactly the 15 نگ and the cost 7,800, takes the partners' parts
/// out again, and opens the trip. The purchase can then be cancelled only after the write-off is
/// undone.
#[test]
fn cancelling_the_sale_reverses_the_partner_credits_and_opens_the_trip() {
    let w = worked_day();
    let s = sale_34(&w);
    set_today(&w.env, 2026, 10, 11);
    let off = write_off(&w, s.purchase, "5");
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Closed);
    cancel_bikri(&w, s.saved.bikri_id);
    let back = get_purchase(&w, s.purchase);
    assert_eq!(
        (
            back.qty_sold,
            back.qty_left,
            back.cost_left_paisa.rupees(),
            back.result_paisa.rupees()
        ),
        (0, 15, 7_800, -2_600)
    );
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Open);
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM bikri_sales WHERE is_active = 1"),
        0
    );
    // The facts, the credit rows and the partners' accounts net to nothing but the purchase.
    assert_eq!(
        scalar(&w, "SELECT SUM(sign * cost_paisa) FROM analytics_facts"),
        0
    );
    assert_eq!(
        scalar(&w, "SELECT SUM(amount_paisa) FROM lot_credit_parts"),
        0
    );
    assert_eq!(totals_of_three(&w), vec![-4_160, -3_120, -3_120]);
    assert_eq!(cash_balance(&w), 39_600);
    let refused = purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: s.purchase,
            version: back.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(first_field(&refused).as_deref(), Some("id"));
    assert!(format!("{refused:?}").contains("ErrPurchaseHasSales"));
    purchases::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffUndoInput {
            move_id: off.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    let whole = get_purchase(&w, s.purchase);
    assert_eq!(
        (whole.qty_left, whole.cost_left_paisa.rupees()),
        (20, 10_400)
    );
    purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: s.purchase,
            version: whole.version,
            reason: "غلط خریداری".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash_balance(&w), 50_000);
    assert_eq!(totals_of_three(&w), vec![0, 0, 0]);
    // A cancelled purchase cannot be sold from.
    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-2", s.purchase, "1"),
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    assert!(support::has(
        &support::preview(&w, &draft).errors,
        ErrorCode::ErrPurchaseCancelled,
        "lots[0].purchase_id"
    ));
    assert_eq!(trial(&w), 65_000);
}

/// A loss on the lot: 1 نگ at 10 with labour 50 typed. The seller credit is 10 - 1 - 50 = -41. It
/// is split 17 / 12 / 12 and posted بنام on the partners' accounts, never as a negative line.
#[test]
fn a_negative_seller_credit_is_posted_on_the_debit_side_of_the_partners() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "2", "5", "");
    let mut lot = own_lot(&w, "own-1", purchase, "1");
    lot.labour = support::manual("50");
    let draft = support::draft(vec![support::with_sales(
        lot,
        vec![support::sale("s-1", w.akram, "1", "10")],
    )]);
    let p = support::preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert!(
        !support::has(
            &p.warnings,
            ErrorCode::ErrLotNetNegative,
            "lots[0].seller_account_id"
        ),
        "the loss of trip goods is not the seller's debt"
    );
    let own = p.lots[0].own.as_ref().unwrap();
    assert_eq!(own.credit_paisa.rupees(), -41);
    let shown: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, vec![-17, -12, -12]);
    // The cost of 1 of 2 نگ: 10 over 2 = 5. The result is -41 - 5 = -46.
    assert_eq!(
        (own.cost_paisa.rupees(), own.result_paisa.rupees()),
        (5, -46)
    );
    let saved = save_ok(&w, &draft, false);
    let [a, b, c] = w.partners;
    let (code_a, code_b, code_c) = (
        venture_code(&w, a),
        venture_code(&w, b),
        venture_code(&w, c),
    );
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[
            ("03040001", 19, 0),
            (&code_a, 17, 0),
            (&code_b, 12, 0),
            (&code_c, 12, 0),
            ("04010001", 0, 1),
            ("04010002", 0, 1),
            ("04010003", 0, 8),
            ("02010001", 0, 50),
        ])
    );
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let stored: Vec<i64> = detail.lots[0]
        .own
        .as_ref()
        .unwrap()
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(stored, vec![-17, -12, -12]);
    trial(&w);
}

/// A weight purchase: 3 نگ, 12.345 kg at 100, value 1,235 (rounded once). One نگ takes 411.
#[test]
fn a_weight_purchase_is_costed_by_nag() {
    let w = worked_day();
    let trip = trip_of_three(&w);
    let mut input = purchase_input(&w, trip, w.weight_fruit, "3", "100", "");
    input.weight = Some("12.345".into());
    let saved = purchases::save(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(saved.total_cost.rupees(), 1_235);
    let mut lot = own_lot(&w, "own-1", saved.id, "1");
    lot.weight_received = Some("4.115".into());
    let mut line = support::sale("s-1", w.akram, "1", "100");
    line.weight = Some("4.115".into());
    let draft = support::draft(vec![support::with_sales(lot, vec![line])]);
    let p = support::preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(support::calc(&p.lots[0], 0).cost_of_goods.rupees(), 411);
}

/// Cost 1,000 over 3 نگ: two lots of 1 take 333 and 334; the last نگ takes the 333 that is left.
/// Two own lots of one purchase in one بکری share one running state, in lot order.
#[test]
fn a_cost_that_does_not_divide_is_released_in_full_over_lots_and_the_last_nag() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    // 3 x 332 = 996 + freight 4 = 1,000.
    let purchase = buy(&w, trip, item, "3", "332", "4");
    let draft = support::draft(vec![
        support::with_sales(
            own_lot(&w, "own-1", purchase, "1"),
            vec![support::sale("s-1", w.akram, "1", "600")],
        ),
        support::with_sales(
            own_lot(&w, "own-2", purchase, "1"),
            vec![support::sale("s-2", w.arshad, "1", "600")],
        ),
    ]);
    let saved = save_ok(&w, &draft, false);
    // The first nag: split_by_qty(1,000, [1, 2]) gives 333 / 667. The second takes the first part
    // of split_by_qty(667, [1, 1]) = 333 / 334, and on a tie the leftover goes to the qty taken
    // (C21), so it takes 334. One nag is left with 333.
    assert_eq!(line_costs(&w), vec![333, 334]);
    let left = get_purchase(&w, purchase);
    assert_eq!((left.qty_left, left.cost_left_paisa.rupees()), (1, 333));
    set_today(&w.env, 2026, 10, 11);
    let off = write_off(&w, purchase, "1");
    assert_eq!(
        off.cost_paisa.rupees(),
        333,
        "the last nag takes the cost left"
    );
    let done = get_purchase(&w, purchase);
    assert_eq!(done.cost_released_paisa.rupees(), 1_000);
    assert_eq!(trip_status(&w, trip), VentureStatus::Closed);
    cancel_bikri(&w, saved.bikri_id);
    let reopened = get_purchase(&w, purchase);
    assert_eq!(
        (reopened.qty_left, reopened.cost_left_paisa.rupees()),
        (2, 667)
    );
    trial(&w);
}

fn first_error_code(error: &mandi_services::ServiceError) -> String {
    format!("{error:?}")
}

/// Refusals of an own lot: not sold in full, more than the stock, the wrong use of the اپنا مال
/// account, and a write-off of more than is left.
#[test]
fn refusals_of_an_own_lot() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    // Received 15, lines sell 10.
    let short = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "15"),
        vec![support::sale("s-1", w.akram, "10", "600")],
    )]);
    let p = support::preview(&w, &short);
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrOwnLotNotFullySold,
        "lots[0].qty_received"
    ));
    assert!(
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::bikri::BikriSaveInput {
                draft: short,
                confirm_cash_below_zero: false
            }
        )
        .is_err()
    );
    // More than the purchase has.
    let over = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "21"),
        vec![support::sale("s-1", w.akram, "21", "600")],
    )]);
    assert!(support::has(
        &support::preview(&w, &over).errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].qty_received"
    ));
    // Two lots of one purchase that together take 21 of 20.
    let two = support::draft(vec![
        support::with_sales(
            own_lot(&w, "own-1", purchase, "12"),
            vec![support::sale("s-1", w.akram, "12", "600")],
        ),
        support::with_sales(
            own_lot(&w, "own-2", purchase, "9"),
            vec![support::sale("s-2", w.arshad, "9", "600")],
        ),
    ]);
    assert!(support::has(
        &support::preview(&w, &two).errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[1].sales[0].qty"
    ));
    // اپنا مال without a purchase, and a purchase on an ordinary seller.
    let mut no_purchase = own_lot(&w, "own-1", purchase, "1");
    no_purchase.purchase_id = None;
    let d = support::draft(vec![support::with_sales(
        no_purchase,
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    assert!(support::has(
        &support::preview(&w, &d).errors,
        ErrorCode::ErrRequired,
        "lots[0].purchase_id"
    ));
    let mut plain = support::lot(&w, "lot-1", w.rahim, w.banana, "1");
    plain.purchase_id = Some(purchase);
    let d = support::draft(vec![support::with_sales(
        plain,
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    assert!(support::has(
        &support::preview(&w, &d).errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].purchase_id"
    ));
    // A different item than the purchase's.
    let mut other = own_lot(&w, "own-1", purchase, "1");
    other.item_id = Some(w.banana);
    let d = support::draft(vec![support::with_sales(
        other,
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    assert!(support::has(
        &support::preview(&w, &d).errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].item_id"
    ));
    // نقد is refused on an own lot; a typed freight only warns.
    let mut cash = own_lot(&w, "own-1", purchase, "1");
    cash.cash_paid = Some("100".into());
    cash.freight = Some("50".into());
    let d = support::draft(vec![support::with_sales(
        cash,
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    let p = support::preview(&w, &d);
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].cash_paid"
    ));
    assert!(support::has(
        &p.warnings,
        ErrorCode::ErrOwnLotFreightInCost,
        "lots[0].freight"
    ));
    // As the زمیندار of the header, اپنا مال is refused.
    let mut header = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "1"),
        vec![support::sale("s-1", w.akram, "1", "600")],
    )]);
    header.header.zamindar_account_id = Some(support::id_of(&w.env, "030119998"));
    assert!(support::has(
        &support::preview(&w, &header).errors,
        ErrorCode::ErrOwnGoodsAccountRestricted,
        "header.zamindar_account_id"
    ));
    // As a buyer.
    let d = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "1"),
        vec![support::sale(
            "s-1",
            support::id_of(&w.env, "030119998"),
            "1",
            "600",
        )],
    )]);
    assert!(support::has(
        &support::preview(&w, &d).errors,
        ErrorCode::ErrOwnGoodsAccountRestricted,
        "lots[0].sales[0].buyer_account_id"
    ));
    // The write-off is more than the stock left.
    let big = purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: purchase,
            qty: "21".into(),
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert_eq!(first_field(&big).as_deref(), Some("qty"));
    // Nothing above wrote anything: only the purchase itself is there.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM lot_credit_parts"), 0);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM purchases"), 1);
    trial(&w);
}

/// An own lot cannot be dated before its purchase, as a sale in another shop cannot (money review
/// of plan 16b, second pass, finding 1; PRD FR-12.24, FR-12.26, FR-12.32). The purchase is bought
/// on 11-10-26; a بکری dated 10-10-26 (a day that is still open) is refused, one dated 11-10-26 is
/// not.
#[test]
fn an_own_lot_dated_before_its_purchase_is_refused() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    set_today(&w.env, 2026, 10, 11);
    let mut input = purchase_input(&w, trip, item, "20", "500", "400");
    input.date = "11-10-2026".into();
    let purchase = purchases::save(&w.env.ctx, &w.owner, &input).unwrap().id;
    let mut draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "15"),
        vec![support::sale("s-1", w.akram, "15", "600")],
    )]);
    let early = support::preview(&w, &draft);
    assert!(
        support::has(
            &early.errors,
            ErrorCode::ErrDateBeforeReceived,
            "lots[0].purchase_id"
        ),
        "{:?}",
        early.errors
    );
    let entries = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    assert!(
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::bikri::BikriSaveInput {
                draft: draft.clone(),
                confirm_cash_below_zero: false
            }
        )
        .is_err()
    );
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
        entries,
        "a refused save writes nothing"
    );
    // The day of the purchase is fine.
    draft.header.entry_date = "11-10-2026".into();
    let same_day = support::preview(&w, &draft);
    assert!(same_day.errors.is_empty(), "{:?}", same_day.errors);
}

/// No voucher, جنرل entry or opening can put a balance on اپنا مال (plan 16b acceptance 4,
/// D4-A31). Only an own lot may use it.
#[test]
fn nothing_but_an_own_lot_can_use_the_own_goods_account() {
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let w = worked_day();
    let own = support::id_of(&w.env, "030119998");
    let e = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt_input(own, "100")).unwrap_err();
    assert_eq!(first_field(&e).as_deref(), Some("account_id"));
    assert!(
        first_error_code(&e).contains("ErrOwnGoodsAccountRestricted"),
        "{e:?}"
    );
    let e =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(own, "100", "")).unwrap_err();
    assert!(
        first_error_code(&e).contains("ErrOwnGoodsAccountRestricted"),
        "{e:?}"
    );
    let advance = AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: own,
        amount: "100".into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    };
    let e = vouchers::save_advance(&w.env.ctx, &w.owner, &advance).unwrap_err();
    assert!(
        first_error_code(&e).contains("ErrOwnGoodsAccountRestricted"),
        "{e:?}"
    );
    // A جنرل entry through the one posting function.
    let cash = support::id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(own), Side::Dr, rupees(100)),
        LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(100)),
    ])
    .unwrap();
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    let e = w
        .env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date,
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap_err();
    assert!(
        first_error_code(&e).contains("ErrOwnGoodsAccountRestricted"),
        "{e:?}"
    );
    // An opening (D4-A31) on اپنا مال and on a partner's 0604 account.
    let partner_account = venture_account(&w, w.partners[0]);
    for id in [own, partner_account] {
        let version = accounts::get(&w.env.ctx, id).unwrap().version;
        let e = accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &OpeningsInput {
                lines: vec![OpeningLineInput {
                    account_id: id,
                    version,
                    opening_date: "10-10-2026".into(),
                    opening_dr: "100".into(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                }],
            },
        )
        .unwrap_err();
        assert!(
            first_error_code(&e).contains("ErrOpeningNotAllowed"),
            "{id}: {e:?}"
        );
    }
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE a.code = '030119998'"
        ),
        0
    );
}

// ---- Edits of an own lot (money review of plan 16b, second pass) ----

/// An edit that cuts the quantity from 15 to 14: the old 15 نگ and their 7,800 are given back, and
/// the 14 take split(10,400, [14, 6]) = 7,280 from the whole stock. 6 نگ with a cost of 3,120 stay.
/// The edit preview and the save agree (the `edit` helper checks it).
#[test]
fn an_edit_that_cuts_the_qty_takes_the_cost_again_from_the_whole_stock() {
    let w = worked_day();
    let s = sale_34(&w);
    let opened = open(&w, s.saved.bikri_id, s.saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[0].qty_received = "14".into();
    changed.lots[0].sales[0].qty = "14".into();
    let edited = edit(&w, &opened, changed);
    assert_eq!(line_costs(&w), vec![7_280]);
    let own = edited.preview.lots[0].own.as_ref().unwrap();
    // 14 x 600 = 8,400, commission 504, credit 7,896, result 7,896 - 7,280.
    assert_eq!(
        (
            own.cost_paisa.rupees(),
            own.credit_paisa.rupees(),
            own.result_paisa.rupees()
        ),
        (7_280, 7_896, 616)
    );
    let parts: Vec<i64> = own
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(parts.iter().sum::<i64>(), 7_896);
    assert_eq!(parts, vec![3_160, 2_368, 2_368]);
    let row = get_purchase(&w, s.purchase);
    assert_eq!(
        (
            row.qty_sold,
            row.qty_left,
            row.cost_released_paisa.rupees(),
            row.cost_left_paisa.rupees()
        ),
        (14, 6, 7_280, 3_120)
    );
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Open);
    trial(&w);
}

/// An edit that adds a second line, which sells the last 5: the first line keeps its 7,800, the new
/// line is the last نگ and takes the 2,600 that is left. The trip closes.
#[test]
fn an_edit_that_adds_a_line_gives_the_new_line_the_cost_left_and_closes_the_trip() {
    let w = worked_day();
    let s = sale_34(&w);
    let opened = open(&w, s.saved.bikri_id, s.saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[0].qty_received = "20".into();
    changed.lots[0]
        .sales
        .push(support::sale("s-2", w.arshad, "5", "580"));
    edit(&w, &opened, changed);
    assert_eq!(line_costs(&w), vec![7_800, 2_600]);
    let row = get_purchase(&w, s.purchase);
    assert_eq!(
        (row.qty_left, row.cost_left_paisa.rupees()),
        (0, 0),
        "all the cost is released with the last nag"
    );
    assert_eq!(row.cost_released_paisa.rupees(), 10_400);
    assert_eq!(trip_status(&w, s.trip), VentureStatus::Closed);
    trial(&w);
}

/// Two own lots of one purchase in one بکری: 10 and 6 of 20 / 10,400 take 5,200 and 3,120. An edit
/// keeps lot 1 and cuts lot 2 to 5: lot 1's 10 / 5,200 come out of the stock first, and lot 2 takes
/// split(5,200, [5, 5]) = 2,600.
#[test]
fn an_edit_of_two_own_lots_keeps_one_and_changes_the_other() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    let draft = support::draft(vec![
        support::with_sales(
            own_lot(&w, "own-1", purchase, "10"),
            vec![support::sale("s-1", w.akram, "10", "600")],
        ),
        support::with_sales(
            own_lot(&w, "own-2", purchase, "6"),
            vec![support::sale("s-2", w.arshad, "6", "600")],
        ),
    ]);
    let saved = save_ok(&w, &draft, false);
    assert_eq!(line_costs(&w), vec![5_200, 3_120]);
    let left = get_purchase(&w, purchase);
    assert_eq!((left.qty_left, left.cost_left_paisa.rupees()), (4, 2_080));

    let opened = open(&w, saved.bikri_id, saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[1].qty_received = "5".into();
    changed.lots[1].sales[0].qty = "5".into();
    edit(&w, &opened, changed);
    assert_eq!(line_costs(&w), vec![5_200, 2_600]);
    let row = get_purchase(&w, purchase);
    assert_eq!(
        (
            row.qty_sold,
            row.qty_left,
            row.cost_released_paisa.rupees(),
            row.cost_left_paisa.rupees()
        ),
        (15, 5, 7_800, 2_600)
    );
    trial(&w);
}

/// A trip cost is spread after 10 of 20 / 10,400 were sold (cost 5,200): the purchase's total cost
/// is 11,400 and its cost left 6,200. An edit that changes only the rate keeps the stored 5,200
/// (FR-12.20). An edit that cuts the qty to 8 gives back 10 / 5,200 and takes
/// split(11,400, [8, 12]) = 4,560, so 12 نگ with a cost of 6,840 stay.
#[test]
fn an_edit_after_a_trip_cost_keeps_the_stored_cost_or_takes_it_again() {
    let w = worked_day();
    let item = orange(&w);
    let trip = trip_of_three(&w);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "10"),
        vec![support::sale("s-1", w.akram, "10", "600")],
    )]);
    let saved = save_ok(&w, &draft, false);
    assert_eq!(line_costs(&w), vec![5_200]);
    trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            amount: "1,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let after_cost = get_purchase(&w, purchase);
    assert_eq!(
        (
            after_cost.total_cost_paisa.rupees(),
            after_cost.cost_left_paisa.rupees(),
            after_cost.qty_left
        ),
        (11_400, 6_200, 10)
    );

    let opened = open(&w, saved.bikri_id, saved.entry_id);
    let mut rate = opened.draft.clone();
    rate.lots[0].sales[0].seller_rate = "610".into();
    let edited = edit(&w, &opened, rate);
    assert_eq!(line_costs(&w), vec![5_200], "the stored cost stays");
    let same = get_purchase(&w, purchase);
    assert_eq!(
        (
            same.cost_released_paisa.rupees(),
            same.cost_left_paisa.rupees()
        ),
        (5_200, 6_200)
    );

    let reopened = open(&w, saved.bikri_id, edited.entry_id);
    let mut cut = reopened.draft.clone();
    cut.lots[0].qty_received = "8".into();
    cut.lots[0].sales[0].qty = "8".into();
    edit(&w, &reopened, cut);
    assert_eq!(line_costs(&w), vec![4_560]);
    let row = get_purchase(&w, purchase);
    assert_eq!(
        (
            row.qty_left,
            row.cost_released_paisa.rupees(),
            row.cost_left_paisa.rupees()
        ),
        (12, 4_560, 6_840)
    );
    trial(&w);
}

/// Undo straight after the save (PRD FR-7.47) of an own lot whose sale closed the trip: the trip
/// opens, the stock and the partners' parts come back, and the draft it gives back still names the
/// purchase, so it can be corrected and saved again with the same numbers.
#[test]
fn undo_after_the_save_of_an_own_lot_opens_the_trip_and_the_draft_can_be_saved_again() {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")]);
    let purchase = buy(&w, trip, item, "80", "1,200", "");
    let draft = support::draft(vec![support::with_sales(
        own_lot(&w, "own-1", purchase, "80"),
        vec![support::sale("s-1", w.arshad, "80", "1,400")],
    )]);
    let saved = save_ok(&w, &draft, false);
    assert_eq!(trip_status(&w, trip), VentureStatus::Closed);

    let undone = bikri::undo(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::UndoInput {
            id: saved.bikri_id,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(trip_status(&w, trip), VentureStatus::Open);
    let back = get_purchase(&w, purchase);
    assert_eq!(
        (back.qty_sold, back.qty_left, back.cost_left_paisa.rupees()),
        (0, 80, 96_000)
    );
    assert_eq!(
        scalar(&w, "SELECT SUM(amount_paisa) FROM lot_credit_parts"),
        0
    );
    assert_eq!(totals_of_three(&w), vec![-48_000, 0, 0]);
    assert_eq!(undone.draft.lots[0].purchase_id, Some(purchase));

    let again = save_ok(&w, &undone.draft, false);
    assert_ne!(again.bikri_id, saved.bikri_id);
    assert_eq!(trip_status(&w, trip), VentureStatus::Closed);
    // The same numbers as the sale had before the undo (no truck cost in this run): الف paid 48,000
    // from the shop, so -48,000 + 5,632 + 52,640; ب paid from his pocket, so 4,224 + 52,640.
    assert_eq!(totals_of_three(&w), vec![10_272, 56_864, 4_224]);
    let row = get_purchase(&w, purchase);
    assert_eq!(
        (row.qty_left, row.cost_released_paisa.rupees()),
        (0, 96_000)
    );
    trial(&w);
}

/// D4-A33: the statement of partner الف on the E2E-34 run (PRD FR-17.10). Every amount that
/// reaches him has a line of its own, from posted records: -4,160 (his shop part of purchase خ-1),
/// +480 (his part of the commission 1,200), +3,384 (his 40% of the seller credit 8,460), ending at
/// -296. After E2E-34b the write-off and the closing of the trip are note lines with no amount, and
/// the running total stays.
#[test]
fn d4_a33_the_statement_of_partner_a_on_the_trip_run() {
    use mandi_services::core::error::ErrorValue;
    use mandi_services::core::statement::{
        PartnersStatementInput, StatementDetail, StatementHeading,
    };
    let w = worked_day();
    let s = sale_34(&w);
    let a = w.partners[0];
    let statement_of = |w: &support::World| {
        partners::statement(
            &w.env.ctx,
            &PartnersStatementInput {
                partner_id: a,
                from: None,
                to: None,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let st = statement_of(&w);
    let rows: Vec<_> = st
        .lines
        .iter()
        .map(|l| {
            (
                l.heading,
                l.detail,
                l.number,
                l.secondary_number,
                l.amount_paisa.map(|p| p.rupees()),
                l.running_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                StatementHeading::Trip,
                StatementDetail::TripPaid,
                Some(1),
                Some(1),
                Some(-4_160),
                -4_160
            ),
            (
                StatementHeading::Bikri,
                StatementDetail::IncomeShare,
                Some(1),
                None,
                Some(480),
                -3_680
            ),
            (
                StatementHeading::Trip,
                StatementDetail::TripCredit,
                Some(1),
                Some(1),
                Some(3_384),
                -296
            ),
        ]
    );
    // The lines carry what the words need: a purchase, and the figures of the sale (PRD C21, C25).
    assert_eq!(
        st.lines[0].values.get("kind"),
        Some(&ErrorValue::Text("purchase".into()))
    );
    let credit = &st.lines[2];
    let number = |name: &str| match credit.values.get(name) {
        Some(ErrorValue::Number(n)) => *n,
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(
        (
            number("qty"),
            number("sales_paisa") / 100,
            number("cost_paisa") / 100,
            number("result_paisa") / 100,
            number("share_bps")
        ),
        (15, 8_460, 7_800, 660, 4_000)
    );
    assert_eq!(
        credit.subject.as_ref().map(|r| r.name_en.as_str()),
        Some("Orange")
    );
    assert_eq!(
        (
            st.opening_total_paisa.rupees(),
            st.closing_total_paisa.rupees()
        ),
        (0, -296)
    );
    // His total on the overview is the same number.
    assert_eq!(
        partners::overview(&w.env.ctx)
            .unwrap()
            .iter()
            .find(|r| r.partner_id == a)
            .map(|r| (r.total_paisa.rupees(), r.month_ventures_paisa.rupees())),
        Some((-296, -4_160 + 3_384))
    );

    // E2E-34b: the last 5 spoil on 11-10-26 and the trip closes. Two note lines, no amount.
    set_today(&w.env, 2026, 10, 11);
    write_off(&w, s.purchase, "5");
    let st = statement_of(&w);
    assert_eq!(st.lines.len(), 5);
    let notes: Vec<_> = st.lines[3..]
        .iter()
        .map(|l| {
            (
                l.entry_date.as_str(),
                l.detail,
                l.amount_paisa,
                l.running_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        notes,
        vec![
            ("2026-10-11", StatementDetail::TripWriteOff, None, -296),
            ("2026-10-11", StatementDetail::TripClosed, None, -296),
        ]
    );
    let closed = &st.lines[4];
    let figure = |name: &str| match closed.values.get(name) {
        Some(ErrorValue::Number(n)) => *n / 100,
        other => panic!("{name}: {other:?}"),
    };
    // Cost 10,400, sales 8,460, result -1,940, his 40% and his net: money back 3,384 less money in
    // 4,160 = -776 (money agent, plan 16b finding 3).
    assert_eq!(
        (
            figure("cost_paisa"),
            figure("sales_paisa"),
            figure("result_paisa"),
            figure("net_paisa")
        ),
        (10_400, 8_460, -1_940, -776)
    );
    assert_eq!(
        closed.values.get("share_bps"),
        Some(&ErrorValue::Number(4_000))
    );
    assert_eq!(st.closing_total_paisa.rupees(), -296);
    assert_eq!(
        (
            st.totals.plus_paisa.rupees(),
            st.totals.minus_paisa.rupees()
        ),
        (3_864, 4_160)
    );

    // A range starting on 11-10-26 opens at -296 and holds only the two notes.
    let ranged = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: a,
            from: Some("11-10-2026".into()),
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        (
            ranged.opening_total_paisa.rupees(),
            ranged.lines.len(),
            ranged.closing_total_paisa.rupees()
        ),
        (-296, 2, -296)
    );
}

/// The sale of E2E-34 cancelled: its reversal gives lines of its own, the amounts cancel, and the
/// statement still ends at the total of the books.
#[test]
fn the_statement_of_a_cancelled_trip_sale_nets_to_the_total_of_the_books() {
    use mandi_services::core::statement::PartnersStatementInput;
    let w = worked_day();
    let s = sale_34(&w);
    cancel_bikri(&w, s.saved.bikri_id);
    for (index, partner) in w.partners.iter().enumerate() {
        let st = partners::statement(
            &w.env.ctx,
            &PartnersStatementInput {
                partner_id: *partner,
                from: None,
                to: None,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        let total = partner_total(&w, index);
        assert_eq!(st.closing_total_paisa.rupees(), total);
        assert_eq!(
            st.lines.last().map(|l| l.running_paisa.rupees()),
            Some(total)
        );
        // Only the purchase part is left: -4,160 / -3,120 / -3,120.
        assert_eq!(total, [-4_160, -3_120, -3_120][index]);
    }
}
