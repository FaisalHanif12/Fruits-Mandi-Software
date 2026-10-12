//! Commission rules and which rule wins (PRD 11, plan 09). Rules are data: a change here never
//! changes a saved sale line (PRD FR-11.6). Nothing is ever deleted; a rule is made inactive.

use mandi_services::core::rules::{
    RuleInput, RuleRow, RuleSaved, RuleTest, RuleTestInput, RulesGetInput, RulesListInput,
    RulesSetActiveInput,
};
use mandi_services::{auth, rules};

use super::run;
use crate::error::UiError;

/// The rules that match a filter. Each row says the level it decides at (2 to 6).
#[tauri::command]
pub async fn rules_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RulesListInput,
) -> Result<Vec<RuleRow>, UiError> {
    run(app, "rules_list", move |ctx| {
        auth::require(ctx)?;
        rules::list(ctx, &input.filter)
    })
    .await
}

/// One rule, for the edit form. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn rules_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RulesGetInput,
) -> Result<RuleRow, UiError> {
    run(app, "rules_get", move |ctx| {
        auth::require(ctx)?;
        rules::get(ctx, input.id)
    })
    .await
}

/// Adds a rule (`id` null) or edits one (`id` and the `version` read). One transaction: the
/// rule and its audit row. Errors: `ERR_RULE_NOT_ALLOWED` (`method`, `charge_type`,
/// `account_id`, `item_group`), `ERR_PERCENT_OVER_100`, `ERR_MUST_BE_MORE_THAN_ZERO` (`value`,
/// per نگ), the money and date codes, `ERR_DATE_RANGE` (`to_date`), `ERR_NOT_FOUND` (`account_id`,
/// `item_id`, `id`), `ERR_VERSION_CHANGED`. Returns the rules whose dates overlap (PRD FR-11.5).
#[tauri::command]
pub async fn rules_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RuleInput,
) -> Result<RuleSaved, UiError> {
    run(app, "rules_save", move |ctx| {
        let actor = auth::require(ctx)?;
        rules::save(ctx, &actor, &input)
    })
    .await
}

/// Makes a rule active or inactive. Errors: `ERR_NOT_FOUND`. Returns the overlap warnings.
#[tauri::command]
pub async fn rules_set_active<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RulesSetActiveInput,
) -> Result<RuleSaved, UiError> {
    run(app, "rules_set_active", move |ctx| {
        let actor = auth::require(ctx)?;
        rules::set_active(ctx, &actor, input.id, input.is_active)
    })
    .await
}

/// "Test a rule" (PRD FR-11.8): for a seller, a buyer, an item and a day, the winner of each
/// charge, its level, the levels skipped, and the amount on a quantity and rate, with the
/// working. Nothing is saved.
#[tauri::command]
pub async fn rules_test<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RuleTestInput,
) -> Result<RuleTest, UiError> {
    run(app, "rules_test", move |ctx| {
        auth::require(ctx)?;
        rules::test(ctx, &input)
    })
    .await
}
