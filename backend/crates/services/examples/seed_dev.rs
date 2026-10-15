//! Makes a development database with the PRD 4.3 opening position, through the real services
//! (not raw SQL), and the masters and parties the worked day needs. Run by `pnpm seed`:
//!
//! ```bash
//! cargo run -p mandi-services --example seed_dev -- <path to mandi.db>
//! ```
//!
//! Steps (PRD 4.3, plan 10): the first-run wizard; the cities, truck stands, packing, items,
//! expense heads and parties of the worked day, each with an Urdu and an English name; on
//! 31-08-26 the opening balances of روکڑ and ابتدائی سرمایہ; on 01-09-26 a پیشگی voucher of 15,000
//! cash to رحیم; on 10-10-26 the days up to 09-10-26 are closed. The dates are fixed by a clock
//! that is set for each step, so the data is the same on any day. It then prints the E2E-19
//! numbers. Running it again on the same file changes nothing; `pnpm seed -- --reset` starts
//! again. The Owner signs in as `owner` with the password `mandi-dev-1`.
//!
//! The audit key: by default the seed reads the audit key from the real key store (Windows
//! Credential Manager, or the macOS Keychain on a Mac), like the app does. On a Mac a Keychain
//! prompt may appear: the user must enter the Mac password and choose Always Allow. Or run
//! `MANDI_SEED_TEST_KEY=1 pnpm seed` for a throwaway dev key that is never used for real data.
//! That key is a fixed one held in memory (any value of the variable turns it on, even `0`). A
//! dev database made with it cannot be verified by the real app on the same keychain: the audit
//! chain check reports a break. Use a database made with the real key for anything the app opens.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use chrono::{TimeZone, Utc};
use mandi_services::auth::{self, FirstRunRaw};
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{
    AddaInput, CityInput, ExpenseHeadInput, ItemInput, MasterInput, PackingInput,
};
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, DayClosePreviewInput, VoucherMode,
};
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::db::clock::ManualClock;
use mandi_services::db::{Db, DbOptions};
use mandi_services::settings::Firm;
use mandi_services::{
    Context, Secret, ServiceError, accounts, assist, day, ledger, masters, vouchers,
};

const OWNER_USER: &str = "owner";
const OWNER_PASSWORD: &str = "mandi-dev-1";

fn say(line: &str) {
    // Printing macros are denied in this workspace; this writes the same way.
    writeln!(std::io::stdout(), "{line}").ok();
}

/// Tries again with the words the spelling check asked about, as the Owner would confirm them
/// on screen (PRD FR-7.66). The words are in the word list or are plain names, so a seed run
/// confirms only what the check asks.
fn confirming<T>(
    mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>,
) -> Result<T, ServiceError> {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..6 {
        match attempt(&words) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
    Err(ServiceError::reject(ErrorCode::ErrInternal))
}

fn set_day(clock: &ManualClock, year: i32, month: u32, day: u32) {
    if let Some(time) = Utc.with_ymd_and_hms(year, month, day, 9, 0, 0).single() {
        clock.set(time);
    }
}

fn run(path: &Path) -> Result<(), ServiceError> {
    let backups = path
        .parent()
        .map_or_else(|| PathBuf::from("backups"), |dir| dir.join("backups"));
    let clock = Arc::new(ManualClock::new(
        Utc.with_ymd_and_hms(2026, 8, 31, 9, 0, 0)
            .single()
            .unwrap_or_default(),
    ));
    let mut options = DbOptions::new(backups);
    options.clock = clock.clone();
    // The audit key lives in the Windows Credential Manager (or the macOS keychain), like the
    // app's own. Any value of `MANDI_SEED_TEST_KEY` (even `0`) uses a fixed key in memory
    // instead, so a run needs no Keychain prompt. It is a throwaway dev key, never for real
    // data: the real app on the same keychain cannot verify the audit chain of a database made
    // that way.
    if std::env::var_os("MANDI_SEED_TEST_KEY").is_some() {
        options.key_store = Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32])));
    }
    let db = Db::open_checked(path, options)?;
    let ctx = Context::new(Arc::new(db));
    say(&format!("seed: {}", path.display()));

    let first_time = auth::status(&ctx)?.first_run;
    if first_time {
        // The wizard of PRD 8, with the firm of the PRD header.
        let code = auth::first_run(
            &ctx,
            &FirstRunRaw {
                firm: Firm {
                    name_ur: "ماہر عدنان اینڈ برادرز".to_owned(),
                    name_en: "Maher Adnan and Brothers".to_owned(),
                    mandi_ur: "نئی سبزی منڈی کوٹ رادھا کشن".to_owned(),
                    mandi_en: "New Sabzi Mandi Kot Radha Kishan".to_owned(),
                    address: "Kot Radha Kishan".to_owned(),
                    phone: "0300-1234567".to_owned(),
                },
                owner_username: OWNER_USER.to_owned(),
                owner_display_name: "Maher Adnan".to_owned(),
                password: Secret::new(OWNER_PASSWORD),
                password_repeat: Secret::new(OWNER_PASSWORD),
            },
        )?;
        say(&format!(
            "seed: owner `{OWNER_USER}`, password `{OWNER_PASSWORD}`, recovery code {code}"
        ));
    } else {
        say("seed: already set up; use `pnpm seed -- --reset` to start again");
    }
    auth::login(&ctx, OWNER_USER, &Secret::new(OWNER_PASSWORD))
        .map_err(|_| ServiceError::reject(ErrorCode::ErrWrongPassword))?;
    let owner = auth::require(&ctx)?;
    if !first_time {
        report(&ctx)?;
        return Ok(());
    }

    // The masters and parties of the worked day (PRD 4.10), on 31-08-26.
    set_day(&clock, 2026, 8, 31);
    let city = |ur: &str, en: &str| {
        confirming(|words| {
            masters::save(
                &ctx,
                &owner,
                &MasterInput::City(CityInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: ur.into(),
                    name_en: en.into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        })
        .map(|row| row.id())
    };
    let swat = city("سوات", "Swat")?;
    let lahore = city("لاہور", "Lahore")?;
    for (ur, en, city_id) in [
        ("سوات اڈا", "Swat Adda", swat),
        ("لاہور اڈا", "Lahore Adda", lahore),
    ] {
        confirming(|words| {
            masters::save(
                &ctx,
                &owner,
                &MasterInput::Adda(AddaInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: ur.into(),
                    name_en: en.into(),
                    city_id: Some(city_id),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        })?;
    }
    confirming(|words| {
        masters::save(
            &ctx,
            &owner,
            &MasterInput::Packing(PackingInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: "پیٹی".into(),
                name_en: "Box".into(),
                approx_kg: String::new(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    })?;
    for (ur, en, group) in [
        ("کیلا", "Banana", "fruit"),
        ("سیب", "Apple", "fruit"),
        ("مولی", "Radish", "vegetable"),
    ] {
        confirming(|words| {
            masters::save(
                &ctx,
                &owner,
                &MasterInput::Item(ItemInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: ur.into(),
                    name_en: en.into(),
                    item_group: group.into(),
                    default_packing_id: None,
                    sold_by: "unit".into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        })?;
    }
    for (ur, en) in [("بجلی", "Electricity"), ("تنخواہ", "Salary")] {
        confirming(|words| {
            masters::save(
                &ctx,
                &owner,
                &MasterInput::ExpenseHead(ExpenseHeadInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: ur.into(),
                    name_en: en.into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        })?;
    }
    let party = |kind: AccountType, ur: &str, en: &str| {
        confirming(|words| {
            accounts::save(
                &ctx,
                &owner,
                &AccountInput {
                    id: None,
                    version: None,
                    account_type: kind,
                    control_id: None,
                    code: String::new(),
                    name_ur: ur.into(),
                    name_en: en.into(),
                    confirmed_rules_words: words.to_vec(),
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
                },
            )
        })
        .map(|row| row.id)
    };
    let rahim = party(AccountType::Seller, "رحیم", "Rahim")?;
    party(AccountType::Seller, "کریم", "Karim")?;
    party(AccountType::Buyer, "اکرم ٹول ٹیکس", "Akram Toll Tax")?;
    party(AccountType::Buyer, "ارشد ملتانی", "Arshad Multani")?;

    // Step 1 of PRD 4.3, "today" 31-08-26: the openings on the account forms.
    let cash = code_id(&ctx, "01010001")?;
    let capital = code_id(&ctx, "06010001")?;
    let line = |id: i64, dr: &str, cr: &str| -> Result<OpeningLineInput, ServiceError> {
        Ok(OpeningLineInput {
            account_id: id,
            version: accounts::get(&ctx, id)?.version,
            opening_date: "31-08-2026".into(),
            opening_dr: dr.into(),
            opening_cr: cr.into(),
            opening_is_advance: false,
        })
    };
    accounts::openings_save(
        &ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(cash, "65,000", "")?, line(capital, "", "65,000")?],
        },
    )?;

    // Step 2, "today" 01-09-26: پیشگی پ-1 to رحیم, 15,000 cash.
    set_day(&clock, 2026, 9, 1);
    vouchers::save_advance(
        &ctx,
        &owner,
        &AdvanceInput {
            date: "01-09-2026".into(),
            seller_account_id: rahim,
            amount: "15,000".into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )?;

    // Step 3, "today" 10-10-26: close the days up to and including 09-10-26.
    set_day(&clock, 2026, 10, 10);
    let preview = day::close_preview(
        &ctx,
        &DayClosePreviewInput {
            up_to: "09-10-2026".into(),
        },
    )?;
    say(&format!(
        "seed: closing {} days, {} to {}",
        preview.days, preview.from_date, preview.up_to
    ));
    day::close(
        &ctx,
        &owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )?;
    report(&ctx)
}

fn code_id(ctx: &Context, code: &str) -> Result<i64, ServiceError> {
    ctx.db()
        .read(|conn| {
            mandi_services::db::accounts::id_by_code(conn, code).map_err(ServiceError::from)
        })?
        .ok_or_else(|| ServiceError::reject(ErrorCode::ErrNotFound))
}

/// Prints the numbers of E2E-19 and checks them against PRD 4.3: a seed that does not give
/// them fails loudly.
fn report(ctx: &Context) -> Result<(), ServiceError> {
    let trial = ledger::trial_balance(ctx, None)?;
    let cash = ledger::account_balance(ctx, code_id(ctx, "01010001")?)?;
    let difference = ledger::account_balance(ctx, code_id(ctx, "06019999")?)?;
    let rahim = code_id(ctx, "030110001")?;
    let advances = assist::advances(ctx, rahim)?;
    let status = day::status(ctx)?;
    let rupees = |p: mandi_services::core::money::Paisa| p.rupees();
    say(&format!(
        "E2E-19: trial balance {} = {}",
        rupees(trial.total_dr),
        rupees(trial.total_cr)
    ));
    say(&format!(
        "E2E-19: ابتدائی فرق {} ({})",
        rupees(difference.amount_paisa),
        match difference.side {
            BalanceSide::Zero => "zero",
            BalanceSide::Dr => "debit",
            BalanceSide::Cr => "credit",
        }
    ));
    say(&format!(
        "E2E-19: advances open {}",
        rupees(advances.open_paisa)
    ));
    say(&format!("E2E-19: cash {}", rupees(cash.amount_paisa)));
    let closed = status.closed_up_to.unwrap_or_default();
    say(&format!("E2E-19: closed up to {closed}"));
    let right = rupees(trial.total_dr) == 65_000
        && rupees(trial.total_cr) == 65_000
        && difference.side == BalanceSide::Zero
        && rupees(advances.open_paisa) == 15_000
        && cash.side == BalanceSide::Dr
        && rupees(cash.amount_paisa) == 50_000
        && closed == "2026-10-09";
    if right {
        Ok(())
    } else {
        Err(ServiceError::Data(
            "the seeded numbers are not those of PRD 4.3 and E2E-19".to_owned(),
        ))
    }
}

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        writeln!(std::io::stderr(), "usage: seed_dev <path to mandi.db>").ok();
        return ExitCode::from(2);
    };
    match run(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            writeln!(std::io::stderr(), "seed failed: {error:?}").ok();
            ExitCode::FAILURE
        }
    }
}
