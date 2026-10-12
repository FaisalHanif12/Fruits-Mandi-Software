//! Checks every field of an account and of an opening line (plan 07, PRD 10.2, FR-10.3, FR-10.9).
//!
//! Layer 2 of Code Standards 4.1. All errors of one input come back together with their field
//! paths. Whether a code is free, whether a record exists and whether the spelling is
//! confirmed are checked by `services::accounts` with these results.

use super::{AccountInput, AccountType, FIRST_SERIAL, LAST_SERIAL};
use crate::dates::BusinessDate;
use crate::error::{ErrorCode, FieldError};
use crate::masters::{ColValue, NameTexts, Reference};
use crate::money::Paisa;
use crate::text;
use crate::validate::{self, FieldErrors};

/// The largest collection weekday (0 = Monday, 6 = Sunday).
pub const MAX_WEEKDAY: i64 = 6;

/// An opening amount typed on an account form or a line of the openings grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpeningRequest {
    /// The opening date. `None` when there is no opening amount.
    pub date: Option<BusinessDate>,
    /// The opening debit. Zero if none.
    pub dr: Paisa,
    /// The opening credit. Zero if none.
    pub cr: Paisa,
    /// The debit counts as an advance (sellers only).
    pub is_advance: bool,
}

impl OpeningRequest {
    /// No opening at all.
    pub const NONE: OpeningRequest = OpeningRequest {
        date: None,
        dr: Paisa::ZERO,
        cr: Paisa::ZERO,
        is_advance: false,
    };

    /// True if there is an amount on either side.
    pub fn has_amount(&self) -> bool {
        self.dr != Paisa::ZERO || self.cr != Paisa::ZERO
    }
}

/// An account checked field by field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAccount {
    /// The row to edit, or `None` to add.
    pub id: Option<i64>,
    /// The version the edit was made on.
    pub version: Option<i64>,
    /// The kind of account.
    pub account_type: AccountType,
    /// The control, for type `other`.
    pub control_id: Option<i64>,
    /// The full code the user typed (digits), or `None` for "give me the next one".
    pub typed_code: Option<String>,
    /// The columns to write, except the code, serial, control and the opening columns (names and
    /// search columns are in here).
    pub fields: Vec<(&'static str, ColValue)>,
    /// The two names, for the spelling check.
    pub names: NameTexts,
    /// The father or firm names (Urdu, English), for the spelling check. Either may be empty.
    pub father: (String, String),
    /// The words the user confirmed.
    pub confirmed_rules_words: Vec<String>,
    /// Records that must exist.
    pub references: Vec<Reference>,
    /// The opening that was typed.
    pub opening: OpeningRequest,
}

fn error(code: ErrorCode, field: &str) -> FieldError {
    FieldError::new(code, field)
}

/// An optional name in one script: empty is fine, otherwise the same checks as a name.
fn optional_name(
    errors: &mut FieldErrors,
    field: &str,
    input: &str,
    check: fn(&str, &str) -> Result<String, FieldError>,
) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    errors.take(check(field, input)).unwrap_or_default()
}

fn reference(
    errors: &mut FieldErrors,
    field: &'static str,
    table: &'static str,
    id: Option<i64>,
    out: &mut Vec<Reference>,
) {
    if let Some(id) = id {
        if id > 0 {
            out.push(Reference { field, table, id });
        } else {
            errors.push(error(ErrorCode::ErrNotFound, field));
        }
    }
}

/// Checks an opening: the amounts, the side rule, the date and the advance tick.
///
/// `prefix` goes in front of the field names (`""` on the account form, `"lines[2]."` on a
/// line of the grid). The date is only checked when there is an amount to post.
///
/// # Errors
/// `ERR_OPENING_BOTH_SIDES` (`opening_cr`), `ERR_REQUIRED` and the date codes on `opening_date`,
/// the money codes on `opening_dr` and `opening_cr`, `ERR_ADVANCE_NOT_SELLER` on
/// `opening_is_advance`.
// Eight plain arguments: the text of three fields, two flags and the day window. A struct would only
// move the same eight values into a type used in two places.
#[allow(clippy::too_many_arguments)]
pub fn opening_request(
    prefix: &str,
    opening_date: &str,
    opening_dr: &str,
    opening_cr: &str,
    is_advance: bool,
    is_seller: bool,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<OpeningRequest, Vec<FieldError>> {
    let mut errors = FieldErrors::default();
    let f = |name: &str| format!("{prefix}{name}");
    let dr = errors.take(validate::money_optional(&f("opening_dr"), opening_dr));
    let cr = errors.take(validate::money_optional(&f("opening_cr"), opening_cr));
    // A typed 0 is the same as nothing.
    let dr = dr.flatten().unwrap_or(Paisa::ZERO);
    let cr = cr.flatten().unwrap_or(Paisa::ZERO);
    if dr != Paisa::ZERO && cr != Paisa::ZERO {
        errors.push(error(ErrorCode::ErrOpeningBothSides, &f("opening_cr")));
    }
    let has_amount = dr != Paisa::ZERO || cr != Paisa::ZERO;
    if is_advance && !(is_seller && dr != Paisa::ZERO) {
        errors.push(error(
            ErrorCode::ErrAdvanceNotSeller,
            &f("opening_is_advance"),
        ));
    }
    let date = if has_amount {
        if opening_date.trim().is_empty() {
            errors.push(error(ErrorCode::ErrRequired, &f("opening_date")));
            None
        } else {
            errors.take(validate::posting_date(
                &f("opening_date"),
                opening_date,
                closed_up_to,
                today,
            ))
        }
    } else {
        None
    };
    errors.finish()?;
    Ok(OpeningRequest {
        date,
        dr,
        cr,
        is_advance: is_advance && has_amount,
    })
}

/// The serial inside a full account code, if the code is the control code plus 4 digits.
///
/// - `ERR_CODE_INVALID` if it is not digits, or does not start with the control code, or has
///   the wrong length (PRD FR-10.2).
/// - `ERR_CODE_RESERVED` for serial 0000 and 9000 to 9999, which belong to system accounts
///   (PRD FR-10.3).
///
/// ```
/// use mandi_core::accounts::account_code_for;
/// assert_eq!(account_code_for("code", "0304", "03040003").unwrap(), 3);
/// assert!(account_code_for("code", "0304", "03049999").is_err());
/// assert!(account_code_for("code", "0304", "03110003").is_err());
/// ```
pub fn account_code_for(field: &str, control_code: &str, typed: &str) -> Result<u32, FieldError> {
    let digits: String = typed.trim().chars().collect();
    let serial_text = digits
        .strip_prefix(control_code)
        .filter(|rest| rest.len() == 4 && rest.bytes().all(|b| b.is_ascii_digit()));
    let Some(serial_text) = serial_text else {
        return Err(error(ErrorCode::ErrCodeInvalid, field));
    };
    let serial: u32 = serial_text
        .parse()
        .map_err(|_| error(ErrorCode::ErrCodeInvalid, field))?;
    if !(FIRST_SERIAL..=LAST_SERIAL).contains(&serial) {
        return Err(error(ErrorCode::ErrCodeReserved, field));
    }
    Ok(serial)
}

/// Checks the input of one account.
///
/// # Errors
/// Every wrong field, each with its code and path, all together.
pub fn prepare_account(
    input: &AccountInput,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<PreparedAccount, Vec<FieldError>> {
    let mut errors = FieldErrors::default();
    let version = match (input.id, input.version) {
        (Some(_), None) => {
            errors.push(error(ErrorCode::ErrRequired, "version"));
            None
        }
        (Some(_), Some(v)) => Some(v),
        (None, _) => None,
    };
    // The control is only typed for `other`.
    let control_id = if input.account_type == AccountType::Other {
        if input.control_id.is_none() {
            errors.push(error(ErrorCode::ErrRequired, "control_id"));
        }
        input.control_id
    } else {
        None
    };
    let typed_code = {
        let digits = input.code.trim();
        if digits.is_empty() {
            None
        } else if digits.bytes().all(|b| b.is_ascii_digit()) && digits.len() <= 12 {
            Some(digits.to_owned())
        } else {
            errors.push(error(ErrorCode::ErrCodeInvalid, "code"));
            None
        }
    };
    let ur = errors.take(validate::name_ur("name_ur", &input.name_ur));
    let en = errors.take(validate::name_en("name_en", &input.name_en));
    let father_ur = optional_name(
        &mut errors,
        "father_or_firm_ur",
        &input.father_or_firm_ur,
        validate::name_ur,
    );
    let father_en = optional_name(
        &mut errors,
        "father_or_firm_en",
        &input.father_or_firm_en,
        validate::name_en,
    );
    let phone1 = errors.take(validate::phone_optional("phone1", &input.phone1));
    let phone2 = errors.take(validate::phone_optional("phone2", &input.phone2));
    let address = errors.take(validate::free_text("address", &input.address));
    let note = errors.take(validate::free_text("note", &input.note));
    let credit_limit = errors.take(validate::money_optional(
        "credit_limit",
        &input.credit_limit,
    ));
    if let Some(day) = input.collection_weekday
        && !(0..=MAX_WEEKDAY).contains(&day)
    {
        errors.push(
            error(ErrorCode::ErrOutOfRange, "collection_weekday")
                .with("min", crate::error::ErrorValue::Number(0))
                .with("max", crate::error::ErrorValue::Number(MAX_WEEKDAY)),
        );
    }
    let mut references = Vec::new();
    reference(
        &mut errors,
        "city_id",
        "cities",
        input.city_id,
        &mut references,
    );
    reference(
        &mut errors,
        "adda_id",
        "addas",
        input.adda_id,
        &mut references,
    );
    reference(
        &mut errors,
        "linked_account_id",
        "accounts",
        input.linked_account_id,
        &mut references,
    );
    reference(
        &mut errors,
        "default_marka_id",
        "markas",
        input.default_marka_id,
        &mut references,
    );
    reference(
        &mut errors,
        "collector_staff_id",
        "staff",
        input.collector_staff_id,
        &mut references,
    );
    if input.id.is_some() && input.linked_account_id == input.id {
        errors.push(error(ErrorCode::ErrNotFound, "linked_account_id"));
    }
    let opening = match opening_request(
        "",
        &input.opening_date,
        &input.opening_dr,
        &input.opening_cr,
        input.opening_is_advance,
        input.account_type == AccountType::Seller,
        closed_up_to,
        today,
    ) {
        Ok(opening) => opening,
        Err(found) => {
            for e in found {
                errors.push(e);
            }
            OpeningRequest::NONE
        }
    };
    errors.finish()?;
    let (
        Some(ur),
        Some(en),
        Some(phone1),
        Some(phone2),
        Some(address),
        Some(note),
        Some(credit_limit),
    ) = (ur, en, phone1, phone2, address, note, credit_limit)
    else {
        return Err(vec![error(ErrorCode::ErrInternal, "account")]);
    };
    let mut fields: Vec<(&'static str, ColValue)> = vec![
        ("name_ur", ColValue::Text(ur.clone())),
        ("name_en", ColValue::Text(en.clone())),
        ("search_ur", ColValue::Text(text::search_ur(&ur))),
        ("search_en", ColValue::Text(text::search_en(&en))),
        (
            "search_phonetic",
            ColValue::Text(text::search_phonetic(&en)),
        ),
        ("father_or_firm_ur", optional(&father_ur)),
        ("father_or_firm_en", optional(&father_en)),
        ("city_id", id_or_null(input.city_id)),
        ("adda_id", id_or_null(input.adda_id)),
        ("phone1", phone_or_null(phone1)),
        ("phone2", phone_or_null(phone2)),
        ("address", optional(&address)),
        ("linked_account_id", id_or_null(input.linked_account_id)),
        (
            "credit_allowed",
            ColValue::Int(i64::from(input.credit_allowed)),
        ),
        (
            "credit_limit_paisa",
            credit_limit
                .filter(|p| *p != Paisa::ZERO)
                .map_or(ColValue::Null, |p| ColValue::Int(p.as_paisa())),
        ),
        ("default_marka_id", id_or_null(input.default_marka_id)),
        ("collector_staff_id", id_or_null(input.collector_staff_id)),
        ("collection_weekday", id_or_null(input.collection_weekday)),
        ("is_active", ColValue::Int(i64::from(input.is_active))),
        ("note", optional(&note)),
    ];
    // Keep the order stable for the audit row.
    fields.shrink_to_fit();
    Ok(PreparedAccount {
        id: input.id,
        version,
        account_type: input.account_type,
        control_id,
        typed_code,
        fields,
        names: NameTexts {
            field_ur: "name_ur",
            name_ur: ur,
            field_en: "name_en",
            name_en: en,
        },
        father: (father_ur, father_en),
        confirmed_rules_words: input.confirmed_rules_words.clone(),
        references,
        opening,
    })
}

fn optional(value: &str) -> ColValue {
    if value.is_empty() {
        ColValue::Null
    } else {
        ColValue::Text(value.to_owned())
    }
}

fn id_or_null(value: Option<i64>) -> ColValue {
    value.map_or(ColValue::Null, ColValue::Int)
}

fn phone_or_null(value: Option<String>) -> ColValue {
    value.map_or(ColValue::Null, ColValue::Text)
}
