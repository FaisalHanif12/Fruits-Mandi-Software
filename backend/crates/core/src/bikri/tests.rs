//! Reading the text of a draft and splitting the freight (PRD 12.3 to 12.4, C17).

use super::*;
use crate::dates::BusinessDate;
use crate::error::ErrorCode;
use crate::money::{Paisa, Qty};

fn today() -> BusinessDate {
    BusinessDate::from_ymd(2026, 10, 10).unwrap()
}

fn sale(buyer: Option<i64>, qty: &str, rate: &str) -> SaleInput {
    SaleInput {
        key: "s".to_owned(),
        buyer_account_id: buyer,
        pukhta: None,
        qty: qty.to_owned(),
        weight: None,
        seller_rate: rate.to_owned(),
        rate_differs: false,
        buyer_rate: None,
        payment_type: None,
        seller_comm: None,
        buyer_comm: None,
        laga: None,
        note: None,
    }
}

fn lot(sales: Vec<SaleInput>) -> LotInput {
    LotInput {
        key: "l".to_owned(),
        seller_account_id: Some(1),
        pukhta: None,
        item_id: Some(1),
        packing_id: Some(1),
        marka_id: None,
        store_id: None,
        qty_received: "50".to_owned(),
        weight_received: None,
        freight: Some("3,000".to_owned()),
        freight_paid_by: FreightPaidBy::Firm,
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: Some("5000".to_owned()),
        store_rent: None,
        adjust_advance: true,
        sales,
    }
}

fn header() -> HeaderInput {
    HeaderInput {
        entry_date: "10-10-2026".to_owned(),
        bill_date: String::new(),
        book: None,
        truck_no: Some(" LES-1234 ".to_owned()),
        zamindar_account_id: None,
        adda_id: None,
        driver_name: None,
        driver_phone: None,
        total_freight: None,
        store_id: None,
        note: None,
    }
}

#[test]
fn reads_the_text_of_event_1() {
    let draft = BikriDraft {
        header: header(),
        lots: vec![lot(vec![
            sale(Some(7), "20", "600"),
            sale(Some(8), "30", "580"),
        ])],
    };
    let prepared = prepare_draft(&draft, today(), None);
    assert!(prepared.issues.is_empty(), "{:?}", prepared.issues);
    assert_eq!(prepared.header.truck_no.as_deref(), Some("LES-1234"));
    assert_eq!(prepared.header.bill_date, prepared.header.entry_date);
    let lot = &prepared.lots[0];
    assert_eq!(lot.qty_received.map(Qty::get), Some(50));
    assert_eq!(lot.freight, Paisa::from_rupees(3_000).unwrap());
    assert_eq!(lot.cash_paid, Some(Paisa::from_rupees(5_000).unwrap()));
    assert_eq!(lot.sales[0].qty.map(Qty::get), Some(20));
    assert_eq!(
        lot.sales[1].seller_rate,
        Some(Paisa::from_rupees(580).unwrap())
    );
}

#[test]
fn a_sum_in_the_qty_field_is_taken() {
    let draft = BikriDraft {
        header: header(),
        lots: vec![lot(vec![sale(Some(7), "20+5", "600")])],
    };
    let prepared = prepare_draft(&draft, today(), None);
    assert_eq!(prepared.lots[0].sales[0].qty.map(Qty::get), Some(25));
}

#[test]
fn a_row_with_no_buyer_and_no_rate_is_blank_and_not_read() {
    let draft = BikriDraft {
        header: header(),
        lots: vec![lot(vec![sale(None, "abc", "")])],
    };
    let prepared = prepare_draft(&draft, today(), None);
    assert!(prepared.issues.is_empty());
    assert!(prepared.lots[0].sales[0].is_blank);
}

#[test]
fn every_mistake_comes_back_with_its_field() {
    let mut bad = sale(Some(7), "abc", "6.5");
    bad.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "101".to_owned(),
    });
    let draft = BikriDraft {
        header: header(),
        lots: vec![lot(vec![bad])],
    };
    let prepared = prepare_draft(&draft, today(), None);
    let found: Vec<(ErrorCode, &str)> = prepared
        .issues
        .iter()
        .map(|e| (e.code, e.field.as_str()))
        .collect();
    assert_eq!(
        found,
        vec![
            (ErrorCode::ErrNotANumber, "lots[0].sales[0].qty"),
            (
                ErrorCode::ErrDecimalsNotAllowed,
                "lots[0].sales[0].seller_rate"
            ),
            (ErrorCode::ErrPercentOver100, "lots[0].sales[0].seller_comm"),
        ]
    );
    assert!(prepared.lots[0].sales[0].has_errors);
}

#[test]
fn a_closed_or_future_posting_day_is_refused_at_the_header() {
    let mut h = header();
    h.entry_date = "09-10-2026".to_owned();
    let closed = BusinessDate::from_ymd(2026, 10, 9);
    let draft = BikriDraft {
        header: h,
        lots: vec![],
    };
    let prepared = prepare_draft(&draft, today(), closed);
    assert_eq!(prepared.issues[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(prepared.issues[0].field, "header.entry_date");
}

#[test]
fn too_many_lots_and_lines_are_cut_off() {
    let lots = (0..MAX_LOTS + 1).map(|_| lot(vec![])).collect();
    let draft = BikriDraft {
        header: header(),
        lots,
    };
    let prepared = prepare_draft(&draft, today(), None);
    assert_eq!(prepared.lots.len(), MAX_LOTS);
    assert!(
        prepared
            .issues
            .iter()
            .any(|e| e.code == ErrorCode::ErrOutOfRange && e.field == "lots")
    );
    let many = (0..MAX_SALE_LINES + 3)
        .map(|_| sale(Some(7), "1", "600"))
        .collect();
    let draft = BikriDraft {
        header: header(),
        lots: vec![lot(many)],
    };
    let prepared = prepare_draft(&draft, today(), None);
    assert_eq!(prepared.lots[0].sales.len(), MAX_SALE_LINES);
    assert!(
        prepared
            .issues
            .iter()
            .any(|e| e.code == ErrorCode::ErrOutOfRange && e.field == "lots[0].sales")
    );
}

#[test]
fn the_freight_split_follows_c17() {
    // PRD C17: 1,000 over lots of 30 and 20 نگ gives 600 and 400.
    let input = SplitFreightInput {
        total_freight: "1,000".to_owned(),
        lots: vec![
            FreightLotInput {
                key: "a".to_owned(),
                qty_received: "30".to_owned(),
            },
            FreightLotInput {
                key: "b".to_owned(),
                qty_received: "20".to_owned(),
            },
        ],
    };
    let parts = split_freight(&input).unwrap();
    let rupees: Vec<(&str, i64)> = parts
        .iter()
        .map(|p| (p.key.as_str(), p.freight_paisa.rupees()))
        .collect();
    assert_eq!(rupees, vec![("a", 600), ("b", 400)]);
    // 1,000 over three lots of 1 نگ: the leftover rupee goes to the first lot.
    let three = SplitFreightInput {
        total_freight: "1000".to_owned(),
        lots: ["a", "b", "c"]
            .iter()
            .map(|k| FreightLotInput {
                key: (*k).to_owned(),
                qty_received: "1".to_owned(),
            })
            .collect(),
    };
    let rupees: Vec<i64> = split_freight(&three)
        .unwrap()
        .iter()
        .map(|p| p.freight_paisa.rupees())
        .collect();
    assert_eq!(rupees, vec![334, 333, 333]);
}

#[test]
fn the_freight_split_names_the_bad_fields() {
    let input = SplitFreightInput {
        total_freight: "10.5".to_owned(),
        lots: vec![FreightLotInput {
            key: "a".to_owned(),
            qty_received: "0".to_owned(),
        }],
    };
    let errors = split_freight(&input).unwrap_err();
    let found: Vec<(ErrorCode, &str)> = errors.iter().map(|e| (e.code, e.field.as_str())).collect();
    assert_eq!(
        found,
        vec![
            (ErrorCode::ErrDecimalsNotAllowed, "total_freight"),
            (ErrorCode::ErrMustBeMoreThanZero, "lots[0].qty_received"),
        ]
    );
    let none = SplitFreightInput {
        total_freight: "100".to_owned(),
        lots: vec![],
    };
    assert_eq!(
        split_freight(&none).unwrap_err()[0].code,
        ErrorCode::ErrNoLots
    );
}
