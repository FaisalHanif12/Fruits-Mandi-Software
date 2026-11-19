//! Read-only help for the entry screens (Tech Stack 4.7, PRD 7.6). Nothing here writes
//! (Code Standards B29).

use mandi_services::core::accounts::{PartyCard, PartyCardInput};
use mandi_services::core::bikri::{LastRateInput, RateHint, SellerMemory, SellerMemoryInput};
use mandi_services::core::paging::PageInput;
use mandi_services::core::vouchers::{
    AdvancesInfo, AssistAdvancesInput, CollectionList, CollectionListInput, OpenBills,
    OpenBillsInput, OpenPayables, OpenPayablesInput,
};
use mandi_services::{assist, auth, vouchers};

use serde::Deserialize;
use ts_rs::TS;

use super::run;
use crate::error::UiError;

/// Input of `assist_open_bills`: the buyer and, since plan 76 point 8, the page of his bills. No
/// page gives the oldest 200, as before.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpenBillsPageInput {
    /// The buyer.
    #[serde(flatten)]
    #[ts(flatten)]
    pub base: OpenBillsInput,
    /// Where the page starts and how many bills it holds (at most 200).
    #[serde(flatten)]
    #[ts(flatten)]
    pub page: PageInput,
}

/// Input of `assist_open_payables`: the seller and the page of his postings. No page gives the
/// oldest 200, as before (all of them with `include_settled`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpenPayablesPageInput {
    /// The seller.
    #[serde(flatten)]
    #[ts(flatten)]
    pub base: OpenPayablesInput,
    /// Where the page starts and how many postings it holds.
    #[serde(flatten)]
    #[ts(flatten)]
    pub page: PageInput,
}

/// Input of `assist_advances`: the seller and the page of his open advances. No page gives the
/// oldest 200.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AssistAdvancesPageInput {
    /// The seller.
    #[serde(flatten)]
    #[ts(flatten)]
    pub base: AssistAdvancesInput,
    /// Where the page starts and how many advances it holds (at most 200).
    #[serde(flatten)]
    #[ts(flatten)]
    pub page: PageInput,
}

/// The card next to an account field: names, father or firm, city, phone, balance with its
/// side, last entry date, a buyer's credit limit and use, a seller's open advance, and warnings
/// (PRD FR-7.38). Balances are read from the journal. With `chukta_id` the card is that چکتا
/// buyer's own: his name and phone and no balance (`is_chukta`, PRD FR-13.6). Errors:
/// `ERR_NOT_FOUND` on `account_id` or `chukta_id`.
#[tauri::command]
pub async fn assist_party_card<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartyCardInput,
) -> Result<PartyCard, UiError> {
    run(app, "assist_party_card", move |ctx| {
        auth::require(ctx)?;
        match input.chukta_id {
            Some(chukta_id) => assist::chukta_card(ctx, chukta_id),
            None => assist::party_card(ctx, input.account_id),
        }
    })
    .await
}

/// A seller's open advances and his last پیشگی voucher, for the card in the پیشگی screen (PRD
/// FR-7.60). Read only. Errors: `ERR_NOT_FOUND` on `seller_id`, `ERR_NOT_A_SELLER`.
#[tauri::command]
pub async fn assist_advances<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistAdvancesPageInput,
) -> Result<AdvancesInfo, UiError> {
    run(app, "assist_advances", move |ctx| {
        auth::require(ctx)?;
        assist::advances_page(ctx, input.base.seller_id, input.page)
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
/// used and his balance (PRD FR-7.58). For the چکتا کھاتہ buyer account the person is named by
/// `person_key`. The "پورا بقایا" and "پرانا بل" buttons fill the amount from this answer. Read
/// only. Errors: `ERR_NOT_FOUND` (`account_id`), `ERR_NOT_A_BUYER`, `ERR_PERSON_REQUIRED`.
#[tauri::command]
pub async fn assist_open_bills<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OpenBillsPageInput,
) -> Result<OpenBills, UiError> {
    run(app, "assist_open_bills", move |ctx| {
        auth::require(ctx)?;
        vouchers::open_bills_page(ctx, &input.base, input.page)
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
    input: OpenPayablesPageInput,
) -> Result<OpenPayables, UiError> {
    run(app, "assist_open_payables", move |ctx| {
        auth::require(ctx)?;
        vouchers::open_payables_page(ctx, &input.base, input.page)
    })
    .await
}

/// The اگراہی فہرست (PRD FR-15.6, plan 53 T2): every active buyer who owes, with his dues, grouped
/// by collector and city. There is no collection day. Read only.
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
