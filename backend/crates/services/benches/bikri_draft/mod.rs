//! The typed draft and the partners the بکری benchmarks work on (plans 12 and 13). Shared by
//! `bikri` (the live summary) and `bikri_save` (the save), so there is one copy.

#![allow(dead_code)]

use mandi_services::core::bikri::{BikriDraft, FreightPaidBy, HeaderInput, LotInput, SaleInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::partners::{PartnerCreateInput, PartnersSetSharesInput, ShareInput};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, auth, partners};

use crate::support::fail;

/// Adds partners A, B and C with 40 / 30 / 30 from today, as the worked day has them.
pub fn add_partners(env: &TestEnv, actor: &auth::Actor) {
    let mut ids = Vec::new();
    for (ur, en) in [("الف", "Partner A"), ("ب", "Partner B"), ("ج", "Partner C")] {
        let mut words: Vec<String> = Vec::new();
        let id = loop {
            let input = PartnerCreateInput {
                name_ur: ur.to_owned(),
                name_en: en.to_owned(),
                opening_capital: None,
                confirmed_rules_words: words.clone(),
            };
            match partners::create(&env.ctx, actor, &input) {
                Ok(row) => break row.id,
                Err(ServiceError::Rejected(items))
                    if items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
                {
                    for item in items {
                        if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                            words.extend(found.split(' ').map(str::to_owned));
                        }
                    }
                }
                Err(error) => fail(&format!("cannot add a partner: {error}")),
            }
        };
        ids.push(id);
    }
    let shares = ids
        .iter()
        .zip(["40", "30", "30"])
        .map(|(&partner_id, share)| ShareInput {
            partner_id,
            share: share.to_owned(),
        })
        .collect();
    partners::set_shares(
        &env.ctx,
        actor,
        &PartnersSetSharesInput {
            effective_from: "10-10-2026".to_owned(),
            shares,
        },
    )
    .unwrap_or_else(|e| fail(&format!("cannot set the shares: {e}")));
}

pub fn id_by_code(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|conn| mandi_services::db::accounts::id_by_code(conn, code))
        .ok()
        .flatten()
        .unwrap_or_else(|| fail(&format!("the volume has no account {code}")))
}

pub fn first_id(env: &TestEnv, table: &str, skip: i64) -> i64 {
    env.ctx
        .db()
        .read(|conn| {
            conn.query_row(
                &format!("SELECT id FROM {table} ORDER BY id LIMIT 1 OFFSET ?1"),
                [skip],
                |r| r.get(0),
            )
        })
        .unwrap_or_else(|e| fail(&format!("the volume has no {table}: {e}")))
}

/// `lots` lots of `lines` lines each: 500 نگ received, lines of 10 نگ at 600 to buyers in turn.
pub fn draft(env: &TestEnv, lots: usize, lines: usize) -> BikriDraft {
    let packing = first_id(env, "packings", 0);
    let lot = |n: usize| LotInput {
        key: format!("lot-{n}"),
        lot_id: None,
        seller_account_id: Some(id_by_code(env, &format!("03011{:04}", 1 + n))),
        pukhta: None,
        item_id: Some(first_id(env, "items", i64::try_from(n % 3).unwrap_or(0))),
        packing_id: Some(packing),
        marka_id: None,
        store_id: None,
        qty_received: "500".to_owned(),
        weight_received: None,
        freight: Some("3,000".to_owned()),
        freight_paid_by: FreightPaidBy::Firm,
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: Some("5,000".to_owned()),
        store_rent: None,
        adjust_advance: true,
        sales: (0..lines)
            .map(|j| SaleInput {
                key: format!("s-{n}-{j}"),
                sale_id: None,
                buyer_account_id: Some(id_by_code(
                    env,
                    &format!("0304{:04}", 1 + (n * lines + j) % 500),
                )),
                pukhta: None,
                qty: "10".to_owned(),
                weight: None,
                seller_rate: "600".to_owned(),
                rate_differs: false,
                buyer_rate: None,
                payment_type: None,
                seller_comm: None,
                buyer_comm: None,
                laga: None,
                note: None,
            })
            .collect(),
    };
    BikriDraft {
        header: HeaderInput {
            entry_date: "10-10-2026".to_owned(),
            bill_date: String::new(),
            book: None,
            truck_no: Some("LES-1234".to_owned()),
            zamindar_account_id: None,
            adda_id: None,
            driver_name: None,
            driver_phone: None,
            total_freight: None,
            store_id: None,
            note: None,
        },
        lots: (0..lots).map(lot).collect(),
    }
}
