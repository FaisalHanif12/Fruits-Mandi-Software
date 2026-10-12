//! Read-only help for the entry screens (Tech Stack 4.7, PRD 7.6). Nothing here writes
//! (Code Standards B29).

use mandi_services::core::accounts::{PartyCard, PartyCardInput};
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
