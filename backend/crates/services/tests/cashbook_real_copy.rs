//! The روکڑ بک answer for a COPY of the Owner's real database (plan 96, T1).
//!
//! It is `#[ignore]`: a plain `cargo test` never runs it. It needs `MANDI_REAL_DB`, the path of a
//! COPY of `mandi.db` (never the file the app has open). The copy is copied once more into a temp
//! folder, so the file named is not even opened. The read does not need the chain key: a read of
//! the cash book never checks a hash chain, so the test uses the fixed test key and the Keychain is
//! not touched.
//!
//! It calls the same service function as the command `cashbook_get` with the input the روکڑ screen
//! sends for its first view: the 30 days that end today, offset 0, rows per page 25 (the default),
//! no sort, `group_by_month` true. The answer is written in the serde shape the command returns.
//!
//! Run (from `backend/`):
//! `MANDI_REAL_DB=<copy> MANDI_REAL_TODAY=2026-11-24 cargo test -p mandi-services --test cashbook_real_copy -- --ignored --nocapture`
//! Output: `MANDI_REAL_OUT` or `frontend/tests/fixtures/cashbook_book_umar_real.json`.
#![cfg(test)]

mod common;

use std::path::PathBuf;

use mandi_services::cashbook;
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::windows::{WindowSource, WindowsBoundsInput};
use mandi_services::testing::TestEnv;
use mandi_services::windows;

fn today() -> (i32, u32, u32) {
    let text = std::env::var("MANDI_REAL_TODAY").unwrap_or_else(|_| "2026-11-24".to_owned());
    let mut parts = text.split('-').map(|p| p.parse::<u32>().unwrap());
    let y = parts.next().unwrap();
    let m = parts.next().unwrap();
    let d = parts.next().unwrap();
    (i32::try_from(y).unwrap(), m, d)
}

#[test]
#[ignore = "needs MANDI_REAL_DB, a COPY of the Owner's mandi.db"]
fn cashbook_book_for_a_copy_of_the_real_database() {
    let Ok(source) = std::env::var("MANDI_REAL_DB") else {
        eprintln!("skipped: MANDI_REAL_DB (a COPY of mandi.db) is not set");
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("data").join("mandi.db");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::copy(&source, &path).expect("copy the database");
    let env = TestEnv::new(root.path()).expect("open the copy");
    let (y, m, d) = today();
    common::set_today(&env, y, m, d);

    // The window of the screen: DAYS_WINDOW = 30 days ending today (frontend `daysWindowOf`).
    let end = chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap();
    let start = end - chrono::Duration::days(29);
    let input = CashbookInput {
        from: Some(start.format("%d-%m-%Y").to_string()),
        to: Some(end.format("%d-%m-%Y").to_string()),
        offset: 0,
        limit: 25,
        sort: None,
        group_by_month: true,
        show_reversed: false,
        kinds: None,
    };
    eprintln!("input: {input:?}");
    let answer = cashbook::get(&env.ctx, &input).expect("cashbook::get");
    let json = serde_json::to_string_pretty(&answer).unwrap() + "\n";

    let out = std::env::var("MANDI_REAL_OUT").map_or_else(
        |_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../frontend/tests/fixtures/cashbook_book_umar_real.json")
        },
        PathBuf::from,
    );
    std::fs::write(&out, &json).expect("write the fixture");
    eprintln!("wrote {} ({} rows)", out.display(), answer.rows.len());

    // What the window of the screen reads first (`windows_bounds`, source cashbook): written beside it.
    let bounds = windows::bounds(
        &env.ctx,
        &WindowsBoundsInput {
            source: WindowSource::Cashbook,
        },
    )
    .expect("windows::bounds");
    let bounds_out = out.with_file_name("windows_bounds_cashbook_umar_real.json");
    std::fs::write(
        &bounds_out,
        serde_json::to_string_pretty(&bounds).unwrap() + "\n",
    )
    .expect("write the bounds fixture");
    eprintln!("bounds: {bounds:?}");
}
