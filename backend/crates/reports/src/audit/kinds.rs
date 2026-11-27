//! Which kind of event an audit row is (PRD FR-21.3, v1.61, plan 100): the filter by kind and the
//! filter حفاظتی واقعات. The SQL of the filter and the Rust of the row use the same lists, and a
//! test checks that they agree.

use serde_json::Value;

use super::AuditEventKind;
use crate::reports::common::Conds;

/// The actions of each kind that need no look at the values.
const EDIT: [&str; 5] = [
    "edit",
    "update",
    "change_split",
    "set_sell_date",
    "set_shares",
];
const CANCEL: [&str; 4] = [
    "cancel",
    "write_off_undo",
    "mixed_undo",
    "month_close_reverse",
];
const REOPEN: [&str; 2] = ["day_reopen", "day_fix_open"];
const WRONG_SIGN: [&str; 7] = [
    "day_reopen_sign_wrong",
    "day_fix_sign_wrong",
    "voucher_edit_sign_wrong",
    "owner_sign_change_sign_wrong",
    "garden_rent_sign_wrong",
    "openings_to_cash_sign_wrong",
    "cancel_sign_wrong",
];

/// The backup key's refusal says why under `refused`.
fn refused_sign(new_value: Option<&Value>) -> bool {
    new_value
        .and_then(|v| v.get("refused"))
        .and_then(Value::as_str)
        == Some("sign")
}

/// The kind of a row, or `None` when it is of no kind of the filter.
pub(super) fn kind_of(
    action: &str,
    entity: &str,
    new_value: Option<&Value>,
) -> Option<AuditEventKind> {
    let kind = match action {
        a if EDIT.contains(&a) => AuditEventKind::Edit,
        a if CANCEL.contains(&a) => AuditEventKind::Cancel,
        "reopen" if entity == "garden" => AuditEventKind::Cancel,
        a if REOPEN.contains(&a) => AuditEventKind::Reopen,
        "partner_code_failed" => AuditEventKind::WrongCode,
        a if WRONG_SIGN.contains(&a) => AuditEventKind::WrongSign,
        "reveal_refused" if refused_sign(new_value) => AuditEventKind::WrongSign,
        "reveal_refused" | "login_failed" => AuditEventKind::WrongPassword,
        "lockout_started" => AuditEventKind::Lockout,
        "closed_day_refused" => AuditEventKind::ClosedDay,
        _ => return None,
    };
    Some(kind)
}

/// The SQL condition of one kind. Constants only: nothing typed reaches it (Code Standards B14).
const fn sql_of(kind: AuditEventKind) -> &'static str {
    match kind {
        AuditEventKind::Edit => {
            "a.action IN ('edit', 'update', 'change_split', 'set_sell_date', 'set_shares')"
        }
        AuditEventKind::Cancel => {
            "(a.action IN ('cancel', 'write_off_undo', 'mixed_undo', 'month_close_reverse') \
              OR (a.action = 'reopen' AND a.entity = 'garden'))"
        }
        AuditEventKind::Reopen => "a.action IN ('day_reopen', 'day_fix_open')",
        AuditEventKind::WrongCode => "a.action = 'partner_code_failed'",
        AuditEventKind::WrongSign => {
            "(a.action IN ('day_reopen_sign_wrong', 'day_fix_sign_wrong', \
              'voucher_edit_sign_wrong', 'owner_sign_change_sign_wrong', \
              'garden_rent_sign_wrong', 'openings_to_cash_sign_wrong', 'cancel_sign_wrong') \
              OR (a.action = 'reveal_refused' AND json_valid(a.new_json) \
                  AND json_extract(a.new_json, '$.refused') = 'sign'))"
        }
        AuditEventKind::WrongPassword => {
            "(a.action = 'login_failed' OR (a.action = 'reveal_refused' AND NOT \
              (json_valid(a.new_json) AND json_extract(a.new_json, '$.refused') IS 'sign')))"
        }
        AuditEventKind::Lockout => "a.action = 'lockout_started'",
        AuditEventKind::ClosedDay => "a.action = 'closed_day_refused'",
    }
}

/// The security events as one condition.
const SECURITY_SQL: &str = "(a.action IN ('partner_code_failed', 'day_reopen_sign_wrong', \
     'day_fix_sign_wrong', 'voucher_edit_sign_wrong', 'owner_sign_change_sign_wrong', \
     'garden_rent_sign_wrong', 'openings_to_cash_sign_wrong', 'cancel_sign_wrong', \
     'reveal_refused', 'login_failed', 'lockout_started', 'closed_day_refused'))";

/// Adds the filter by kind and the security filter to the conditions.
pub(super) fn add(conds: &mut Conds, kind: Option<AuditEventKind>, security_only: bool) {
    if let Some(kind) = kind {
        conds.always(sql_of(kind));
    }
    if security_only {
        conds.always(SECURITY_SQL);
    }
}

#[cfg(test)]
mod tests {
    use mandi_db::rusqlite::{Connection, params};
    use serde_json::json;

    use super::*;

    /// Every action of the lists, the garden reopen and the two reasons of the backup key.
    fn samples() -> Vec<(String, String, Option<Value>)> {
        let mut out: Vec<(String, String, Option<Value>)> = EDIT
            .iter()
            .chain(CANCEL.iter())
            .chain(REOPEN.iter())
            .chain(WRONG_SIGN.iter())
            .chain(
                [
                    "partner_code_failed",
                    "login_failed",
                    "lockout_started",
                    "closed_day_refused",
                    "create",
                    "login",
                ]
                .iter(),
            )
            .map(|a| ((*a).to_owned(), "voucher".to_owned(), None))
            .collect();
        out.push(("reopen".to_owned(), "garden".to_owned(), None));
        out.push(("reopen".to_owned(), "other".to_owned(), None));
        for why in ["sign", "password", "locked"] {
            out.push((
                "reveal_refused".to_owned(),
                "backup_key".to_owned(),
                Some(json!({ "refused": why })),
            ));
        }
        out
    }

    #[test]
    fn the_sql_of_each_kind_agrees_with_the_rust() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE a (action TEXT, entity TEXT, new_json TEXT)")
            .unwrap();
        let all = [
            AuditEventKind::Edit,
            AuditEventKind::Cancel,
            AuditEventKind::Reopen,
            AuditEventKind::WrongCode,
            AuditEventKind::WrongSign,
            AuditEventKind::WrongPassword,
            AuditEventKind::Lockout,
            AuditEventKind::ClosedDay,
        ];
        for (action, entity, new_value) in samples() {
            conn.execute("DELETE FROM a", []).unwrap();
            conn.execute(
                "INSERT INTO a (action, entity, new_json) VALUES (?1, ?2, ?3)",
                params![action, entity, new_value.as_ref().map(Value::to_string)],
            )
            .unwrap();
            let rust = kind_of(&action, &entity, new_value.as_ref());
            let matches = |condition: &str| -> bool {
                let sql = format!("SELECT COUNT(*) FROM a WHERE {condition}");
                conn.query_row(&sql, [], |r| r.get::<_, i64>(0)).unwrap() == 1
            };
            for kind in all {
                assert_eq!(
                    matches(sql_of(kind)),
                    rust == Some(kind),
                    "{action} {entity} {kind:?}"
                );
            }
            assert_eq!(
                matches(SECURITY_SQL),
                rust.is_some_and(|k| AuditEventKind::SECURITY.contains(&k)),
                "{action} security"
            );
        }
    }
}
