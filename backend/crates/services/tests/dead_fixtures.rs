//! Real answers of the plan 40 commands (the dead amount, PRD FR-10.10, E2E-44) as JSON files for
//! the frontend: `masters_dead_preview`, `masters_dead_mark`, `masters_dead_undo`,
//! `reports_dead_amount` (R38), the dashboard tiles after the move and after the recovery, the
//! party card, the receipt picker with `include_dead`, and the error answers of the move.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used) and `<name>.json` is the answer the real
//! service gives. The test compares the answers with the files on disk and FAILS if any differ, so a
//! fixture can never go stale. After a deliberate change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test dead_fixtures`, read the diff and
//! tell `frontend`.
//!
//! The story is E2E-44: Events 1 to 5 of the worked day, ارشد dies on 11-10-26 (owing 18,684), his
//! son pays 4,000 on 12-10-26.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_services::core::accounts::{AccountScope, SearchAccountsInput};
use mandi_services::core::bikri::{
    BikriSaveInput, LineChargeMethod, PaymentType, PukhtaInput, PukhtaPeopleInput,
};
use mandi_services::core::dead::{DeadAmountInput, DeadMarkInput, DeadPreviewInput, DeadUndoInput};
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::{assist, bikri, dead, search, vouchers};
use serde::Serialize;
use serde_json::{Value, json};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(folder()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// An error as the wire shows it: the code, the field and the values.
fn wire_error(error: &mandi_services::ServiceError) -> Value {
    match error {
        mandi_services::ServiceError::Rejected(items) => json!({
            "ok": false,
            "errors": items.iter().map(|i| json!({
                "code": i.code,
                "field": i.field,
                "values": i.values,
            })).collect::<Vec<_>>(),
        }),
        other => panic!("{other:?}"),
    }
}

fn tiles(w: &support::World) -> Value {
    let page = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap();
    // Only the dues figures of the tiles, so the file stays small and about this plan.
    let t = &page.tiles;
    json!({
        "buyers_dues_paisa": t.buyers_dues_paisa,
        "sellers_dues_paisa": t.sellers_dues_paisa,
        "dead_amount_paisa": t.dead_amount_paisa,
        "dead_count": t.dead_count,
        "total_dues_paisa": t.total_dues_paisa,
        "sellers_payable_paisa": t.sellers_payable_paisa,
        "advances_open_paisa": t.advances_open_paisa,
        "lent_open_paisa": t.lent_open_paisa,
    })
}

#[test]
fn the_real_answers_of_the_plan_40_commands_are_the_fixtures() {
    let w = reports_world::full_day();
    let ctx = &w.env.ctx;
    pair("dead_tiles_end_of_day_1", &json!({}), &tiles(&w));

    // 11-10-26: the preview, the move and the card.
    set_today(&w.env, 2026, 10, 11);
    let preview_request = json!({ "account_id": w.arshad, "pukhta_person_key": null });
    let preview = dead::preview(ctx, &typed::<DeadPreviewInput>(&preview_request)).unwrap();
    assert_eq!(preview.owed_paisa.rupees(), 18_684);
    pair("masters_dead_preview_arshad", &preview_request, &preview);

    let nothing_owed = json!({ "account_id": w.rahim, "pukhta_person_key": null });
    pair(
        "masters_dead_preview_nothing_owed",
        &nothing_owed,
        &dead::preview(ctx, &typed::<DeadPreviewInput>(&nothing_owed)).unwrap(),
    );

    // The refusals of the move, as the wire shows them.
    let refuse = |name: &str, request: Value| {
        let error = dead::mark(ctx, &w.owner, &typed::<DeadMarkInput>(&request)).unwrap_err();
        pair(name, &request, &wire_error(&error));
    };
    refuse(
        "masters_dead_mark_nothing_owed",
        json!({ "account_id": w.rahim, "pukhta_person_key": null, "entry_date": "11-10-2026",
                "reason": "died", "note": "" }),
    );
    refuse(
        "masters_dead_mark_note_required",
        json!({ "account_id": w.arshad, "pukhta_person_key": null, "entry_date": "11-10-2026",
                "reason": "other", "note": "" }),
    );

    let mark_request = json!({ "account_id": w.arshad, "pukhta_person_key": null,
        "entry_date": "11-10-2026", "reason": "died", "note": "فوت ہو گئے" });
    let marked = dead::mark(ctx, &w.owner, &typed::<DeadMarkInput>(&mark_request)).unwrap();
    pair("masters_dead_mark_arshad", &mark_request, &marked);
    pair("dead_tiles_after_the_move", &json!({}), &tiles(&w));
    pair(
        "dead_party_card_arshad",
        &json!({ "account_id": w.arshad }),
        &assist::party_card(ctx, w.arshad).unwrap(),
    );
    let picker = json!({ "scope": "buyer", "control_id": null, "text": "arshad", "limit": 10,
        "offset": 0, "include_dead": true });
    pair(
        "search_accounts_include_dead",
        &picker,
        &search::search_accounts(ctx, &w.owner, &typed::<SearchAccountsInput>(&picker)).unwrap(),
    );
    let _ = AccountScope::Buyer;
    let list = json!({ "filter": "owed", "search": "", "offset": 0, "limit": 100 });
    pair(
        "reports_dead_amount_after_the_move",
        &list,
        &mandi_reports::dead::page(ctx.db(), &typed::<DeadAmountInput>(&list)).unwrap(),
    );

    // 12-10-26: his son pays 4,000; and a recovery over what he owes is refused.
    set_today(&w.env, 2026, 10, 12);
    let over = json!({ "date": "12-10-2026", "account_id": w.arshad, "person_key": null,
        "amount": "18,685", "collector_staff_id": null, "narration": "" });
    let error = vouchers::save_receipt(ctx, &w.owner, &typed::<ReceiptInput>(&over)).unwrap_err();
    pair(
        "vouchers_save_receipt_dead_over_the_amount",
        &over,
        &wire_error(&error),
    );
    let receipt = json!({ "date": "12-10-2026", "account_id": w.arshad, "person_key": null,
        "amount": "4,000", "collector_staff_id": null, "narration": "" });
    vouchers::save_receipt(ctx, &w.owner, &typed::<ReceiptInput>(&receipt)).unwrap();
    pair("dead_tiles_after_the_recovery", &json!({}), &tiles(&w));
    pair(
        "reports_dead_amount_after_the_recovery",
        &list,
        &mandi_reports::dead::page(ctx.db(), &typed::<DeadAmountInput>(&list)).unwrap(),
    );

    // Undone: the answer, and the "all" list with the move kept in the history.
    let undo = json!({ "mark_id": marked.mark_id, "reason": "غلط اندراج" });
    pair(
        "masters_dead_undo_arshad",
        &undo,
        &dead::undo(ctx, &w.owner, &typed::<DeadUndoInput>(&undo)).unwrap(),
    );
    let all = json!({ "filter": "all", "search": "", "offset": 0, "limit": 100 });
    pair(
        "reports_dead_amount_all_after_the_undo",
        &all,
        &mandi_reports::dead::page(ctx.db(), &typed::<DeadAmountInput>(&all)).unwrap(),
    );

    // A one-time buyer (ناصر) owes 2,000 and dies: the receipt screen finds him with the flag.
    let mut lot = support::bare_lot(&w, "lot-n", w.rahim, w.banana, "2");
    let mut sale = support::sale("s-n", w.pukhta_buyer, "2", "1000");
    sale.pukhta = Some(PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0300-1111111".into(),
    });
    sale.payment_type = Some(PaymentType::Credit);
    sale.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    sale.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    sale.laga = support::typed(LineChargeMethod::PerUnit, "0");
    lot.sales = vec![sale];
    bikri::save(
        ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let key = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    let person_mark = json!({ "account_id": w.pukhta_buyer, "pukhta_person_key": key,
        "entry_date": "12-10-2026", "reason": "left", "note": "" });
    dead::mark(ctx, &w.owner, &typed::<DeadMarkInput>(&person_mark)).unwrap();
    let people = json!({ "text": "nasir", "side": "buyer", "include_dead": true });
    pair(
        "search_pukhta_people_include_dead",
        &people,
        &bikri::search_people(ctx, &typed::<PukhtaPeopleInput>(&people)).unwrap(),
    );
    let hidden = json!({ "text": "nasir", "side": "buyer" });
    pair(
        "search_pukhta_people_dead_hidden",
        &hidden,
        &bikri::search_people(ctx, &typed::<PukhtaPeopleInput>(&hidden)).unwrap(),
    );
}
