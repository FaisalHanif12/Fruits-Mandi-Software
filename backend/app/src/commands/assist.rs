//! Read-only help for the entry screens (Tech Stack 4.7, PRD 7.6). Nothing here writes
//! (Code Standards B29).

use mandi_services::core::accounts::{PartyCard, PartyCardInput};
use mandi_services::core::bikri::{
    LastRateInput, RateHint, SellerMemory, SellerMemoryInput, TruckMemory, TruckMemoryInput,
};
use mandi_services::core::vouchers::{
    AdvancesInfo, AssistAdvancesInput, CollectionList, CollectionListInput, OpenBills,
    OpenBillsInput, OpenPayables, OpenPayablesInput,
};
use mandi_services::{assist, auth, vouchers};

use super::run;
use crate::error::UiError;

/// The card next to an account field: names, father or firm, city, phone, balance with its
/// side, last entry date, a buyer's credit limit and use, a seller's open advance, and warnings
/// (PRD FR-7.38). Balances are read from the journal. Errors: `ERR_NOT_FOUND` on `account_id`.
#[tauri::command]
pub async fn assist_party_card<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartyCardInput,
) -> Result<PartyCard, UiError> {
    run(app, "assist_party_card", move |ctx| {
        auth::require(ctx)?;
        assist::party_card(ctx, input.account_id)
    })
    .await
}

/// A seller's open advances and his last پیشگی voucher, for the card in the پیشگی screen (PRD
/// FR-7.60). Read only. Errors: `ERR_NOT_FOUND` on `seller_id`, `ERR_NOT_A_SELLER`.
#[tauri::command]
pub async fn assist_advances<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistAdvancesInput,
) -> Result<AdvancesInfo, UiError> {
    run(app, "assist_advances", move |ctx| {
        auth::require(ctx)?;
        assist::advances(ctx, input.seller_id)
    })
    .await
}

/// What the app remembers about a truck number: its last بکری (to fill اڈا, زمیندار and driver),
/// a بکری of it on the posting day (the same-truck notice), and truck numbers that start with the
/// typed text (PRD FR-7.49, FR-7.50). Read only. Errors: the date codes on `date`.
#[tauri::command]
pub async fn assist_truck_memory<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TruckMemoryInput,
) -> Result<TruckMemory, UiError> {
    run(app, "assist_truck_memory", move |ctx| {
        auth::require(ctx)?;
        assist::truck_memory(ctx, &input)
    })
    .await
}

/// The item and packing of a seller's last lot and his مارکہ (PRD FR-7.51). The screen puts them
/// into the visible fields. Read only. Errors: `ERR_NOT_FOUND`, `ERR_NOT_A_SELLER` on
/// `seller_account_id`.
#[tauri::command]
pub async fn assist_seller_memory<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SellerMemoryInput,
) -> Result<SellerMemory, UiError> {
    run(app, "assist_seller_memory", move |ctx| {
        auth::require(ctx)?;
        assist::seller_memory(ctx, input.seller_account_id)
    })
    .await
}

/// The last saved rate of an item and packing, for grey placeholder text only (PRD FR-7.53, Code
/// Standards B29). Null when it never sold. Read only. Errors: the date codes on `date`.
#[tauri::command]
pub async fn assist_last_rate<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LastRateInput,
) -> Result<Option<RateHint>, UiError> {
    run(app, "assist_last_rate", move |ctx| {
        auth::require(ctx)?;
        assist::last_rate(ctx, &input)
    })
    .await
}

/// A buyer's open bills, oldest first, with their age, the total, the credit that no bill has
/// used and his balance (PRD FR-7.58). For the پختہ کھاتہ buyer account the person is named by
/// `person_key`. The "پورا بقایا" and "پرانا بل" buttons fill the amount from this answer. Read
/// only. Errors: `ERR_NOT_FOUND` (`account_id`), `ERR_NOT_A_BUYER`, `ERR_PERSON_REQUIRED`.
#[tauri::command]
pub async fn assist_open_bills<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OpenBillsInput,
) -> Result<OpenBills, UiError> {
    run(app, "assist_open_bills", move |ctx| {
        auth::require(ctx)?;
        vouchers::open_bills(ctx, &input)
    })
    .await
}

/// What is payable to a seller (PRD FR-7.59): his credit postings (بکری, opening credit, جنرل
/// credits) with the advance already adjusted against each, what was paid, and what is still
/// open, oldest first, worked out when shown. Also his open advances, what he owes beyond them
/// and his balance. Read only. Errors: `ERR_NOT_FOUND`, `ERR_NOT_A_SELLER` (`seller_id`),
/// `ERR_PERSON_REQUIRED`.
#[tauri::command]
pub async fn assist_open_payables<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OpenPayablesInput,
) -> Result<OpenPayables, UiError> {
    run(app, "assist_open_payables", move |ctx| {
        auth::require(ctx)?;
        vouchers::open_payables(ctx, &input)
    })
    .await
}

/// The اگراہی فہرست (PRD FR-15.6): the buyers planned for a day by their collection weekday,
/// with their dues, grouped by collector and city. Read only. Errors: the date codes on `date`.
#[tauri::command]
pub async fn assist_collection_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CollectionListInput,
) -> Result<CollectionList, UiError> {
    run(app, "assist_collection_list", move |ctx| {
        auth::require(ctx)?;
        vouchers::collection_list(ctx, &input)
    })
    .await
}
