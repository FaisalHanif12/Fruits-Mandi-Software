//! `app_state{what:"audit"}`: the audit rows of one بکری or one account, newest first (PRD FR-21.3,
//! plan 55 §11.1 R4).
//!
//! It reads the audit viewer's own list (`mandi_reports::audit::list`) for one record. For each row
//! the model gets the day, what was done (`action`) to what (`entity`), and the fields that changed.
//! A changed money or number field gives its old and new value; a changed text field gives only its
//! name, because a text can be a phone, an address or a note, which PRD FR-7.83 keeps back. The
//! reason the Owner typed, the user and the old and new JSON as stored are never given. While the
//! partner section is locked the audit page hides a partner's old and new values (PRD FR-7.82).
//!
//! Example (worked day): `app_state{what:"audit", bikri:1}` gives the row that created بکری 1 on
//! 10-10-26.

use mandi_reports::audit::{AuditChange, AuditFilters, AuditListInput, AuditRow, AuditValue, list};
use mandi_services::bikri;
use mandi_services::core::bikri::BikriFindInput;
use serde_json::Value;

use super::records::named;
use super::report::reports_stop;
use super::state::on_screen;
use super::{Draft, Env, State, Stop, ToolError};
use crate::answer::ScreenRoute;
use crate::results::{Money, Node, Val};

/// Most audit rows given.
const ROWS_CAP: i64 = 12;
/// Most changed fields given for one row.
const CHANGES_CAP: usize = 12;

fn bad(code: &str, field: &str, detail: &str) -> Stop {
    Stop::Error(ToolError::new(code, field, detail))
}

/// One side of a change: money and numbers as figures, a flag as a flag, a text only as "text".
fn value_node(value: &AuditValue) -> Node {
    match value {
        AuditValue::Null => Node::Null,
        AuditValue::Bool { value } => Node::Bool(*value),
        AuditValue::Number { value } => Node::Val(Val::Count(*value)),
        AuditValue::Money { paisa } => Node::Val(Val::Money(Money::plain(*paisa))),
        AuditValue::Text { .. } | AuditValue::Hidden => Node::Str("text".to_owned()),
    }
}

fn change_node(change: &AuditChange) -> Node {
    Node::obj(vec![
        ("field", Node::Str(change.path.clone())),
        ("old", value_node(&change.old)),
        ("new", value_node(&change.new)),
    ])
}

fn row_node(row: &AuditRow) -> Node {
    let cut = row.changes.len() > CHANGES_CAP || row.changes_truncated;
    Node::obj(vec![
        ("date", Node::Val(Val::Date(row.date.clone()))),
        ("action", Node::Str(row.action.clone())),
        ("entity", Node::Str(row.entity.clone())),
        (
            "changes",
            Node::List(
                row.changes
                    .iter()
                    .take(CHANGES_CAP)
                    .map(change_node)
                    .collect(),
            ),
        ),
        ("changes_cut", Node::Bool(cut)),
    ])
}

/// The tool, for `what: "audit"`: the rows of one party's account, of one بکری, or, with neither,
/// the newest rows of the whole log (the Owner asks what changed lately). A party and a number
/// together are a mistake.
pub(super) fn run(
    env: &Env<'_>,
    state: &State,
    args: &Value,
    party: Option<&str>,
    bikri_number: Option<i64>,
) -> Result<Draft, Stop> {
    // The record asked for: its entity word and id, or none for the whole log.
    let (entity, entity_id): (&str, Option<i64>) = match (party, bikri_number) {
        (Some(_), Some(_)) => {
            return Err(bad(
                "ERR_REQUIRED",
                "party",
                "give a party or a bikri number, not both; give neither for the newest changes",
            ));
        }
        (None, None) => ("all", None),
        (Some(text), None) => {
            let (id, _) = named(env, state, text, "party", &["a"])?;
            ("account", Some(id))
        }
        (None, Some(number)) => {
            let found = bikri::find(
                env.ctx,
                &BikriFindInput {
                    number: Some(number),
                    ..BikriFindInput::default()
                },
            )
            .map_err(super::service_stop)?
            .into_iter()
            .find(|r| r.number == number)
            .ok_or_else(|| bad("ERR_NOT_FOUND", "bikri", "no such بکری"))?;
            ("bikri", Some(found.id))
        }
    };
    let mut page = list(
        env.ctx.db(),
        &AuditListInput {
            filters: AuditFilters {
                entity: if entity_id.is_some() {
                    entity.to_owned()
                } else {
                    String::new()
                },
                entity_id,
                ..AuditFilters::default()
            },
            offset: 0,
            limit: ROWS_CAP,
            sort: None,
            group_by_month: false,
        },
    )
    .map_err(reports_stop)?;
    // A partner's events carry his shares: behind the partner code (PRD FR-7.82).
    let has_partner_rows = page.rows.iter().any(|r| r.entity == "partner");
    if env.partner_locked {
        page.hide_partners();
    }
    let total = page.total_count;
    let more = total > i64::try_from(page.rows.len()).unwrap_or(i64::MAX);
    let rows = page
        .rows
        .iter()
        .map(|r| (format!("au{}", r.id), row_node(r)))
        .collect();
    let node = Node::obj(vec![
        ("what", Node::Str("audit".to_owned())),
        ("entity", Node::Str(entity.to_owned())),
        ("rows", Node::Keyed(rows)),
        ("count", Node::Val(Val::Count(total))),
        ("more", Node::Bool(more)),
    ]);
    let mut draft = on_screen(
        Draft::new("app_state", args.clone(), node),
        ScreenRoute::Audit,
    );
    draft.partner_parts_locked = env.partner_locked && has_partner_rows;
    draft.partner_data = !env.partner_locked && has_partner_rows;
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_services::core::money::Paisa;

    #[test]
    fn a_text_value_never_leaves_only_its_field_name_does() {
        let secret = AuditValue::Text {
            text: "0300-1111111".to_owned(),
        };
        assert_eq!(value_node(&secret), Node::Str("text".to_owned()));
        assert_eq!(
            value_node(&AuditValue::Hidden),
            Node::Str("text".to_owned())
        );
        let paisa = Paisa::from_rupees(500).unwrap();
        assert_eq!(
            value_node(&AuditValue::Money { paisa }),
            Node::Val(Val::Money(Money::plain(paisa)))
        );
        assert_eq!(
            value_node(&AuditValue::Number { value: 3 }),
            Node::Val(Val::Count(3))
        );
        assert_eq!(value_node(&AuditValue::Null), Node::Null);
    }
}
