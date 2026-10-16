//! Read-only help for the entry screens (Tech Stack 4.7, PRD 7.6). Nothing here writes
//! (Code Standards B29).

use mandi_services::core::accounts::{PartyCard, PartyCardInput};
use mandi_services::core::bikri::{
    LastRateInput, RateHint, SellerMemory, SellerMemoryInput, TruckMemory, TruckMemoryInput,
};
use mandi_services::core::vouchers::{AdvancesInfo, AssistAdvancesInput};
use mandi_services::{assist, auth};

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
