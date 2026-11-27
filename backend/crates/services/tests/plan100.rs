//! Plan 100 (PRD v1.61, Umar 28-11-2026): net seller figures (A), the total worth (B), every cancel
//! with the partner code, the sign and a reason (C), the audit of security events (D), the voucher
//! lists of several types (F), the most used first in pickers (I) and the bell panel (J). Every
//! expected number is the plan's worked numbers, the PRD's worked day or worked out in the
//! comments.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::audit::{self, AuditEventKind, AuditFilters, AuditListInput, AuditRow};
use mandi_reports::dashboard::{
    self, BackupFacts, SellerDuesInput, SellerDuesSide, TotalWorthInput, TotalWorthSide,
    TotalWorthSource,
};
use mandi_reports::reports::{Cell, ReportFilters, ReportId};
use mandi_services::core::accounts::{
    AccountScope, AccountsTouchInput, BalanceSide, SearchAccountsInput,
};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::UndoInput;
use mandi_services::core::cancel::{
    ArrivalCancelRequest, ArrivalWriteOffUndoRequest, BikriCancelRequest, BikriWriteOffUndoRequest,
    CancelProof, FruitCancelRequest, GardenEntryCancelRequest, GardenReopenRequest,
    PurchaseCancelRequest, PurchaseWriteOffUndoRequest, TripEntryCancelRequest,
    VoucherCancelRequest,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::partner_code::PartnerCodeSetInput;
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesReportInput, OpenPayablesInput, VoucherEditInput, VoucherEntry,
    VoucherPreviewInput, VoucherType, VouchersListInput,
};
use mandi_services::{
    Secret, ServiceError, assist, audit as service_audit, auth, cancel_guard, day, partner_code,
    search, vouchers,
};
use proptest::prelude::*;
use reports_world::*;
use serde::de::DeserializeOwned;
use serde_json::json;

use common::OWNER_PASSWORD;

const CODE: &str = "4826";

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn typed<T: DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn rejections(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|r| (r.code, r.field.clone())).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn set_code(w: &support::World) {
    partner_code::set(
        &w.env.ctx,
        &w.owner,
        &PartnerCodeSetInput {
            password: OWNER_PASSWORD.into(),
            code: CODE.into(),
            code_again: CODE.into(),
        },
    )
    .unwrap();
    partner_code::lock(&w.env.ctx).unwrap();
}

fn proof(code: &str, sign: &str) -> CancelProof {
    CancelProof {
        partner_code: code.into(),
        sign: sign.into(),
    }
}

/// Rows of the books: a refused cancel must not change them.
fn book_rows(w: &support::World) -> (i64, i64, i64) {
    let conn = common::raw(&w.dir);
    let count = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    (
        count("SELECT COUNT(*) FROM journal_entries"),
        count("SELECT COUNT(*) FROM journal_lines"),
        count("SELECT COUNT(*) FROM vouchers WHERE status = 'cancelled'"),
    )
}

fn scalar(w: &support::World, sql: &str) -> i64 {
    common::raw(&w.dir)
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

// ---- A: the sellers' figures are netted -----------------------------------------------------------

/// فیصل's numbers of plan 100: payable 117,040 and advance 100,000 on کریم. رحیم keeps his open
/// opening advance of 15,000.
fn umar_sellers() -> support::World {
    let w = worked_day();
    let general = typed(json!({
        "date": "10-10-2026", "narration": null,
        "lines": [
            { "account_id": w.akram, "dr": "117,040", "cr": "", "narration": null },
            { "account_id": w.karim, "dr": "", "cr": "117,040", "narration": null }
        ],
        "confirm_cash_below_zero": false
    }));
    mandi_services::vouchers::general_save(&w.env.ctx, &w.owner, &general).unwrap();
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &typed::<AdvanceInput>(json!({
            "date": "10-10-2026", "seller_account_id": w.karim, "amount": "100,000",
            "expected_goods": null, "narration": null, "confirm_cash_below_zero": true
        })),
    )
    .unwrap();
    w
}

#[test]
fn umars_seller_shows_the_net_on_every_tile_and_page() {
    let w = umar_sellers();
    let db = w.env.ctx.db();
    // کریم: P 117,040, A 100,000, L 17,040 جمع. رحیم: P 0, A 15,000.
    let tiles = dashboard::get(db, off()).unwrap().tiles;
    assert_eq!(tiles.sellers_payable_paisa, rupees(17_040));
    assert_eq!(tiles.advances_open_paisa, rupees(15_000));
    assert_eq!(tiles.sellers_dues_paisa, rupees(0));
    let today = mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10").unwrap();
    let narrow = dashboard::sellers(db, today).unwrap();
    assert_eq!(
        (
            narrow.payable_paisa,
            narrow.advances_open_paisa,
            narrow.dues_paisa
        ),
        (rupees(17_040), rupees(15_000), rupees(0))
    );

    let page = dashboard::seller_dues(db, &SellerDuesInput::default()).unwrap();
    let karim = page
        .rows
        .iter()
        .find(|r| r.party.account_id == w.karim)
        .unwrap();
    assert_eq!(
        (
            karim.payable_paisa,
            karim.advance_open_paisa,
            karim.owes_paisa,
            karim.balance.amount_paisa,
            karim.balance.side
        ),
        (
            rupees(17_040),
            rupees(0),
            rupees(0),
            rupees(17_040),
            BalanceSide::Cr
        )
    );
    assert_eq!(page.shop_owes_paisa, tiles.sellers_payable_paisa);
    assert_eq!(page.advances_open_paisa, tiles.advances_open_paisa);
    // payable - advance - dues is the sum of the sellers' balances: 17,040 جمع and 15,000 بنام.
    assert_eq!(
        (page.total.amount_paisa, page.total.side),
        (rupees(2_040), BalanceSide::Cr)
    );
    let shop = dashboard::seller_dues(
        db,
        &SellerDuesInput {
            side: SellerDuesSide::ShopOwes,
            ..SellerDuesInput::default()
        },
    )
    .unwrap();
    assert_eq!(shop.total_count, 1);

    // ادائیگی: "پورا بقایا" fills the net; the working keeps the gross figures.
    let open = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.karim,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(
        (
            open.net_payable_paisa,
            open.open_paisa,
            open.advances_open_paisa
        ),
        (rupees(17_040), rupees(117_040), rupees(100_000))
    );

    // The پیشگی page lists only the advance no payable covers; its total is the tile.
    let page_of = |only_open| {
        vouchers::advances_report(
            &w.env.ctx,
            &AdvancesReportInput {
                seller_id: None,
                as_of: String::new(),
                offset: 0,
                limit: 100,
                search: None,
                only_open,
            },
        )
        .unwrap()
    };
    let only = page_of(true);
    assert_eq!(only.total_count, 1);
    assert_eq!(only.rows[0].account.id, w.rahim);
    assert_eq!(only.uncovered_paisa, Some(rupees(15_000)));
    let all = page_of(false);
    assert_eq!(all.open_paisa, rupees(115_000));
    assert_eq!(all.uncovered_paisa, Some(rupees(15_000)));
    let karim_row = all.rows.iter().find(|r| r.account.id == w.karim).unwrap();
    assert_eq!(karim_row.uncovered_paisa, Some(rupees(0)));
    assert_eq!(karim_row.advance_covered, Some(rupees(100_000)));
    assert_eq!(all.advance_covered, Some(rupees(100_000)));

    // The covered advance (Umar, follow-up): no screen shows it as پیشگی باقی.
    let card = assist::party_card(&w.env.ctx, w.karim).unwrap();
    assert_eq!(
        (card.advance_open, card.advance_covered, card.net_payable),
        (Some(rupees(0)), Some(rupees(100_000)), Some(rupees(17_040)))
    );
    assert!(card.warnings.is_empty());
    let advances = assist::advances(&w.env.ctx, w.karim).unwrap();
    assert_eq!(
        (
            advances.open_paisa,
            advances.uncovered_paisa,
            advances.advance_covered
        ),
        (rupees(100_000), rupees(0), Some(rupees(100_000)))
    );
    assert_eq!(
        (open.uncovered_advance_paisa, open.advance_covered),
        (rupees(0), Some(rupees(100_000)))
    );
    // رحیم: nothing covers his 15,000.
    let card = assist::party_card(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(
        (card.advance_open, card.advance_covered, card.net_payable),
        (Some(rupees(15_000)), None, Some(rupees(0)))
    );

    // R26 keeps the open advance and splits it: covered and uncovered.
    let r26 = report(&w, ReportId::R26, ReportFilters::default(), 0, 100);
    let row = |code: &str| {
        let row = r26
            .rows
            .iter()
            .find(|r| code_of(&r.cells[0]) == code)
            .unwrap();
        [
            money(&row.cells[4]),
            money(&row.cells[5]),
            money(&row.cells[6]),
        ]
    };
    let karim_code = karim_row.account.code.clone();
    assert_eq!(row(&karim_code), [100_000, 100_000, 0]);
    let totals: Vec<i64> = [4, 5, 6].iter().map(|i| money(&r26.totals[*i])).collect();
    assert_eq!(totals, vec![115_000, 100_000, 15_000]);
    // As of an earlier day the payables are not known, so the two columns are empty.
    let past = report(
        &w,
        ReportId::R26,
        ReportFilters {
            as_of: "09-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        100,
    );
    assert!(
        past.rows
            .iter()
            .all(|r| r.cells[5] == Cell::Empty && r.cells[6] == Cell::Empty)
    );
    assert_eq!(
        (past.totals[5].clone(), past.totals[6].clone()),
        (Cell::Empty, Cell::Empty)
    );

    // FR-15.7 and FR-7.68 use the net: 17,040 is not above it, 17,041 is.
    let preview = |amount: &str| {
        vouchers::preview(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry: VoucherEntry::Payment(payment(w.karim, amount)),
                replacing_voucher_id: None,
            },
        )
        .unwrap()
    };
    let whole = preview("17,040");
    assert_eq!(whole.payable_open_paisa, Some(rupees(17_040)));
    assert!(!whole.payable_is_zero_for_seller && whole.warnings.is_empty());
    let above = preview("17,041");
    assert_eq!(above.warnings.len(), 1);
    assert_eq!(above.warnings[0].code, ErrorCode::ErrPaymentAbovePayable);
    // رحیم has an advance and nothing payable: the پیشگی note.
    let rahim = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Payment(payment(w.rahim, "1,000")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(rahim.payable_is_zero_for_seller);
}

// ---- B: کل مالیت ----------------------------------------------------------------------------------

/// The books' equity (heads 04, 05 and 06) without the linked partner accounts 0605, جمع above 0:
/// what the net of کل مالیت must be (plan 100 B).
fn equity(w: &support::World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         JOIN account_heads h ON h.id = k.head_id
         WHERE h.code IN ('04', '05', '06') AND k.code <> '0605'",
    )
}

/// The tile is the books: have less give is the equity, every part adds up to its side, every
/// row of a side adds up to its part, and the named parts equal their own tiles.
fn assert_total_worth_is_the_books(w: &support::World) {
    let db = w.env.ctx.db();
    let tiles = dashboard::get(db, off()).unwrap().tiles;
    let mut sides = Vec::new();
    for side in [TotalWorthSide::Have, TotalWorthSide::Give] {
        let page = dashboard::total_worth(
            db,
            &TotalWorthInput {
                side,
                offset: 0,
                limit: 100,
            },
        )
        .unwrap();
        assert_eq!(page.totals, tiles.total_worth);
        assert_eq!(page.total_count, i64::try_from(page.rows.len()).unwrap());
        let mut side_total = 0;
        for part in page.parts.iter().filter(|p| p.side == side) {
            let rows: i64 = page
                .rows
                .iter()
                .filter(|r| r.source == part.source)
                .map(|r| r.amount_paisa.rupees())
                .sum();
            assert_eq!(rows, part.total_paisa.rupees(), "{:?}", part.source);
            side_total += rows;
        }
        sides.push(side_total);
        let part = |source| {
            page.parts
                .iter()
                .find(|p| p.source == source)
                .map_or(rupees(0), |p| p.total_paisa)
        };
        if side == TotalWorthSide::Have {
            assert_eq!(part(TotalWorthSource::Cash), tiles.cash_in_hand_paisa);
            assert_eq!(part(TotalWorthSource::BuyersDues), tiles.buyers_dues_paisa);
            assert_eq!(
                part(TotalWorthSource::SellersDues),
                tiles.sellers_dues_paisa
            );
            assert_eq!(part(TotalWorthSource::DeadAmount), tiles.dead_amount_paisa);
            assert_eq!(part(TotalWorthSource::MoneyLent), tiles.lent_open_paisa);
            assert_eq!(
                part(TotalWorthSource::OpenAdvances),
                tiles.advances_open_paisa
            );
        } else {
            assert_eq!(
                part(TotalWorthSource::SellersPayable),
                tiles.sellers_payable_paisa
            );
        }
    }
    assert_eq!(sides[0], tiles.total_worth.have_paisa.rupees());
    assert_eq!(sides[1], tiles.total_worth.give_paisa.rupees());
    assert_eq!(tiles.total_worth.net_paisa.rupees(), sides[0] - sides[1]);
    assert_eq!(
        tiles.total_worth.net_paisa.rupees(),
        equity(w),
        "net = equity"
    );
    // The linked partner accounts (0605) and their contra are outside both: they cancel out.
    let linked = scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('0605', '0307')",
    );
    assert_eq!(linked, 0, "0605 and the contra cancel out");
    assert_cards_are_the_tiles(w);
}

/// The party card works from the balance and the tiles from the payable working: over every
/// seller not in the dead amount, the cards' open advances add up to the tile پیشگی باقی, and each
/// card's net payable is his row of the seller dues page (money review B3).
fn assert_cards_are_the_tiles(w: &support::World) {
    let db = w.env.ctx.db();
    let tiles = dashboard::get(db, off()).unwrap().tiles;
    let page = dashboard::seller_dues(
        db,
        &SellerDuesInput {
            limit: 100,
            ..SellerDuesInput::default()
        },
    )
    .unwrap();
    let sellers: Vec<i64> = {
        let conn = common::raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
                 WHERE k.code = '03011'",
            )
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let mut advances = 0;
    for id in sellers {
        let card = assist::party_card(&w.env.ctx, id).unwrap();
        if card.dead.is_some() {
            continue;
        }
        advances += card.advance_open.map_or(0, |a| a.rupees());
        let row_payable = page
            .rows
            .iter()
            .find(|r| r.party.account_id == id)
            .map_or(0, |r| r.payable_paisa.rupees());
        assert_eq!(
            card.net_payable.map_or(0, |p| p.rupees()),
            row_payable,
            "account {id}"
        );
    }
    assert_eq!(advances, tiles.advances_open_paisa.rupees());
}

#[test]
fn total_worth_of_the_worked_day_is_65_488_which_is_the_equity() {
    // Plan 100 B: have 58,484 + 21,564 = 80,048; give 10,160 + 300 + 4,000 + 100 = 14,560; net
    // 65,488 = capital 65,000 + profit 2,488 - drawings 2,000.
    let w = full_day();
    let tile = dashboard::get(w.env.ctx.db(), off())
        .unwrap()
        .tiles
        .total_worth;
    assert_eq!(
        (tile.have_paisa, tile.give_paisa, tile.net_paisa),
        (rupees(80_048), rupees(14_560), rupees(65_488))
    );
    assert_eq!(equity(&w), 65_488);
    assert_total_worth_is_the_books(&w);
    let give = dashboard::total_worth(
        w.env.ctx.db(),
        &TotalWorthInput {
            side: TotalWorthSide::Give,
            offset: 0,
            limit: 100,
        },
    )
    .unwrap();
    let payables: Vec<i64> = give
        .rows
        .iter()
        .filter(|r| r.source == TotalWorthSource::Payables)
        .map(|r| r.amount_paisa.rupees())
        .collect();
    assert_eq!(payables.iter().sum::<i64>(), 4_400);
    assert_eq!(payables.len(), 3);

    // The print holds every row of both sides, and the net in its totals row.
    let spec = dashboard::total_worth_spec(w.env.ctx.db()).unwrap();
    assert_eq!(spec.total_count, 3 + 4);
    let print = dashboard::total_worth_print_data(
        w.env.ctx.db(),
        &dashboard::TotalWorthPrintInput {
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(print.row_count, "7");
}

#[test]
fn total_worth_stays_the_books_with_lent_money_a_linked_khata_a_dead_person_and_gardens() {
    // After the worked day: money lent, a الگ کھاتہ linked to the shop, a buyer moved to the dead
    // amount, a seller paid above his payable, a buyer paying more than he owes.
    let w = full_day();
    let person: mandi_services::core::lent::LentPerson =
        mandi_services::lent::person_create_linked(
            &w.env.ctx,
            &w.owner,
            &typed(
                json!({ "name_ur": "نواز", "name_en": "Nawaz", "phone": null, "staff_id": null,
            "confirmed_rules_words": [], "person_type": "other", "link": null }),
            ),
        )
        .unwrap();
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "person_account_id": person.account.id,
            "amount": "5,000", "reason": "ضرورت", "return_by": null, "narration": null,
            "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    assert_total_worth_is_the_books(&w);
    let khata = mandi_services::khata::create_linked(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "name_ur": "خاندانی کھاتہ", "name_en": "Family Khata", "note": null,
            "confirmed_rules_words": [], "link": { "kind": "shop" } }),
        ),
    )
    .unwrap();
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "khata_account_id": khata.account.id,
            "amount": "10,000", "reason": "خاندان", "narration": null,
            "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    assert_total_worth_is_the_books(&w);
    // کریم is paid 12,000 against 10,160: he owes 1,840 beyond his advances.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "12,000")).unwrap();
    // اکرم pays 5,000 against 2,880: 2,120 stays as his credit.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "5,000")).unwrap();
    assert_total_worth_is_the_books(&w);
    mandi_services::dead::mark(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "account_id": w.arshad, "date": "10-10-2026", "reason": "died",
            "note": "" }),
        ),
    )
    .unwrap();
    assert_total_worth_is_the_books(&w);

    // Gardens: an open garden with money in, and a trip.
    let (garden, _) = garden_40b();
    assert_total_worth_is_the_books(&garden);
    let (trip, _, _, _) = trip_34_world();
    assert_total_worth_is_the_books(&trip);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Any run of entries after the worked day, linked الگ کھاتہ and ادھار رقم included: have less
    /// give stays the books' equity, and every named part stays its tile (money review point 11).
    #[test]
    fn total_worth_stays_the_books_after_any_run_of_entries(
        ops in proptest::collection::vec((0_u8..11, 1_i64..30_000), 1..12),
    ) {
        let w = full_day();
        let khata = |link: serde_json::Value, ur: &str, en: &str| {
            mandi_services::khata::create_linked(
                &w.env.ctx,
                &w.owner,
                &typed(json!({ "name_ur": ur, "name_en": en, "note": null,
                    "confirmed_rules_words": [], "link": link })),
            )
            .unwrap()
            .account
            .id
        };
        let shop_khata = khata(json!({ "kind": "shop" }), "خاندانی کھاتہ", "Family Khata");
        let partner_link = json!({ "kind": "partner", "partner_id": w.partners[0] });
        let his_khata = khata(partner_link, "گھر کھاتہ", "Home Khata");
        let person: mandi_services::core::lent::LentPerson =
            mandi_services::lent::person_create_linked(
                &w.env.ctx,
                &w.owner,
                &typed(json!({ "name_ur": "نواز", "name_en": "Nawaz", "phone": null,
                    "staff_id": null, "confirmed_rules_words": [], "person_type": "other",
                    "link": { "kind": "partner", "partner_id": w.partners[1] } })),
            )
            .unwrap();
        let person = person.account.id;
        let head = support::id_of(&w.env, "05010001");
        for (op, rupees_typed) in ops {
            let amount = rupees_typed.to_string();
            let money = |id: i64, key: &str| {
                let mut value = json!({ "date": "10-10-2026", "amount": amount, "narration": null,
                    "reason": "کام", "confirm_cash_below_zero": true });
                value[key] = json!(id);
                value
            };
            let ctx = &w.env.ctx;
            let owner = &w.owner;
            let outcome = match op {
                0 => vouchers::save_receipt(ctx, owner, &receipt(w.akram, &amount)).map(drop),
                1 => vouchers::save_receipt(ctx, owner, &receipt(w.arshad, &amount)).map(drop),
                2 => vouchers::save_receipt(ctx, owner, &receipt(w.karim, &amount)).map(drop),
                3 => vouchers::save_payment(ctx, owner, &payment(w.karim, &amount)).map(drop),
                4 => vouchers::save_payment(ctx, owner, &payment(w.rahim, &amount)).map(drop),
                5 => {
                    let mut value = money(w.karim, "seller_account_id");
                    value["expected_goods"] = json!(null);
                    vouchers::save_advance(ctx, owner, &typed(value)).map(drop)
                }
                6 => vouchers::save_khata_out(ctx, owner, &typed(money(shop_khata, "khata_account_id")))
                    .map(drop),
                7 => vouchers::save_khata_in(ctx, owner, &typed(money(his_khata, "khata_account_id")))
                    .map(drop),
                8 => {
                    let mut value = money(person, "person_account_id");
                    value["return_by"] = json!(null);
                    vouchers::save_money_lent(ctx, owner, &typed(value)).map(drop)
                }
                9 => {
                    let mut value = money(person, "person_account_id");
                    value["mode"] = json!("cash");
                    value["salary_head_id"] = json!(null);
                    vouchers::save_money_lent_return(ctx, owner, &typed(value)).map(drop)
                }
                _ => {
                    let mut value = money(head, "expense_head_id");
                    value["paid_to"] = json!(null);
                    vouchers::save_shop_expense(ctx, owner, &typed(value)).map(drop)
                }
            };
            // An entry the app refuses writes nothing; any other failure is a bug.
            if let Err(error) = outcome {
                prop_assert!(matches!(error, ServiceError::Rejected(_)), "{op}: {error:?}");
            }
        }
        assert_total_worth_is_the_books(&w);
    }
}

// ---- C: every cancel asks the code, the sign and a reason -----------------------------------------

type Guarded = Box<dyn Fn(&support::World, &str, CancelProof) -> Result<(), ServiceError>>;

/// Every guarded cancel, each aimed at a record that does not exist.
fn every_cancel() -> Vec<(&'static str, Guarded)> {
    macro_rules! guarded {
        ($name:literal, $request:ty, $call:path, $extra:tt) => {{
            let call: Guarded = Box::new(|w, reason, proof| {
                let mut value = json!($extra);
                value["reason"] = json!(reason);
                value["partner_code"] = json!(proof.partner_code);
                value["sign"] = json!(proof.sign);
                let request: $request = typed(value);
                $call(&w.env.ctx, &w.owner, &request).map(|_| ())
            });
            ($name, call)
        }};
    }
    vec![
        guarded!("bikri", BikriCancelRequest, cancel_guard::cancel_bikri, {"id": 999999, "version": 1}),
        guarded!("bikri write off", BikriWriteOffUndoRequest, cancel_guard::undo_bikri_write_off, {"move_id": 999999}),
        guarded!("voucher", VoucherCancelRequest, cancel_guard::cancel_voucher, {"id": 999999, "version": 1}),
        guarded!("fruit", FruitCancelRequest, cancel_guard::cancel_fruit, {"id": 999999, "version": 1}),
        guarded!("purchase", PurchaseCancelRequest, cancel_guard::cancel_purchase, {"id": 999999, "version": 1}),
        guarded!("purchase write off", PurchaseWriteOffUndoRequest, cancel_guard::undo_purchase_write_off, {"move_id": 999999}),
        guarded!("trip cost", TripEntryCancelRequest, cancel_guard::cancel_trip_cost, {"id": 999999, "version": 1}),
        guarded!("trip other sale", TripEntryCancelRequest, cancel_guard::cancel_trip_other_sale, {"id": 999999, "version": 1}),
        guarded!("garden entry", GardenEntryCancelRequest, cancel_guard::cancel_garden_entry, {"id": 999999, "version": 1}),
        guarded!("garden arrival", ArrivalCancelRequest, cancel_guard::cancel_garden_arrival, {"id": 999999, "version": 1}),
        guarded!("garden close", GardenReopenRequest, cancel_guard::undo_garden_close, {"id": 999999, "version": 1}),
        guarded!("arrival write off", ArrivalWriteOffUndoRequest, cancel_guard::undo_arrival_write_off, {"move_id": 999999}),
    ]
}

#[test]
fn every_cancel_refuses_without_or_with_a_wrong_code_sign_or_reason_and_changes_nothing() {
    let w = full_day();
    let cancels = every_cancel();
    assert_eq!(cancels.len(), 12);
    // No code set yet: nothing is cancelled.
    for (name, call) in &cancels {
        let error = call(&w, "غلط", proof(CODE, "adnan")).unwrap_err();
        assert_eq!(
            rejections(&error),
            vec![(ErrorCode::ErrPartnerCodeNotSet, Some("partner_code".into()))],
            "{name}"
        );
    }
    set_code(&w);
    let before = book_rows(&w);
    for (name, call) in &cancels {
        // All three missing, together.
        let error = call(&w, " ", proof("", "")).unwrap_err();
        assert_eq!(
            rejections(&error),
            vec![
                (ErrorCode::ErrReasonRequired, Some("reason".into())),
                (ErrorCode::ErrSignRequired, Some("sign".into())),
                (ErrorCode::ErrRequired, Some("partner_code".into())),
            ],
            "{name}"
        );
        let error = call(&w, "غلط", proof("12", "adnan")).unwrap_err();
        assert_eq!(
            rejections(&error),
            vec![(
                ErrorCode::ErrPartnerCodeInvalid,
                Some("partner_code".into())
            )],
            "{name}"
        );
        // A wrong sign and a wrong code, both shown.
        let error = call(&w, "غلط", proof("1111", "munshi")).unwrap_err();
        assert_eq!(
            rejections(&error),
            vec![
                (ErrorCode::ErrSignWrong, Some("sign".into())),
                (ErrorCode::ErrPartnerCodeWrong, Some("partner_code".into())),
            ],
            "{name}"
        );
        // The right proof gets through to the cancel itself, which finds nothing.
        let error = call(&w, "غلط", proof(CODE, "adnan")).unwrap_err();
        assert_eq!(rejections(&error)[0].0, ErrorCode::ErrNotFound, "{name}");
        // Giving the code never opens the partner section.
        assert!(partner_code::is_locked(&w.env.ctx).unwrap(), "{name}");
    }
    assert_eq!(book_rows(&w), before, "a refused cancel changed the books");
    // Each wrong try is in the audit log, without what was typed.
    assert_eq!(common::audit_count(&w.dir, "cancel_sign_wrong"), 12);
    assert_eq!(common::audit_count(&w.dir, "partner_code_failed"), 12);
    let leaked = scalar(
        &w,
        "SELECT COUNT(*) FROM audit_log WHERE
           COALESCE(old_json,'') || COALESCE(new_json,'') || COALESCE(reason,'') LIKE '%munshi%'
        OR COALESCE(old_json,'') || COALESCE(new_json,'') LIKE '%1111%'
        OR COALESCE(old_json,'') || COALESCE(new_json,'') LIKE '%4826%'",
    );
    assert_eq!(leaked, 0);
    common::assert_chain_ok(&w.dir);
}

#[test]
fn the_right_code_and_sign_cancel_a_receipt_and_wrong_codes_start_the_wait() {
    let w = full_day();
    set_code(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    let version = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version;
    let request = |code: &str| -> VoucherCancelRequest {
        typed(
            json!({ "id": saved.voucher_id, "version": version, "reason": "غلط رقم",
            "partner_code": code, "sign": "adnan" }),
        )
    };
    // Ten wrong codes start a wait of one minute (as FR-8.2), and the wait is audited.
    for _ in 0..9 {
        let error =
            cancel_guard::cancel_voucher(&w.env.ctx, &w.owner, &request("1111")).unwrap_err();
        assert_eq!(rejections(&error)[0].0, ErrorCode::ErrPartnerCodeWrong);
    }
    let error = cancel_guard::cancel_voucher(&w.env.ctx, &w.owner, &request("1111")).unwrap_err();
    assert_eq!(rejections(&error)[0].0, ErrorCode::ErrLockedWait);
    assert_eq!(common::audit_count(&w.dir, "lockout_started"), 1);
    // While it waits, even the right code is refused.
    let error = cancel_guard::cancel_voucher(&w.env.ctx, &w.owner, &request(CODE)).unwrap_err();
    assert_eq!(rejections(&error)[0].0, ErrorCode::ErrLockedWait);
    assert_eq!(
        vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().status,
        "saved"
    );
    w.env.mono.advance_seconds(61);
    w.env.clock.advance_seconds(61);
    let done = cancel_guard::cancel_voucher(&w.env.ctx, &w.owner, &request(CODE)).unwrap();
    assert_eq!(done.voucher_id, saved.voucher_id);
    let reason: String = common::raw(&w.dir)
        .query_row(
            "SELECT reason FROM audit_log WHERE action = 'cancel' AND entity = 'voucher'
             ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reason, "غلط رقم");
}

fn undo_input(id: i64) -> UndoInput {
    UndoInput {
        id,
        confirm_cash_below_zero: false,
    }
}

#[test]
fn the_undo_straight_after_a_save_needs_nothing_and_only_then() {
    // Straight after the save: no code is even set.
    let w = worked_day();
    let first = save(&w, &support::event1(&w));
    w.env.mono.advance_seconds(10);
    cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(first.bikri_id)).unwrap();

    // Too late: the same call later is refused and nothing changes.
    let w = worked_day();
    let late = save(&w, &support::event1(&w));
    w.env.mono.advance_seconds(16);
    let before = book_rows(&w);
    let error =
        cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(late.bikri_id)).unwrap_err();
    assert_eq!(
        rejections(&error),
        vec![(ErrorCode::ErrUndoExpired, Some("id".into()))]
    );
    assert_eq!(book_rows(&w), before);

    // Not in another session: after a lock or a sign-out the undo is gone.
    let w = worked_day();
    let other = save(&w, &support::event1(&w));
    auth::logout(&w.env.ctx).unwrap();
    auth::login(&w.env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    let error =
        cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(other.bikri_id)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);

    // A بکری nobody just saved (an id from elsewhere) is never undone.
    let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(12_345)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);
}

/// The cash is 0 after a بکری paid 1,000 on the spot and an expense of 1,000: its undo would take
/// cash below zero, so it asks first (PRD FR-6.7).
fn undo_that_asks_about_cash() -> (support::World, i64) {
    let w = support::world_with(0, true);
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "10");
    let mut line = support::sale("line-1", w.akram, "10", "100");
    line.paid = Some("1,000".into());
    lot.sales = vec![line];
    let saved = save(&w, &support::draft(vec![lot]));
    let head = support::id_of(&w.env, "05010001");
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "expense_head_id": head, "amount": "1,000",
            "paid_to": null, "narration": null, "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    (w, saved.bikri_id)
}

#[test]
fn the_confirmed_retry_of_an_undo_in_time_is_taken_for_two_minutes() {
    let confirmed = |id| UndoInput {
        id,
        confirm_cash_below_zero: true,
    };
    let asks = |w: &support::World, id| {
        let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(id)).unwrap_err();
        assert!(
            error.has_code(ErrorCode::ErrCashBelowZeroNotConfirmed),
            "{error:?}"
        );
    };
    // Asked at 10 s, answered at 130 s (120 s after the first request): taken.
    let (w, id) = undo_that_asks_about_cash();
    w.env.mono.advance_seconds(10);
    asks(&w, id);
    w.env.mono.advance_seconds(120);
    cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &confirmed(id)).unwrap();

    // One second later than that: too late.
    let (w, id) = undo_that_asks_about_cash();
    w.env.mono.advance_seconds(10);
    asks(&w, id);
    w.env.mono.advance_seconds(121);
    let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &confirmed(id)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);

    // A later request does not move the two minutes, and without the yes it is too late.
    let (w, id) = undo_that_asks_about_cash();
    asks(&w, id);
    w.env.mono.advance_seconds(12);
    asks(&w, id);
    w.env.mono.advance_seconds(10);
    let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &undo_input(id)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);
    w.env.mono.advance_seconds(99);
    let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &confirmed(id)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);

    // The first request after 15 s is refused, with or without the yes.
    let (w, id) = undo_that_asks_about_cash();
    w.env.mono.advance_seconds(16);
    let error = cancel_guard::undo_bikri(&w.env.ctx, &w.owner, &confirmed(id)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUndoExpired);
}

#[test]
fn the_edit_preview_of_a_partners_own_money_is_open_without_the_code() {
    // PRD FR-17.11 v1.61: his personal expense is open; every other edit preview stays hidden.
    let w = full_day();
    set_code(&w);
    let voucher = |kind: &str| -> (i64, i64) {
        common::raw(&w.dir)
            .query_row(
                "SELECT id, version FROM vouchers WHERE type = ?1",
                [kind],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap()
    };
    let review = |(id, version): (i64, i64), entry: VoucherEntry| {
        mandi_services::edit::voucher_review(
            &w.env.ctx,
            &VoucherEditInput {
                id,
                version,
                entry,
                reason: "غلط رقم".into(),
                sign: String::new(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap()
    };
    let personal = |amount: &str| {
        VoucherEntry::PersonalExpense(typed(json!({ "date": "10-10-2026",
            "partner_id": w.partners[0], "amount": amount, "reason": "گھر کا خرچ",
            "return_by": null, "taken_by": null, "narration": null,
            "confirm_cash_below_zero": false })))
    };
    let shop = |amount: &str| {
        let head = support::id_of(&w.env, "05010001");
        VoucherEntry::ShopExpense(typed(
            json!({ "date": "10-10-2026", "expense_head_id": head,
            "amount": amount, "paid_to": null, "narration": null,
            "confirm_cash_below_zero": false }),
        ))
    };

    let his = review(voucher("personal_expense"), personal("2,500"));
    assert!(his.errors.is_empty(), "{:?}", his.errors);
    assert!(!his.review.effect.partners_locked);
    let figures: Vec<(i64, i64, i64)> = his
        .review
        .effect
        .partners
        .iter()
        .map(|p| {
            (
                p.partner.id,
                p.before_paisa.rupees(),
                p.after_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(figures, vec![(w.partners[0], -2_000, -2_500)]);

    // A shop expense edit stays hidden.
    let other = review(voucher("shop_expense"), shop("3,100"));
    assert!(other.errors.is_empty(), "{:?}", other.errors);
    assert!(other.review.effect.partners_locked && other.review.effect.partners.is_empty());
    // A change of type to or from دکان خرچہ is refused, and stays hidden too.
    for (from, entry) in [
        ("shop_expense", personal("3,000")),
        ("personal_expense", shop("2,000")),
    ] {
        let changed = review(voucher(from), entry);
        assert!(!changed.errors.is_empty(), "{from}");
        assert!(
            changed.review.effect.partners_locked && changed.review.effect.partners.is_empty(),
            "{from}"
        );
    }
    assert!(partner_code::is_locked(&w.env.ctx).unwrap());
}

// ---- D: the audit of security events --------------------------------------------------------------

fn audit_rows(w: &support::World, filters: AuditFilters) -> Vec<AuditRow> {
    audit::list(
        w.env.ctx.db(),
        &AuditListInput {
            filters,
            offset: 0,
            limit: 1_000,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
    .rows
}

#[test]
fn the_security_filter_lists_wrong_code_sign_password_and_closed_day() {
    let w = full_day();
    set_code(&w);
    // A wrong password at sign-in, after a sign-out.
    auth::logout(&w.env.ctx).unwrap();
    auth::login(&w.env.ctx, "owner", &Secret::new("wrong-one")).unwrap_err();
    auth::login(&w.env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    // A wrong code and a wrong sign on a cancel.
    let request: VoucherCancelRequest = typed(json!({ "id": 1, "version": 1, "reason": "x",
        "partner_code": "1111", "sign": "munshi" }));
    cancel_guard::cancel_voucher(&w.env.ctx, &w.owner, &request).unwrap_err();
    // A save on a closed day.
    day::close(
        &w.env.ctx,
        &w.owner,
        &typed(json!({ "up_to": "10-10-2026" })),
    )
    .unwrap();
    let refused =
        vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10")).unwrap_err();
    service_audit::note_refused_save(&w.env.ctx, "vouchers_save_receipt", &refused);
    // A preview refused on a closed day is not a refused save.
    service_audit::note_refused_save(&w.env.ctx, "vouchers_edit_preview", &refused);

    let security = audit_rows(
        &w,
        AuditFilters {
            security_only: true,
            ..AuditFilters::default()
        },
    );
    let kinds: Vec<AuditEventKind> = security.iter().filter_map(|r| r.kind).collect();
    assert_eq!(kinds.len(), security.len(), "every security row has a kind");
    for kind in [
        AuditEventKind::WrongCode,
        AuditEventKind::WrongSign,
        AuditEventKind::WrongPassword,
        AuditEventKind::ClosedDay,
    ] {
        assert_eq!(kinds.iter().filter(|k| **k == kind).count(), 1, "{kind:?}");
    }
    let closed = audit_rows(
        &w,
        AuditFilters {
            kind: Some(AuditEventKind::ClosedDay),
            ..AuditFilters::default()
        },
    );
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].entity, "voucher");
    assert_eq!(closed[0].action, "closed_day_refused");
    assert!(
        closed[0]
            .new_json
            .as_deref()
            .unwrap()
            .contains("vouchers_save_receipt")
    );
    assert!(
        security
            .iter()
            .all(|r| !r.new_json.as_deref().unwrap_or("").contains("wrong-one"))
    );
}

// ---- F: the lists of several types ----------------------------------------------------------------

fn list(w: &support::World, value: serde_json::Value) -> Vec<(VoucherType, i64)> {
    let input: VouchersListInput = typed(value);
    vouchers::list(&w.env.ctx, &input)
        .unwrap()
        .rows
        .into_iter()
        .map(|r| (r.voucher_type, r.number))
        .collect()
}

#[test]
fn one_list_takes_lent_and_returned_and_the_journal_by_a_party_on_a_line() {
    let w = umar_sellers();
    let person: mandi_services::core::lent::LentPerson =
        mandi_services::lent::person_create_linked(
            &w.env.ctx,
            &w.owner,
            &typed(
                json!({ "name_ur": "نواز", "name_en": "Nawaz", "phone": null, "staff_id": null,
            "confirmed_rules_words": [], "person_type": "other", "link": null }),
            ),
        )
        .unwrap();
    let id = person.account.id;
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "person_account_id": id, "amount": "500",
            "reason": "ضرورت", "return_by": null, "narration": null,
            "confirm_cash_below_zero": true }),
        ),
    )
    .unwrap();
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "person_account_id": id, "amount": "200",
            "mode": "cash", "salary_head_id": null, "narration": null }),
        ),
    )
    .unwrap();
    let rows = list(
        &w,
        json!({ "voucher_type": null, "voucher_types": ["money_lent", "money_lent_return"],
            "from": "", "to": "", "account_id": id }),
    );
    assert_eq!(
        rows,
        vec![
            (VoucherType::MoneyLent, 1),
            (VoucherType::MoneyLentReturn, 1)
        ]
    );
    // The one جنرل entry has a line on کریم and one on اکرم, and none on ارشد.
    for (account, found) in [(w.karim, 1), (w.akram, 1), (w.arshad, 0)] {
        let rows = list(
            &w,
            json!({ "voucher_type": "general", "from": "", "to": "", "account_id": account }),
        );
        assert_eq!(rows.len(), found, "account {account}");
    }
}

// ---- I: most used first ---------------------------------------------------------------------------

#[test]
fn a_party_used_three_times_is_first_when_the_picker_opens() {
    let w = full_day();
    let touch = |id| {
        search::touch_account(
            &w.env.ctx,
            &w.owner,
            &AccountsTouchInput {
                scope: AccountScope::Buyer,
                control_id: None,
                id,
            },
        )
        .unwrap();
    };
    touch(w.akram);
    for _ in 0..3 {
        touch(w.arshad);
    }
    // Used last, but only twice now: the most used stays first.
    touch(w.akram);
    let input: SearchAccountsInput = typed(json!({ "scope": "buyer", "control_id": null,
        "text": "", "offset": 0, "limit": 10 }));
    let hits = search::search_accounts(&w.env.ctx, &w.owner, &input).unwrap();
    let first: Vec<i64> = hits.iter().take(2).map(|h| h.id).collect();
    assert_eq!(first, vec![w.arshad, w.akram]);
}

// ---- J: the bell panel ----------------------------------------------------------------------------

#[test]
fn the_bell_panel_has_three_groups_and_later_hides_until_the_next_bikri() {
    let w = full_day();
    let db = w.env.ctx.db();
    let panel = |hidden: &[String]| dashboard::attention_panel(db, off(), hidden).unwrap();
    let shown = panel(&[]);
    // Sellers: کریم 10,160, posted today. Buyers: ارشد 18,684, اکرم 2,880.
    assert_eq!(shown.sellers.rows.len(), 1);
    assert_eq!(shown.sellers.rows[0].party.account_id, w.karim);
    assert_eq!(shown.sellers.rows[0].amount_paisa, rupees(10_160));
    assert_eq!(shown.sellers.rows[0].days, Some(0));
    assert_eq!(shown.sellers.total_paisa, rupees(10_160));
    let buyers: Vec<(i64, i64)> = shown
        .buyers
        .rows
        .iter()
        .map(|r| (r.party.account_id, r.amount_paisa.rupees()))
        .collect();
    assert_eq!(buyers, vec![(w.arshad, 18_684), (w.akram, 2_880)]);
    assert_eq!(shown.buyers.total_paisa, rupees(21_564));
    let payables: Vec<i64> = shown
        .payables
        .iter()
        .map(|p| p.amount_paisa.rupees())
        .collect();
    assert_eq!(payables, vec![300, 4_000, 100]);
    assert_eq!(shown.hidden_count, 0);
    assert_eq!(
        shown.bell_count,
        i64::try_from(1 + 2 + 3 + shown.general.len()).unwrap()
    );

    // "بعد میں" on اکرم and on the freight: both go, the bell counts less.
    let akram_key = format!("buyer:{}", w.akram);
    assist::snooze(&w.env.ctx, &w.owner, &akram_key).unwrap();
    assist::snooze(&w.env.ctx, &w.owner, &shown.payables[1].item_key).unwrap();
    let hidden = assist::snoozed_keys(&w.env.ctx, &w.owner).unwrap();
    let after = panel(&hidden);
    assert_eq!(after.buyers.rows.len(), 1);
    assert_eq!(after.buyers.total_paisa, rupees(21_564), "the total stays");
    assert_eq!(after.payables.len(), 2);
    assert_eq!(after.hidden_count, 2);
    assert_eq!(after.bell_count, shown.bell_count - 2);

    // The next بکری brings them back.
    save(&w, &event2(&w));
    assert!(
        assist::snoozed_keys(&w.env.ctx, &w.owner)
            .unwrap()
            .is_empty()
    );

    let error = assist::snooze(&w.env.ctx, &w.owner, "buyer:abc").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrFormatInvalid);
}
