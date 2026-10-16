//! Partners and their shares (PRD 17.1 to 17.3, plan 11). Every command needs the signed-in
//! Owner. Partners do not log in (PRD FR-2.4), so no user is made for a partner.

use mandi_services::core::partners::{
    PartnerCreateInput, PartnerDetail, PartnerOverviewRow, PartnerRow, PartnerUpdateInput,
    PartnersCheckSharesInput, PartnersGetInput, PartnersSetSharesInput, SharesCheck,
};
use mandi_services::{auth, partners};

use super::run;
use crate::error::UiError;

/// Every partner, active ones first, then by id. `share_bps` is the share on today's date.
#[tauri::command]
pub async fn partners_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<PartnerRow>, UiError> {
    run(app, "partners_list", move |ctx| {
        auth::require(ctx)?;
        partners::list(ctx)
    })
    .await
}

/// One partner with his share history. Errors: `ERR_NOT_FOUND` (`id`).
#[tauri::command]
pub async fn partners_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersGetInput,
) -> Result<PartnerDetail, UiError> {
    run(app, "partners_get", move |ctx| {
        auth::require(ctx)?;
        partners::get(ctx, input.id)
    })
    .await
}

/// Adds a partner with his capital and drawings accounts. He has no share until
/// `partners_set_shares`. Errors: the name codes (`name_ur`, `name_en`),
/// `ERR_SPELLING_UNCONFIRMED`, the money codes (`opening_capital`), `ERR_DATE_CLOSED` for the
/// opening, `ERR_TOO_MANY_PARTNERS` (`max`).
#[tauri::command]
pub async fn partners_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCreateInput,
) -> Result<PartnerRow, UiError> {
    run(app, "partners_create", move |ctx| {
        let actor = auth::require(ctx)?;
        partners::create(ctx, &actor, &input)
    })
    .await
}

/// Changes a partner's names or makes him inactive. Errors: `ERR_VERSION_CHANGED`,
/// `ERR_NOT_FOUND` (`id`), the name codes, `ERR_PARTNER_HAS_SHARE`.
#[tauri::command]
pub async fn partners_update<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerUpdateInput,
) -> Result<PartnerRow, UiError> {
    run(app, "partners_update", move |ctx| {
        let actor = auth::require(ctx)?;
        partners::update(ctx, &actor, &input)
    })
    .await
}

/// Sets the shares of all active partners from a day. Errors: `ERR_DATE_INVALID` and
/// `ERR_SHARE_BACKDATED` (`effective_from`), `ERR_PARTNER_NOT_LISTED` (`shares`), the percent
/// codes (`shares[i].share`), `ERR_SHARES_NOT_100` (`shares`, value `total_bps`).
#[tauri::command]
pub async fn partners_set_shares<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersSetSharesInput,
) -> Result<Vec<PartnerRow>, UiError> {
    run(app, "partners_set_shares", move |ctx| {
        let actor = auth::require(ctx)?;
        partners::set_shares(ctx, &actor, &input)
    })
    .await
}

/// The running total of typed shares. Read only; nothing is saved.
#[tauri::command]
pub async fn partners_check_shares<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersCheckSharesInput,
) -> Result<SharesCheck, UiError> {
    run(app, "partners_check_shares", move |ctx| {
        auth::require(ctx)?;
        partners::check(ctx, &input)
    })
    .await
}

/// One row per active partner: today, month and year income share, month expense share, profit,
/// money taken and capital balance.
#[tauri::command]
pub async fn partners_overview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<PartnerOverviewRow>, UiError> {
    run(app, "partners_overview", move |ctx| {
        auth::require(ctx)?;
        partners::overview(ctx)
    })
    .await
}
