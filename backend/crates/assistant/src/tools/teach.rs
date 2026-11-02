//! Refusals that teach (plan 51, P51-B2).
//!
//! A tool refusal goes to the model as `{"error":{"code","field","detail","allowed"}}`, and the
//! model may correct its input once. For that to work in one try, the refusal must name the field
//! as the tool's own input spells it, say what is wrong in words, and list the words that are
//! allowed. Umar's question "apni sale ko ziyada kaise karain" lost two rounds on `sort.column`
//! "sales" (no such column) and on `compare` with no `from`: the model got an engine field name, an
//! empty detail and no list.
//!
//! Example: a sort on `sales` in view A is refused with `allowed` = `name, nag, gross, avg_rate,
//! seller_comm, buyer_comm, laga, total_comm, share, difference.total_comm, earlier.total_comm`
//! (the last two only when `compare` is on).
//!
//! No figure is read or changed here: the lists are column names, filter names and view letters.

use mandi_reports::analytics::{AnalyticsFilterKind, AnalyticsView, catalog as view_catalog};
use mandi_reports::reports::{Column, FilterKind};

use super::cells::{is_left_out, shown_key};

/// The columns of the comparison, which the model reads as `difference.<figure>` and
/// `earlier.<figure>` and never by these engine keys.
const COMPARE_KEYS: [&str; 2] = ["compare", "change"];

/// The name of a field as the tool's input spells it. The engine says `filters.from` and
/// `filters.item_id`; the tool's input says `from` and `item`.
pub fn tool_field(engine_field: &str) -> String {
    let Some(rest) = engine_field.strip_prefix("filters.") else {
        return engine_field.to_owned();
    };
    let field = rest.split('.').next().unwrap_or(rest);
    match field {
        "item_id" => "item",
        "seller_id" => "seller",
        "city_id" => "city",
        "adda_id" => "adda",
        "trip_id" => "trip",
        "garden_id" | "garden_account_id" => "garden",
        "partner_id" => "partner",
        "bikri_id" => "bikri",
        "person_key" => "party",
        other => other,
    }
    .to_owned()
}

/// Why an engine code refused, in words for the model. Never empty.
pub fn detail_for(code: &str, field: Option<&str>) -> String {
    let needs_dates = matches!(field, Some("from" | "to"));
    match code {
        "ERR_REQUIRED" if needs_dates => {
            "this needs a period with both dates: give from and to, for example from a period \
             such as this_year or last_month"
                .to_owned()
        }
        "ERR_REQUIRED" => "a value is needed in this field".to_owned(),
        "ERR_DATE_RANGE" => "to is before from".to_owned(),
        "ERR_DATE_INVALID" => "a date that does not exist".to_owned(),
        "ERR_NOT_FOUND" => "there is no such record in the books".to_owned(),
        "ERR_OUT_OF_RANGE" => "a value this input does not take".to_owned(),
        "ERR_FORMAT_INVALID" => "the input is not in the shape the tool takes".to_owned(),
        "ERR_INTERNAL" => {
            "the books could not be read for this input; do not repeat the same call".to_owned()
        }
        _ => "the books refused this input".to_owned(),
    }
}

/// The words a sort or a `where` may use for these columns, as the model reads them: the first
/// column that names a row is `name`, a ledger entry is `entry`, and a comparison is
/// `difference.<figure>` and `earlier.<figure>`.
pub fn column_words(columns: &[Column], analytics: bool, figure: Option<&str>) -> Vec<String> {
    let mut words: Vec<String> = columns
        .iter()
        .enumerate()
        .filter(|(_, c)| !is_left_out(c) && !COMPARE_KEYS.contains(&c.key.as_str()))
        .map(|(i, c)| shown_key(i, c, analytics))
        .collect();
    if let Some(figure) = figure {
        if columns.iter().any(|c| c.key == "change") {
            words.push(format!("difference.{figure}"));
        }
        if columns.iter().any(|c| c.key == "compare") {
            words.push(format!("earlier.{figure}"));
        }
    }
    words
}

/// The views that take a filter, as their letters.
pub fn views_with(kind: AnalyticsFilterKind) -> Vec<String> {
    view_catalog()
        .iter()
        .filter(|entry| entry.filters.contains(&kind))
        .map(|entry| entry.id.code().to_owned())
        .collect()
}

/// The letters of the views that take a comparison.
pub fn compare_views() -> Vec<String> {
    views_with(AnalyticsFilterKind::Compare)
}

/// The tool input fields a report takes, from its catalogue filters.
pub fn report_fields(filters: &[FilterKind]) -> Vec<String> {
    let mut out: Vec<&str> = Vec::new();
    for kind in filters {
        match kind {
            FilterKind::DateRange => out.extend(["from", "to"]),
            FilterKind::AsOf => out.push("as_of"),
            FilterKind::Party | FilterKind::Seller | FilterKind::Zamindar => out.push("party"),
            FilterKind::Item => out.push("item"),
            FilterKind::City => out.push("city"),
            FilterKind::Adda => out.push("adda"),
            FilterKind::GroupBy => out.push("group_by"),
            FilterKind::Partner => out.push("partner"),
            FilterKind::Garden => out.push("garden"),
            FilterKind::Bikri => out.push("bikri"),
            _ => {}
        }
    }
    out.dedup();
    out.into_iter().map(str::to_owned).collect()
}

/// The view letter, for a message.
pub fn view_letter(view: AnalyticsView) -> String {
    view.code().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_fields_become_the_tools_own() {
        assert_eq!(tool_field("filters.from"), "from");
        assert_eq!(tool_field("filters.item_id"), "item");
        assert_eq!(tool_field("filters.buyer.account_id"), "buyer");
        assert_eq!(tool_field("sort.column"), "sort.column");
        assert_eq!(tool_field("filters.compare"), "compare");
    }

    #[test]
    fn every_code_has_a_detail() {
        for code in [
            "ERR_REQUIRED",
            "ERR_DATE_RANGE",
            "ERR_DATE_INVALID",
            "ERR_NOT_FOUND",
            "ERR_OUT_OF_RANGE",
            "ERR_FORMAT_INVALID",
            "ERR_INTERNAL",
            "ERR_ANYTHING_ELSE",
        ] {
            assert!(!detail_for(code, None).is_empty(), "{code}");
            assert!(!detail_for(code, Some("from")).is_empty(), "{code}");
        }
    }

    #[test]
    fn only_some_views_take_a_comparison() {
        let views = compare_views();
        assert!(views.contains(&"A".to_owned()));
        assert!(!views.contains(&"G".to_owned()));
        assert!(!views.contains(&"F".to_owned()));
    }
}
