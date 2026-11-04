//! What the local composers share (plan 55 Section 7): the shape they return, the guide's words, the
//! buttons of a report plan and the steps to do it by hand.
//!
//! Every word comes from the guide (`prompt/guide.json`) or from a Fluent message; every name is a
//! record of the books; every date is exact. No model is called and no figure is worked out (PRD
//! FR-7.76).
//!
//! Example: a plan for R5 (party Faisal, 01-10-2026 to 31-10-2026) gives the button "R5 with Faisal
//! and those dates", and the steps: open R5, choose the buyer Faisal (0022), set the dates, press F9.

use chrono::NaiveDate;
use mandi_reports::reports::{ArgValue, MessageArg, Names, ReportId, TripView};
use mandi_services::core::auth::Language;

use super::{PartyHit, PartyRole, RecordHit, RecordKind, ReportPlan};
use crate::answer::{
    Answer, AnswerName, AnswerPart, AnswerSource, EntryRoute, EntryScreen, HowStep,
};
use crate::ask::{AskOutcome, AskedRole, FoundAs};
use crate::prompt::{GuideText, guide};
use crate::source::{AnswerRecord, SourceParty, SourceReportFilters};

/// What a local composer made. The caller turns it into the reply of the ask command: it adds the
/// question number, the conversation and the counts, and it never calls a model for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composed {
    /// How the question ended: `Answered`, `NotFound` or `OutsideBooks`.
    pub outcome: AskOutcome,
    /// The answer. Null for `NotFound`: the panel owns the words there.
    pub answer: Option<Answer>,
    /// For `NotFound`: the kind of record that was asked for.
    pub asked_role: Option<AskedRole>,
    /// For `NotFound` when the name exists in another role: that record.
    pub found_as: Option<FoundAs>,
    /// For a help answer about a screen: the screen to open, with no party.
    pub entry_route: Option<EntryRoute>,
}

impl Composed {
    /// An answered question.
    pub(super) fn answered(answer: Answer) -> Self {
        Self {
            outcome: AskOutcome::Answered,
            answer: Some(answer),
            asked_role: None,
            found_as: None,
            entry_route: None,
        }
    }
}

/// An answer with nothing in it yet.
pub(super) fn bare(read_at: &str) -> Answer {
    Answer {
        parts: Vec::new(),
        tables: Vec::new(),
        charts: Vec::new(),
        estimate: None,
        advice: false,
        general_advice: false,
        sources: Vec::new(),
        how: Vec::new(),
        differences: Vec::new(),
        next: Vec::new(),
        read_at: read_at.to_owned(),
    }
}

/// One topic of the guide in the app language.
pub(super) fn guide_words(topic: &str, language: Language) -> Option<&'static GuideText> {
    let topic = guide().topics.get(topic)?;
    Some(match language {
        Language::Ur => &topic.ur,
        Language::En => &topic.en,
    })
}

fn text(text: &str) -> AnswerPart {
    AnswerPart::Text {
        text: text.to_owned(),
    }
}

/// The guide's own words of a topic as parts: the summary, each step on its own line with its
/// number, the keys and the worked example. Nothing is added to them.
pub(super) fn guide_parts(words: &GuideText) -> Vec<AnswerPart> {
    let mut parts = vec![text(&words.summary)];
    for (at, line) in words.steps.iter().enumerate() {
        parts.push(text(&format!("\n{}. {line}", at.saturating_add(1))));
    }
    if !words.keys.is_empty() {
        parts.push(text(&format!("\n{}", words.keys)));
    }
    if !words.example.is_empty() {
        parts.push(text(&format!("\n\n{}", words.example)));
    }
    parts
}

// ---- records ------------------------------------------------------------------------------------------

/// A party as a whole record of the answer (a picker value).
pub(super) fn record_of(hit: &PartyHit) -> AnswerRecord {
    AnswerRecord {
        id: hit.id,
        code: (!hit.code.is_empty()).then(|| hit.code.clone()),
        name_ur: hit.name_ur.clone(),
        name_en: hit.name_en.clone(),
        tag: hit.tag.clone(),
    }
}

/// An expense head, a city or an اڈا as a whole record of the answer.
pub(super) fn record_of_other(hit: &RecordHit) -> AnswerRecord {
    AnswerRecord {
        id: hit.id,
        code: (!hit.code.is_empty()).then(|| hit.code.clone()),
        name_ur: hit.name_ur.clone(),
        name_en: hit.name_en.clone(),
        tag: None,
    }
}

/// A party's name for `displayName`.
pub(super) fn name_of(hit: &PartyHit) -> AnswerName {
    AnswerName {
        name_ur: hit.name_ur.clone(),
        name_en: hit.name_en.clone(),
        tag: hit.tag.clone(),
        code: (!hit.code.is_empty()).then(|| hit.code.clone()),
    }
}

/// The record found in another role than the one asked.
pub(super) fn found_as_of(hit: &PartyHit) -> FoundAs {
    FoundAs {
        key: hit.key.clone(),
        id: hit.id,
        role: hit.role.asked_role(),
        name: name_of(hit),
    }
}

// ---- the buttons ---------------------------------------------------------------------------------------

/// The report of a code, if the app has it.
pub(super) fn report_id(code: &str) -> Option<ReportId> {
    serde_json::from_value(serde_json::Value::String(code.to_owned())).ok()
}

fn iso(day: Option<NaiveDate>) -> String {
    day.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// The button of a plan: the report with the filters of plan 54 Section 6.4 set, and an empty
/// `result_id` because nothing was read. `None` for a report the app does not have.
///
/// A trip (R33) and an expense head (R29) have no filter on their screens, so the button carries the
/// period only (and the trip layout for R33); the steps say which row to read (lead ruling of
/// 04-11-2026). The button never claims a filter the screen does not apply.
pub(super) fn source_of(plan: &ReportPlan) -> Option<AnswerSource> {
    let id = report_id(plan.id)?;
    let label_id = mandi_reports::reports::catalog()
        .into_iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.title_id)?;
    let f = &plan.filters;
    let filters = SourceReportFilters {
        from: iso(f.from),
        to: iso(f.to),
        as_of: iso(f.as_of),
        party: f.party.as_ref().map(|hit| SourceParty {
            record: record_of(hit),
            person_key: None,
        }),
        seller: f.seller.as_ref().map(record_of),
        partner: f.partner.as_ref().map(record_of),
        garden: f.garden.as_ref().map(record_of),
        city: f.city.as_ref().map(record_of_other),
        adda: f.adda.as_ref().map(record_of_other),
        trip_view: (plan.id == "R33").then_some(TripView::Trip),
        ..SourceReportFilters::default()
    };
    Some(AnswerSource::Report {
        result_id: String::new(),
        label_id,
        id,
        filters,
    })
}

/// The buttons of a plan: the report, and the second one where the plan gives it.
pub(super) fn sources_of(plan: &ReportPlan) -> Vec<AnswerSource> {
    let mut sources: Vec<AnswerSource> = source_of(plan).into_iter().collect();
    if let Some(also) = plan.also.as_deref() {
        sources.extend(source_of(also));
    }
    sources
}

// ---- the steps -----------------------------------------------------------------------------------------

fn text_arg(name: &str, value: &str) -> MessageArg {
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Text {
            text: value.to_owned(),
        },
    }
}

fn names_arg(name: &str, ur: String, en: String) -> MessageArg {
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Names {
            names: Names { ur, en },
        },
    }
}

fn step(id: &str, args: Vec<MessageArg>) -> HowStep {
    HowStep {
        id: id.to_owned(),
        args,
    }
}

/// The word of a role for the step line (`assistant-how-set-party` and `-set-record` in the panel).
fn role_word(role: PartyRole) -> &'static str {
    match role {
        PartyRole::Buyer => "buyer",
        PartyRole::Seller => "seller",
        PartyRole::Partner => "partner",
        PartyRole::Garden => "garden",
        PartyRole::Trip => "trip",
        PartyRole::Khata => "khata",
        PartyRole::Staff => "staff",
    }
}

fn record_word(kind: RecordKind) -> &'static str {
    match kind {
        RecordKind::ExpenseHead => "head",
        RecordKind::City => "city",
        RecordKind::Adda => "adda",
    }
}

/// "set the party" for a record with a code, "set the record" for one with none (a partner, an
/// expense head).
fn record_step(role: &str, ur: String, en: String, code: &str) -> HowStep {
    if code.is_empty() {
        step(
            "assistant-how-set-record",
            vec![text_arg("role", role), names_arg("name", ur, en)],
        )
    } else {
        step(
            "assistant-how-set-party",
            vec![
                text_arg("role", role),
                names_arg("party", ur, en),
                text_arg("code", code),
            ],
        )
    }
}

fn party_step(hit: &PartyHit) -> HowStep {
    let with_tag = |name: &str| match &hit.tag {
        Some(tag) => format!("{name} {tag}"),
        None => name.to_owned(),
    };
    record_step(
        role_word(hit.role),
        with_tag(&hit.name_ur),
        with_tag(&hit.name_en),
        &hit.code,
    )
}

fn other_step(hit: &RecordHit) -> HowStep {
    record_step(
        record_word(hit.kind),
        hit.name_ur.clone(),
        hit.name_en.clone(),
        &hit.code,
    )
}

/// The steps of a plan, by hand: open the report, choose the party or the record, set the day or the
/// dates, find the row of a trip or an expense head, and (for a print) press F9. They are made from the same plan as the button.
pub(super) fn steps_of(plan: &ReportPlan, press_print: bool) -> Vec<HowStep> {
    let f = &plan.filters;
    let mut steps = vec![step(
        "assistant-how-open-report",
        vec![text_arg("report", plan.id)],
    )];
    let who = [&f.party, &f.seller, &f.partner, &f.garden];
    for hit in who.into_iter().flatten() {
        steps.push(party_step(hit));
    }
    for hit in [&f.city, &f.adda].into_iter().flatten() {
        steps.push(other_step(hit));
    }
    let day = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    match (f.as_of, f.from, f.to) {
        (Some(d), _, _) => steps.push(step(
            "assistant-how-set-day",
            vec![text_arg("day", &day(d))],
        )),
        (None, Some(from), Some(to)) if from == to => {
            steps.push(step(
                "assistant-how-set-day",
                vec![text_arg("day", &day(from))],
            ));
        }
        (None, Some(from), Some(to)) => steps.push(step(
            "assistant-how-set-dates",
            vec![text_arg("from", &day(from)), text_arg("to", &day(to))],
        )),
        _ => {}
    }
    // A trip or an expense head has no filter on its report: the row is found by its name.
    let rows = [
        f.trip.as_ref().map(|h| (&h.name_ur, &h.name_en)),
        f.head.as_ref().map(|h| (&h.name_ur, &h.name_en)),
    ];
    for (ur, en) in rows.into_iter().flatten() {
        steps.push(step(
            "assistant-how-find-row",
            vec![names_arg("name", ur.clone(), en.clone())],
        ));
    }
    if press_print {
        steps.push(step("assistant-how-press-print", Vec::new()));
    }
    steps
}

// ---- the screens ---------------------------------------------------------------------------------------

/// The entry screen of the answer schema by its name (`EntryScreenName::ALL`).
pub(super) fn entry_screen(name: &str) -> Option<EntryScreen> {
    Some(match name {
        "bikri" => EntryScreen::Bikri,
        "wasooli" => EntryScreen::Wasooli,
        "adaigi" => EntryScreen::Adaigi,
        "peshgi" => EntryScreen::Peshgi,
        "kharcha" => EntryScreen::Kharcha,
        "udhar_raqam" => EntryScreen::UdharRaqam,
        "alag_khata" => EntryScreen::AlagKhata,
        "sarmaya" => EntryScreen::Sarmaya,
        "phal" => EntryScreen::Phal,
        "general" => EntryScreen::General,
        "account" => EntryScreen::Account,
        "dashboard" => EntryScreen::Dashboard,
        "settings" => EntryScreen::Settings,
        "masters" => EntryScreen::Masters,
        "accounts" => EntryScreen::Accounts,
        "commission_rules" => EntryScreen::CommissionRules,
        "today_sales" => EntryScreen::TodaySales,
        "unsold_stock" => EntryScreen::UnsoldStock,
        "pukhta_khata" => EntryScreen::PukhtaKhata,
        "collection_list" => EntryScreen::CollectionList,
        "cash_book" => EntryScreen::CashBook,
        "gardens" => EntryScreen::Gardens,
        "market_trips" => EntryScreen::MarketTrips,
        "partners" => EntryScreen::Partners,
        "month_close" => EntryScreen::MonthClose,
        "day_close" => EntryScreen::DayClose,
        "reports" => EntryScreen::Reports,
        "dead_amount" => EntryScreen::DeadAmount,
        "analytics" => EntryScreen::Analytics,
        "backup" => EntryScreen::Backup,
        _ => return None,
    })
}

/// The route that opens a screen as it is, with no party chosen.
pub(super) fn route_of(name: &str) -> Option<EntryRoute> {
    entry_screen(name).map(|screen| EntryRoute {
        screen,
        party: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_answer::EntryScreenName;

    #[test]
    fn every_screen_of_the_answer_schema_has_a_route() {
        for name in EntryScreenName::ALL {
            assert!(route_of(name).is_some(), "{name}");
        }
        assert!(route_of("no_such_screen").is_none());
    }

    /// R33 opens in the trip layout and carries the trip's days; the trip itself is named in the
    /// steps only (its screen has no one-trip filter, lead ruling of 04-11-2026).
    #[test]
    fn a_trip_report_has_the_trip_layout_and_a_step_that_names_the_row() {
        use super::super::{PartyHit, PartyRole, PlanFilters, ReportPlan};
        let day = |d| chrono::NaiveDate::from_ymd_opt(2026, 8, d);
        let plan = ReportPlan {
            id: "R33",
            filters: PlanFilters {
                trip: Some(PartyHit {
                    key: "t2".into(),
                    id: 2,
                    role: PartyRole::Trip,
                    code: "0070".into(),
                    name_ur: "سفر 0070".into(),
                    name_en: "0070 14-08-2026".into(),
                    tag: None,
                    pukhta: false,
                }),
                from: day(14),
                to: day(20),
                ..PlanFilters::default()
            },
            also: None,
            assumed_period: false,
        };
        match source_of(&plan).expect("a button") {
            AnswerSource::Report { id, filters, .. } => {
                assert_eq!(id, ReportId::R33);
                assert_eq!(filters.trip_view, Some(TripView::Trip));
                assert_eq!(
                    (filters.from.as_str(), filters.to.as_str()),
                    ("2026-08-14", "2026-08-20")
                );
            }
            other => panic!("{other:?}"),
        }
        let ids: Vec<String> = steps_of(&plan, false).into_iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            [
                "assistant-how-open-report",
                "assistant-how-set-dates",
                "assistant-how-find-row"
            ]
        );
    }

    #[test]
    fn every_report_the_plans_use_is_in_the_catalogue() {
        let catalogue = mandi_reports::reports::catalog();
        for code in [
            "R2", "R5", "R6", "R8", "R10", "R11", "R12", "R13", "R14", "R18", "R20", "R21", "R24",
            "R29", "R31", "R33", "R34", "R35", "R36", "R37",
        ] {
            let id = report_id(code).unwrap_or_else(|| panic!("{code} is not a report"));
            assert!(catalogue.iter().any(|e| e.id == id), "{code}");
        }
        assert!(report_id("R999").is_none());
    }
}
