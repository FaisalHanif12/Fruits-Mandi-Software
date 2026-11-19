//! The report filters the tool `report` passes on beyond the plain ones (plan 55 §11.1, P1 to P4).
//!
//! Each report's spec in `mandi_reports::reports::catalog` says which filters it takes. The tool
//! used to pass only some of them. These are the rest, each checked against the same spec (a filter
//! the report does not have is an error that lists what it does take), and each copied into the
//! button's filters (`SourceReportFilters`) so the button opens the very report that was read.
//!
//! | Input | Report | Engine filter |
//! |---|---|---|
//! | `seller` | R2 R3 R4 R5 R7 R10 R6 R27 R26 | `seller_id` |
//! | `zamindar` | R2 R3 R4 R5 R6 R27 | `zamindar_id` |
//! | `status` `paid` or `due` | R38 | `status` |
//! | `voucher_type` | R15 | `voucher_type` |
//! | `layout` | R14 `detail`; R33 `item` `day` `month` `partner`; R36 `partner`; R10 `item` | `detail`, `trip_view`, `garden_view`, `stock_view` |
//! | `partner: "guests"` | R37 | `guests_only` |
//! | `item_group` `fruit` or `vegetable` | R2 R3 R4 R7 R10 R27 and analytics A B C D G H | `item_group` |
//! | `grade` `first`, `second` or `third` | R2 R3 R4 and analytics A | `grade` |
//!
//! The `collector` input (R22, R23, R35) is read in `report.rs` with the other records; the
//! reports crate has the filter in each of those three specs.
//!
//! Example (worked day): `report{id:"R38", status:"due"}` keeps the people of the dead amount who still
//! owe; `report{id:"R37", partner:"guests"}` reads only the fruit taken for the shop's guests.

use mandi_reports::reports::{
    FilterKind, GardenView, PaidFilter, ReportFilters, StockView, TripView,
};
use mandi_services::bikri;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{LotFindInput, SaleGrade, lot_label};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::rules::ItemGroup;
use mandi_services::core::vouchers::VoucherType;

use super::records::{named, require_type};
use super::report::not_taken;
use super::{Env, State, ToolError};
use crate::source::{AnswerRecord, SourceReportFilters};

/// The word of `partner` that means the shop's guests, not a partner (R37).
pub const GUESTS_WORD: &str = "guests";

/// The new inputs of `report`, as the model sent them.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Extra<'a> {
    /// A seller (a find match or a key).
    pub seller: Option<&'a str>,
    /// A زمیندار (a find match or a key).
    pub zamindar: Option<&'a str>,
    /// `paid` or `due`.
    pub status: Option<&'a str>,
    /// One kind of voucher, in its stored word.
    pub voucher_type: Option<&'a str>,
    /// How the report lays out its rows.
    pub layout: Option<&'a str>,
    /// `fruit` or `vegetable`: all the items of a group, in place of one item.
    pub item_group: Option<&'a str>,
    /// `first`, `second` or `third`: only the sale lines of that quality.
    pub grade: Option<&'a str>,
    /// A lot key (`l7`) set by the app for the lot the Owner picked.
    pub lot: Option<&'a str>,
}

/// The stored words of the voucher kinds, for the refusal of a wrong one.
const VOUCHER_WORDS: [&str; 13] = [
    "receipt",
    "payment",
    "advance",
    "shop_expense",
    "personal_expense",
    "personal_return",
    "money_lent",
    "money_lent_return",
    "general",
    "capital_in",
    "capital_out",
    "khata_out",
    "khata_in",
];

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|w| (*w).to_owned()).collect()
}

fn wrong_word(field: &str, allowed: &[&str]) -> ToolError {
    ToolError::new(
        "ERR_OUT_OF_RANGE",
        field,
        "not a word this input takes; the allowed words are listed",
    )
    .allowed(words(allowed))
}

/// Reads the `item_group` word of a report or an analytics view (PRD v1.29).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` for a word that is not `fruit` or `vegetable` (the allowed words are listed),
/// and for a group given together with one item: the filter is the group or the item, never both.
pub(super) fn parse_group(word: &str, item_given: bool) -> Result<ItemGroup, ToolError> {
    if item_given {
        return Err(ToolError::new(
            "ERR_OUT_OF_RANGE",
            "item_group",
            "give an item or an item group, not both",
        ));
    }
    ItemGroup::parse(word.trim()).ok_or_else(|| wrong_word("item_group", &["fruit", "vegetable"]))
}

/// Reads the `grade` word of a report or an analytics view (PRD FR-12.42).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `grade` for a word that is not `first`, `second` or `third` (the allowed
/// words are listed).
pub(super) fn parse_grade(word: &str) -> Result<SaleGrade, ToolError> {
    SaleGrade::parse(word.trim()).ok_or_else(|| wrong_word("grade", &["first", "second", "third"]))
}

/// True when the partner input means the guests (R37).
pub(super) fn is_guests(partner: &str) -> bool {
    partner.trim().eq_ignore_ascii_case(GUESTS_WORD)
}

/// The guests filter of R37: `partner: "guests"`.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` when the report is not the fruit register.
pub(super) fn guests(
    takes: &[FilterKind],
    filters: &mut ReportFilters,
    source: &mut SourceReportFilters,
) -> Result<(), ToolError> {
    if !takes.contains(&FilterKind::Guests) {
        return Err(not_taken(takes, "partner"));
    }
    filters.guests_only = Some(true);
    source.guests_only = true;
    Ok(())
}

/// Reads the new inputs into the engine's filters and the button's filters.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` for a filter the report does not have or a word the input does not take;
/// `ERR_FORMAT_INVALID` for a key of another kind of record, or a seller named twice;
/// `ERR_UNKNOWN_REFERENCE` for a record the conversation never showed.
pub(super) fn apply(
    env: &Env<'_>,
    state: &State,
    extra: &Extra<'_>,
    takes: &[FilterKind],
    filters: &mut ReportFilters,
    source: &mut SourceReportFilters,
) -> Result<(), ToolError> {
    if let Some(text) = extra.seller {
        if !takes.contains(&FilterKind::Seller) {
            return Err(not_taken(takes, "seller"));
        }
        // `party` already became the seller on a report that has no buyer filter (R6, R10, R26,
        // R27): naming him twice is a mistake, not a second filter.
        if filters.seller_id.is_some() {
            return Err(ToolError::new(
                "ERR_FORMAT_INVALID",
                "seller",
                "this report takes one seller, and party already named him; give only one",
            ));
        }
        let (id, record) = named(env, state, text, "seller", &["a"])?;
        require_type(env, id, "seller", AccountType::Seller)?;
        filters.seller_id = Some(id);
        source.seller = Some(record);
    }
    if let Some(text) = extra.zamindar {
        if !takes.contains(&FilterKind::Zamindar) {
            return Err(not_taken(takes, "zamindar"));
        }
        let (id, record) = named(env, state, text, "zamindar", &["a"])?;
        require_type(env, id, "zamindar", AccountType::Seller)?;
        filters.zamindar_id = Some(id);
        source.zamindar = Some(record);
    }
    if let Some(word) = extra.status {
        if !takes.contains(&FilterKind::Status) {
            return Err(not_taken(takes, "status"));
        }
        let status = match word {
            "paid" => PaidFilter::Paid,
            "due" => PaidFilter::Due,
            _ => return Err(wrong_word("status", &["paid", "due"])),
        };
        filters.status = Some(status);
        source.status = Some(status);
    }
    if let Some(word) = extra.voucher_type {
        if !takes.contains(&FilterKind::VoucherType) {
            return Err(not_taken(takes, "voucher_type"));
        }
        let kind =
            VoucherType::parse(word).ok_or_else(|| wrong_word("voucher_type", &VOUCHER_WORDS))?;
        filters.voucher_type = Some(kind);
        source.voucher_type = Some(kind);
    }
    if let Some(word) = extra.layout {
        layout(word, takes, filters, source)?;
    }
    if let Some(word) = extra.grade {
        if !takes.contains(&FilterKind::Grade) {
            return Err(not_taken(takes, "grade"));
        }
        let grade = parse_grade(word)?;
        filters.grade = Some(grade);
        source.grade = Some(grade);
    }
    if let Some(word) = extra.item_group {
        if !takes.contains(&FilterKind::ItemGroup) {
            return Err(not_taken(takes, "item_group"));
        }
        let group = parse_group(word, filters.item_id.is_some())?;
        filters.item_group = Some(group);
        source.item_group = Some(group);
    }
    if let Some(key) = extra.lot {
        narrow_to_lot(env, key, takes, filters, source)?;
    }
    Ok(())
}

/// Narrows a report of one seller and one item to the lot the Owner picked (plan 71 L9, PRD
/// FR-12.45, FR-7.77). It is a help of the app, never a refusal: a report that has no lot filter, a
/// report that is not of exactly that seller and item, or a lot that is not theirs is left as it
/// was asked.
///
/// Example: R5 for Ehsan and Mango with the key of Mango-2 reads only the lines of that lot.
fn narrow_to_lot(
    env: &Env<'_>,
    key: &str,
    takes: &[FilterKind],
    filters: &mut ReportFilters,
    source: &mut SourceReportFilters,
) -> Result<(), ToolError> {
    let lot_id = key.strip_prefix('l').and_then(|n| n.parse::<i64>().ok());
    let (Some(lot_id), Some(seller), Some(item)) = (lot_id, filters.seller_id, filters.item_id)
    else {
        return Ok(());
    };
    if !takes.contains(&FilterKind::Lot) {
        return Ok(());
    }
    let (found, _) = bikri::find_lots(
        env.ctx,
        &LotFindInput {
            seller_ids: vec![seller],
            item_ids: vec![item],
            lot_id: Some(lot_id),
            limit: 1,
            ..LotFindInput::default()
        },
    )
    .map_err(|_| ToolError::from_code(ErrorCode::ErrInternal, None))?;
    if let Some(lot) = found.into_iter().next() {
        filters.lot_id = Some(lot_id);
        source.lot = Some(Box::new(AnswerRecord {
            id: lot_id,
            code: None,
            name_ur: format!(
                "{}، {}",
                lot.name.seller.name_ur,
                lot_label(&lot.name.item.name_ur, lot.name.counter)
            ),
            name_en: format!(
                "{}, {}",
                lot.name.seller.name_en,
                lot_label(&lot.name.item.name_en, lot.name.counter)
            ),
            tag: lot.name.seller.tag.clone(),
        }));
    }
    Ok(())
}

/// `layout` is one word for four reports: R14 with its lines (`detail`), R33 by `item`, `day`,
/// `month` or `partner` (`trip`, the default, is the same as leaving it out), R36 by `partner`
/// (`garden`, the default, is the same as leaving it out), and R10 by `item` (one row for each
/// item with the quantities of all its lots; `lot`, the default, is the same as leaving it out).
fn layout(
    word: &str,
    takes: &[FilterKind],
    filters: &mut ReportFilters,
    source: &mut SourceReportFilters,
) -> Result<(), ToolError> {
    if takes.contains(&FilterKind::Detail) {
        return match word {
            "detail" => {
                filters.detail = true;
                source.detail = true;
                Ok(())
            }
            _ => Err(wrong_word("layout", &["detail"])),
        };
    }
    if takes.contains(&FilterKind::TripView) {
        let view = match word {
            "trip" => TripView::Trip,
            "item" => TripView::Item,
            "day" => TripView::Day,
            "month" => TripView::Month,
            "partner" => TripView::Partner,
            _ => {
                return Err(wrong_word(
                    "layout",
                    &["trip", "item", "day", "month", "partner"],
                ));
            }
        };
        filters.trip_view = Some(view);
        source.trip_view = Some(view);
        return Ok(());
    }
    if takes.contains(&FilterKind::GardenView) {
        let view = match word {
            "garden" => GardenView::Garden,
            "partner" => GardenView::Partner,
            _ => return Err(wrong_word("layout", &["garden", "partner"])),
        };
        filters.garden_view = Some(view);
        source.garden_view = Some(view);
        return Ok(());
    }
    if takes.contains(&FilterKind::StockView) {
        // `lot` (the default) is the same as leaving it out.
        let view = match word {
            "lot" => StockView::Lot,
            "item" => StockView::Item,
            _ => return Err(wrong_word("layout", &["lot", "item"])),
        };
        filters.stock_view = Some(view);
        source.stock_view = (view == StockView::Item).then_some(view);
        return Ok(());
    }
    Err(not_taken(takes, "layout"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_guests_word_is_read_in_any_case() {
        assert!(is_guests("guests"));
        assert!(is_guests(" Guests "));
        assert!(!is_guests("s1"));
        assert!(!is_guests("guest"));
    }

    #[test]
    fn an_item_group_is_a_word_and_never_comes_with_an_item() {
        assert_eq!(parse_group("fruit", false).unwrap(), ItemGroup::Fruit);
        assert_eq!(
            parse_group(" vegetable ", false).unwrap(),
            ItemGroup::Vegetable
        );
        let wrong = parse_group("grain", false).unwrap_err();
        assert_eq!(wrong.code, "ERR_OUT_OF_RANGE");
        assert_eq!(wrong.allowed, ["fruit", "vegetable"]);
        let both = parse_group("fruit", true).unwrap_err();
        assert_eq!(both.field.as_deref(), Some("item_group"));
    }

    #[test]
    fn a_grade_is_one_of_three_words() {
        assert_eq!(parse_grade("first").unwrap(), SaleGrade::First);
        assert_eq!(parse_grade(" third ").unwrap(), SaleGrade::Third);
        let wrong = parse_grade("fourth").unwrap_err();
        assert_eq!(wrong.code, "ERR_OUT_OF_RANGE");
        assert_eq!(wrong.field.as_deref(), Some("grade"));
        assert_eq!(wrong.allowed, ["first", "second", "third"]);
        assert!(parse_grade("none").is_err());
    }

    #[test]
    fn every_voucher_word_is_a_voucher_kind() {
        for word in VOUCHER_WORDS {
            assert!(VoucherType::parse(word).is_some(), "{word}");
        }
        assert!(VoucherType::parse("cheque").is_none());
    }

    #[test]
    fn a_layout_belongs_to_one_of_three_reports() {
        let mut filters = ReportFilters::default();
        let mut source = SourceReportFilters::default();
        // R33 by month.
        let trips = [FilterKind::DateRange, FilterKind::TripView];
        layout("month", &trips, &mut filters, &mut source).unwrap();
        assert_eq!(filters.trip_view, Some(TripView::Month));
        assert_eq!(source.trip_view, Some(TripView::Month));
        // R36 by partner; "month" is not a garden layout.
        let gardens = [FilterKind::GardenView];
        layout("partner", &gardens, &mut filters, &mut source).unwrap();
        assert_eq!(filters.garden_view, Some(GardenView::Partner));
        assert_eq!(
            layout("month", &gardens, &mut filters, &mut source)
                .unwrap_err()
                .allowed,
            ["garden", "partner"]
        );
        // R14 with its lines.
        let statement = [FilterKind::Detail];
        layout("detail", &statement, &mut filters, &mut source).unwrap();
        assert!(filters.detail && source.detail);
        // A report with no layout at all refuses and lists what it takes.
        let plain = [FilterKind::DateRange];
        let error = layout("detail", &plain, &mut filters, &mut source).unwrap_err();
        assert_eq!(error.code, "ERR_OUT_OF_RANGE");
        assert_eq!(error.field.as_deref(), Some("layout"));
    }
}
