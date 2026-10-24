//! Real answers of the plan 16 commands, as JSON files for the frontend (plan 16, T2).
//!
//! The same pattern as plan 13 (`bikri_save/fixtures.rs`): `<name>_request.json` is exactly the
//! `input` the command takes (it is read back into the real input type before it is used) and
//! `<name>.json` is the real answer on the PRD worked day. The test compares the answer with the
//! file on disk and FAILS if they differ, so a fixture can never go stale. After a deliberate
//! change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test bikri_edit fixtures`, read the diff
//! and tell `frontend`. The frontend copies the files to `frontend/tests/fixtures/` unchanged.

use std::path::{Path, PathBuf};

use mandi_services::bikri;
use mandi_services::core::bikri::{
    CancelInput, CancelPreviewInput, EditOpenInput, EditPreviewInput, EditSaveInput,
    LaterSaleInput, LaterSaleSaveInput, StockListInput, UndoInput, WriteOffInput,
    WriteOffUndoInput,
};
use mandi_services::core::error::ErrorCode;
use serde::Serialize;
use serde_json::{Value, json};

use crate::helpers::*;
use crate::support::*;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
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

/// One pair of files: the request and the real answer.
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

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// The JSON of one sale row as the screen sends it.
fn row(key: &str, buyer: i64, qty: &str, rate: &str) -> Value {
    json!({ "key": key, "sale_id": null, "buyer_account_id": buyer, "pukhta": null, "qty": qty,
        "weight": null, "seller_rate": rate, "rate_differs": false, "buyer_rate": null,
        "payment_type": null, "seller_comm": null, "buyer_comm": null, "laga": null, "note": null })
}

#[test]
fn the_real_answers_of_the_plan_16_commands_are_the_fixtures() {
    let w = world();

    // The worked day: Event 1 with 45 of the 50 sold (E2E-16), and Event 2.
    let mut part = event1(&w);
    part.lots[0].sales[1].qty = "25".into();
    let one = save_ok(&w, &part);
    let mut cash_sale = line_for_event2(&w);
    let two = save_ok(&w, &cash_sale.take().unwrap());
    assert_eq!((one.number, two.number), (1, 2));
    let lot_id = bikri::get(&w.env.ctx, one.bikri_id).unwrap().lots[0].id;

    // bikri_stock_list: lot 1 has 5 left; the cash sale of Event 2 sold all its goods.
    let request = json!({ "offset": 0, "limit": 0 });
    let list = bikri::stock_list(&w.env.ctx, &typed::<StockListInput>(&request)).unwrap();
    assert_eq!((list.rows.len(), list.balance_total), (1, 5));
    pair("bikri_stock_list", &request, &list);

    // bikri_write_off and bikri_write_off_undo: 3 spoiled, then put back.
    let request = json!({ "lot_id": lot_id, "qty": "3", "reason": "خراب" });
    let done = bikri::write_off(&w.env.ctx, &w.owner, &typed::<WriteOffInput>(&request)).unwrap();
    assert_eq!((done.qty_written_off, done.balance), (3, 2));
    pair("bikri_write_off_lot1", &request, &done);
    let request = json!({ "move_id": done.move_id, "reason": "غلطی" });
    let undone =
        bikri::write_off_undo(&w.env.ctx, &w.owner, &typed::<WriteOffUndoInput>(&request)).unwrap();
    assert_eq!((undone.qty_written_off, undone.balance), (0, 5));
    pair("bikri_write_off_undo_lot1", &request, &undone);

    // bikri_later_sale_preview and bikri_later_sale_save: the last 5 to ارشد at 560 (E2E-16b).
    let request = json!({ "lot_id": lot_id, "date": "10-10-2026",
        "sales": [row("later-1", w.arshad, "5", "560")] });
    let preview =
        bikri::later_sale_preview(&w.env.ctx, &typed::<LaterSaleInput>(&request)).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert_eq!(preview.summary.seller_credit.rupees(), 2_632);
    pair("bikri_later_sale_preview_e2e16b", &request, &preview);
    let mut save_request = request.clone();
    save_request["confirm_cash_below_zero"] = json!(false);
    let later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &typed::<LaterSaleSaveInput>(&save_request),
    )
    .unwrap();
    assert_eq!(later.preview, preview);
    pair("bikri_later_sale_save_e2e16b", &save_request, &later);

    // bikri_open_for_edit, bikri_edit_preview, bikri_edit_save: اکرم's rate 600 → 610 (E2E-13).
    let request = json!({ "bikri_id": one.bikri_id, "entry_id": one.entry_id });
    let opened = bikri::open_for_edit(&w.env.ctx, &typed::<EditOpenInput>(&request)).unwrap();
    assert!(opened.editable);
    pair("bikri_open_for_edit_event1", &request, &opened);
    let mut draft = serde_json::to_value(&opened.draft).unwrap();
    // Two refused edits: ERR_EDIT_NOT_ALLOWED with a fixed `rule` name at the field concerned.
    // The day of a posting cannot change (rule `day_changed`).
    let mut moved = draft.clone();
    moved["header"]["entry_date"] = json!("09-10-2026");
    let request = json!({ "entry_id": opened.entry_id, "draft": moved });
    let refused = bikri::edit_preview(&w.env.ctx, &typed::<EditPreviewInput>(&request)).unwrap();
    assert!(crate::tools::flagged(
        &refused.errors,
        ErrorCode::ErrEditNotAllowed,
        "header.entry_date"
    ));
    pair("bikri_edit_preview_day_changed", &request, &refused);
    // The lot has a later sale, so its seller cannot change (rule `lot_seller_locked`).
    let mut other_seller = draft.clone();
    other_seller["lots"][0]["seller_account_id"] = json!(w.karim);
    let request = json!({ "entry_id": opened.entry_id, "draft": other_seller });
    let refused = bikri::edit_preview(&w.env.ctx, &typed::<EditPreviewInput>(&request)).unwrap();
    assert!(crate::tools::flagged(
        &refused.errors,
        ErrorCode::ErrEditNotAllowed,
        "lots[0].seller_account_id"
    ));
    pair("bikri_edit_preview_lot_seller_locked", &request, &refused);
    draft["lots"][0]["sales"][0]["seller_rate"] = json!("610");
    let request = json!({ "entry_id": opened.entry_id, "draft": draft });
    let edit_preview =
        bikri::edit_preview(&w.env.ctx, &typed::<EditPreviewInput>(&request)).unwrap();
    assert!(edit_preview.errors.is_empty(), "{:?}", edit_preview.errors);
    pair("bikri_edit_preview_akram_610", &request, &edit_preview);
    let request = json!({ "entry_id": opened.entry_id, "version": opened.version, "draft": draft,
        "reason": "ریٹ کی درستی", "confirm_cash_below_zero": false });
    let edited = bikri::edit_save(&w.env.ctx, &w.owner, &typed::<EditSaveInput>(&request)).unwrap();
    assert_eq!(edited.preview, edit_preview);
    pair("bikri_edit_save_akram_610", &request, &edited);

    // bikri_cancel_preview and bikri_cancel: Event 2.
    let request = json!({ "id": two.bikri_id });
    let cancel_preview =
        bikri::cancel_preview(&w.env.ctx, &typed::<CancelPreviewInput>(&request)).unwrap();
    assert!(cancel_preview.can_cancel);
    pair("bikri_cancel_preview_event2", &request, &cancel_preview);
    let request = json!({ "id": two.bikri_id, "version": cancel_preview.version,
        "reason": "غلط اندراج", "confirm_cash_below_zero": false });
    let cancelled = bikri::cancel(&w.env.ctx, &w.owner, &typed::<CancelInput>(&request)).unwrap();
    pair("bikri_cancel_event2", &request, &cancelled);

    // bikri_undo: a new بکری 3 is undone straight after its save.
    let three = save_ok(&w, &event1(&w));
    let request = json!({ "id": three.bikri_id, "confirm_cash_below_zero": false });
    let undo = bikri::undo(&w.env.ctx, &w.owner, &typed::<UndoInput>(&request)).unwrap();
    assert_eq!(undo.number, 3);
    pair("bikri_undo_event1", &request, &undo);
}

/// Event 2 of PRD 4.5 as a draft: کریم's 10 سیب sold to the one-time buyer شاہد for cash.
fn line_for_event2(w: &World) -> Option<mandi_services::core::bikri::BikriDraft> {
    let mut line = sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = person("شاہد", "Shahid", "0300-0000000");
    let mut l = lot(w, "lot-1", w.karim, w.apple, "10");
    l.freight = Some("1,000".into());
    l.cash_paid = None;
    l.sales = vec![line];
    let mut d = draft(vec![l]);
    d.header.truck_no = Some("LHR-0077".into());
    Some(d)
}
