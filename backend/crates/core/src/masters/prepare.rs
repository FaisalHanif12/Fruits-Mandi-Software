//! Checks every field of a master and turns the input into the columns to write (plan 06).
//!
//! Layer 2 of Code Standards 4.1: the backend treats everything the screen sends as untrusted.
//! All errors of one input come back together with their field paths. Nothing here reads the
//! database: whether a code is free, whether a referenced record exists and whether the
//! spelling is confirmed are checked by `services::masters` with these results.

use super::{
    AddaInput, ExpenseHeadInput, ItemInput, MarkaInput, MasterInput, MasterKind, NarrationInput,
    PackingInput, StaffInput,
};
use crate::error::{ErrorCode, FieldError};
use crate::text;
use crate::validate::{self, FieldErrors};

/// The longest master code, in digits.
pub const MAX_CODE_DIGITS: usize = 10;
/// The voucher types a standard narration can be offered in (Architecture 15.7 `vouchers.type`).
pub const NARRATION_USED_IN: [&str; 9] = [
    "receipt",
    "payment",
    "advance",
    "shop_expense",
    "personal_expense",
    "personal_return",
    "money_lent",
    "money_lent_return",
    "general",
];
/// The control under which expense heads make their accounts (PRD FR-10.4, 0501xxxx).
pub const EXPENSE_CONTROL: &str = "0501";

/// One value to write to a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColValue {
    /// Text.
    Text(String),
    /// A whole number.
    Int(i64),
    /// NULL.
    Null,
}

impl ColValue {
    fn text(value: &str) -> ColValue {
        ColValue::Text(value.to_owned())
    }

    fn optional_text(value: Option<String>) -> ColValue {
        value.map_or(ColValue::Null, ColValue::Text)
    }

    fn optional_id(value: Option<i64>) -> ColValue {
        value.map_or(ColValue::Null, ColValue::Int)
    }
}

/// The two names of a record and the fields they belong to, for the spelling check (PRD FR-7.66).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameTexts {
    /// Field path of the Urdu name.
    pub field_ur: &'static str,
    /// The cleaned Urdu name.
    pub name_ur: String,
    /// Field path of the English name.
    pub field_en: &'static str,
    /// The cleaned English name.
    pub name_en: String,
}

/// A record the input points at, which must exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// Field path of the id, for the error.
    pub field: &'static str,
    /// Its table (a fixed name, never from input).
    pub table: &'static str,
    /// Its id.
    pub id: i64,
}

/// The account that a master makes (expense head, bank account), with its search columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountToMake {
    /// The control code the account goes under (`0501` or `0102`).
    pub control_code: &'static str,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `search_ur`.
    pub search_ur: String,
    /// `search_en`.
    pub search_en: String,
    /// `search_phonetic`.
    pub search_phonetic: String,
}

impl AccountToMake {
    fn new(control_code: &'static str, name_ur: &str, name_en: &str) -> AccountToMake {
        AccountToMake {
            control_code,
            name_ur: name_ur.to_owned(),
            name_en: name_en.to_owned(),
            search_ur: text::search_ur(name_ur),
            search_en: text::search_en(name_en),
            search_phonetic: text::search_phonetic(name_en),
        }
    }
}

/// A checked input: the columns to write and what the service still has to check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    /// The kind.
    pub kind: MasterKind,
    /// The row to edit, or `None` to add.
    pub id: Option<i64>,
    /// The version the edit was made on. Always `Some` when `id` is `Some`.
    pub version: Option<i64>,
    /// The code the user typed, or `None` for "give me the next one". Always `None` for kinds
    /// without a code.
    pub code: Option<String>,
    /// The columns to write, besides `code` and the account link. Names and the three search
    /// columns are in here.
    pub fields: Vec<(&'static str, ColValue)>,
    /// The two names, for the spelling check. `None` for marks and narrations (typed as they
    /// are, PRD FR-7.25, FR-7.66).
    pub names: Option<NameTexts>,
    /// The words the user confirmed.
    pub confirmed_rules_words: Vec<String>,
    /// Records that must exist.
    pub references: Vec<Reference>,
    /// The account this master makes or renames.
    pub account: Option<AccountToMake>,
}

fn error(code: ErrorCode, field: &str) -> FieldError {
    FieldError::new(code, field)
}

/// The code typed by the user: 1 to 10 digits. Empty gives `None`. An item, a packing or a staff
/// member must have a full code of its own type (`0040`, `0050`, `0060`; plan 32 R2): otherwise
/// `ERR_CODE_INVALID` on `code`.
fn master_code(errors: &mut FieldErrors, kind: MasterKind, input: &str) -> Option<String> {
    let cleaned: String = input
        .trim()
        .chars()
        .filter(|c| !text::is_invisible(*c))
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.chars().count() > MAX_CODE_DIGITS || !cleaned.chars().all(|c| c.is_ascii_digit()) {
        errors.push(error(ErrorCode::ErrFormatInvalid, "code"));
        return None;
    }
    if let Some(code_type) = kind.code_type()
        && errors
            .take(crate::codes::check_typed_code("code", &cleaned, code_type))
            .is_none()
    {
        return None;
    }
    Some(cleaned)
}

/// A value that must be one of a few words.
fn choice(errors: &mut FieldErrors, field: &str, input: &str, allowed: &[&str]) -> Option<String> {
    let cleaned = input.trim();
    if cleaned.is_empty() {
        errors.push(error(ErrorCode::ErrRequired, field));
        return None;
    }
    if allowed.contains(&cleaned) {
        Some(cleaned.to_owned())
    } else {
        errors.push(error(ErrorCode::ErrFormatInvalid, field));
        None
    }
}

/// Both names with their three search columns. Returns (Urdu, English).
fn names(errors: &mut FieldErrors, input_ur: &str, input_en: &str) -> Option<(String, String)> {
    let ur = errors.take(validate::name_ur("name_ur", input_ur));
    let en = errors.take(validate::name_en("name_en", input_en));
    Some((ur?, en?))
}

fn name_columns(name_ur: &str, name_en: &str) -> Vec<(&'static str, ColValue)> {
    vec![
        ("name_ur", ColValue::text(name_ur)),
        ("name_en", ColValue::text(name_en)),
        ("search_ur", ColValue::Text(text::search_ur(name_ur))),
        ("search_en", ColValue::Text(text::search_en(name_en))),
        (
            "search_phonetic",
            ColValue::Text(text::search_phonetic(name_en)),
        ),
    ]
}

/// A narration text: free text of up to 500 characters, in its own script.
fn narration_text(
    errors: &mut FieldErrors,
    field: &str,
    input: &str,
    urdu: bool,
) -> Option<String> {
    let cleaned = errors.take(validate::free_text(field, input))?;
    if cleaned.is_empty() {
        errors.push(error(ErrorCode::ErrRequired, field));
        return None;
    }
    let has_urdu = cleaned.chars().any(text::is_urdu_letter);
    let has_latin = cleaned.chars().any(text::is_latin_letter);
    let (ok, code) = if urdu {
        (has_urdu && !has_latin, ErrorCode::ErrNameUrScript)
    } else {
        (has_latin && !has_urdu, ErrorCode::ErrNameEnScript)
    };
    if ok {
        Some(cleaned)
    } else {
        errors.push(error(code, field));
        None
    }
}

/// An id that points at another record. Must be a real (positive) id.
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

/// An edit needs the version it was made on.
fn edit_version(errors: &mut FieldErrors, id: Option<i64>, version: Option<i64>) -> Option<i64> {
    match (id, version) {
        (Some(_), None) => {
            errors.push(error(ErrorCode::ErrRequired, "version"));
            None
        }
        (Some(_), Some(v)) => Some(v),
        (None, _) => None,
    }
}

/// Checks the input of one master.
///
/// # Errors
/// Every wrong field, each with its code and path (for example `ERR_NAME_UR_SCRIPT` on
/// `name_ur`, `ERR_FORMAT_INVALID` on `code`), all together.
pub fn prepare(input: &MasterInput) -> Result<Prepared, Vec<FieldError>> {
    let mut errors = FieldErrors::default();
    let prepared = match input {
        MasterInput::City(i) => simple_named(
            &mut errors,
            MasterKind::City,
            i.id,
            i.version,
            &i.code,
            &i.name_ur,
            &i.name_en,
            &i.confirmed_rules_words,
        ),
        MasterInput::Store(i) => simple_named(
            &mut errors,
            MasterKind::Store,
            i.id,
            i.version,
            &i.code,
            &i.name_ur,
            &i.name_en,
            &i.confirmed_rules_words,
        ),
        MasterInput::Item(i) => item(&mut errors, i),
        MasterInput::Packing(i) => packing(&mut errors, i),
        MasterInput::Adda(i) => adda(&mut errors, i),
        MasterInput::Marka(i) => marka(&mut errors, i),
        MasterInput::Narration(i) => narration(&mut errors, i),
        MasterInput::ExpenseHead(i) => expense_head(&mut errors, i),
        MasterInput::Staff(i) => staff(&mut errors, i),
    };
    errors.finish()?;
    prepared.ok_or_else(|| vec![error(ErrorCode::ErrInternal, "kind")])
}

/// The common shape: code, both names. City and store.
#[allow(clippy::too_many_arguments)]
fn simple_named(
    errors: &mut FieldErrors,
    kind: MasterKind,
    id: Option<i64>,
    version: Option<i64>,
    code: &str,
    name_ur: &str,
    name_en: &str,
    confirmed: &[String],
) -> Option<Prepared> {
    let version = edit_version(errors, id, version);
    let code = master_code(errors, kind, code);
    let (ur, en) = names(errors, name_ur, name_en)?;
    Some(Prepared {
        kind,
        id,
        version,
        code,
        fields: name_columns(&ur, &en),
        names: Some(NameTexts {
            field_ur: "name_ur",
            name_ur: ur,
            field_en: "name_en",
            name_en: en,
        }),
        confirmed_rules_words: confirmed.to_vec(),
        references: Vec::new(),
        account: None,
    })
}

fn item(errors: &mut FieldErrors, i: &ItemInput) -> Option<Prepared> {
    let mut base = simple_named(
        errors,
        MasterKind::Item,
        i.id,
        i.version,
        &i.code,
        &i.name_ur,
        &i.name_en,
        &i.confirmed_rules_words,
    );
    let group = choice(errors, "item_group", &i.item_group, &["fruit", "vegetable"]);
    let sold_by = choice(errors, "sold_by", &i.sold_by, &["unit", "weight"]);
    let mut references = Vec::new();
    reference(
        errors,
        "default_packing_id",
        "packings",
        i.default_packing_id,
        &mut references,
    );
    let (group, sold_by) = (group?, sold_by?);
    let base = base.as_mut()?;
    base.fields.push(("item_group", ColValue::Text(group)));
    base.fields.push((
        "default_packing_id",
        ColValue::optional_id(i.default_packing_id),
    ));
    base.fields.push(("sold_by", ColValue::Text(sold_by)));
    base.references = references;
    Some(base.clone())
}

fn packing(errors: &mut FieldErrors, i: &PackingInput) -> Option<Prepared> {
    let mut base = simple_named(
        errors,
        MasterKind::Packing,
        i.id,
        i.version,
        &i.code,
        &i.name_ur,
        &i.name_en,
        &i.confirmed_rules_words,
    );
    let blank = i
        .approx_kg
        .chars()
        .all(|c| c.is_whitespace() || text::is_ignorable_in_number(c));
    let grams = if blank {
        Some(None)
    } else {
        errors
            .take(validate::weight("approx_kg", &i.approx_kg))
            .map(|g| Some(g.get()))
    };
    let grams = grams?;
    let base = base.as_mut()?;
    base.fields
        .push(("approx_grams", grams.map_or(ColValue::Null, ColValue::Int)));
    Some(base.clone())
}

fn adda(errors: &mut FieldErrors, i: &AddaInput) -> Option<Prepared> {
    let mut base = simple_named(
        errors,
        MasterKind::Adda,
        i.id,
        i.version,
        &i.code,
        &i.name_ur,
        &i.name_en,
        &i.confirmed_rules_words,
    );
    let mut references = Vec::new();
    reference(errors, "city_id", "cities", i.city_id, &mut references);
    let base = base.as_mut()?;
    base.fields
        .push(("city_id", ColValue::optional_id(i.city_id)));
    base.references = references;
    Some(base.clone())
}

fn marka(errors: &mut FieldErrors, i: &MarkaInput) -> Option<Prepared> {
    let version = edit_version(errors, i.id, i.version);
    let code = master_code(errors, MasterKind::Marka, &i.code);
    let mark = errors.take(validate::person_name("mark_text", &i.mark_text));
    let mut references = Vec::new();
    reference(
        errors,
        "owner_account_id",
        "accounts",
        i.owner_account_id,
        &mut references,
    );
    Some(Prepared {
        kind: MasterKind::Marka,
        id: i.id,
        version,
        code,
        fields: vec![
            ("mark_text", ColValue::Text(mark?)),
            (
                "owner_account_id",
                ColValue::optional_id(i.owner_account_id),
            ),
        ],
        names: None,
        confirmed_rules_words: Vec::new(),
        references,
        account: None,
    })
}

fn narration(errors: &mut FieldErrors, i: &NarrationInput) -> Option<Prepared> {
    let version = edit_version(errors, i.id, i.version);
    let ur = narration_text(errors, "text_ur", &i.text_ur, true);
    let en = narration_text(errors, "text_en", &i.text_en, false);
    let mut used: Vec<&str> = Vec::new();
    for item in &i.used_in {
        let item = item.trim();
        match NARRATION_USED_IN.iter().find(|allowed| **allowed == item) {
            Some(known) => {
                if !used.contains(known) {
                    used.push(known);
                }
            }
            None => errors.push(error(ErrorCode::ErrFormatInvalid, "used_in")),
        }
    }
    // The stored order is the order of the list, so the same set is always the same text.
    used.sort_by_key(|u| NARRATION_USED_IN.iter().position(|a| a == u));
    let used_json = if used.is_empty() {
        ColValue::Null
    } else {
        let items: Vec<String> = used.iter().map(|u| format!("\"{u}\"")).collect();
        ColValue::Text(format!("[{}]", items.join(",")))
    };
    Some(Prepared {
        kind: MasterKind::Narration,
        id: i.id,
        version,
        code: None,
        fields: vec![
            ("text_ur", ColValue::Text(ur?)),
            ("text_en", ColValue::Text(en?)),
            ("used_in_json", used_json),
        ],
        names: None,
        confirmed_rules_words: Vec::new(),
        references: Vec::new(),
        account: None,
    })
}

fn expense_head(errors: &mut FieldErrors, i: &ExpenseHeadInput) -> Option<Prepared> {
    let mut base = simple_named(
        errors,
        MasterKind::ExpenseHead,
        i.id,
        i.version,
        &i.code,
        &i.name_ur,
        &i.name_en,
        &i.confirmed_rules_words,
    )?;
    // The account takes the cleaned names, the same as the head.
    if let Some(names) = &base.names {
        base.account = Some(AccountToMake::new(
            EXPENSE_CONTROL,
            &names.name_ur,
            &names.name_en,
        ));
    }
    Some(base)
}

fn staff(errors: &mut FieldErrors, i: &StaffInput) -> Option<Prepared> {
    let version = edit_version(errors, i.id, i.version);
    let code = master_code(errors, MasterKind::Staff, &i.code);
    let job = choice(errors, "job", &i.job, &["munshi", "collector", "labour"]);
    let phone = errors.take(validate::phone_optional("phone", &i.phone));
    let (ur, en) = names(errors, &i.name_ur, &i.name_en)?;
    let mut fields = name_columns(&ur, &en);
    fields.push(("role", ColValue::Text(job?)));
    fields.push(("phone", ColValue::optional_text(phone?)));
    Some(Prepared {
        kind: MasterKind::Staff,
        id: i.id,
        version,
        code,
        fields,
        names: Some(NameTexts {
            field_ur: "name_ur",
            name_ur: ur,
            field_en: "name_en",
            name_en: en,
        }),
        confirmed_rules_words: i.confirmed_rules_words.clone(),
        references: Vec::new(),
        account: None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::CityInput;
    use super::*;

    fn city(code: &str, ur: &str, en: &str) -> MasterInput {
        MasterInput::City(CityInput {
            id: None,
            version: None,
            code: code.to_owned(),
            name_ur: ur.to_owned(),
            name_en: en.to_owned(),
            confirmed_rules_words: Vec::new(),
        })
    }

    fn fields(p: &Prepared) -> Vec<(&str, &ColValue)> {
        p.fields.iter().map(|(k, v)| (*k, v)).collect()
    }

    #[test]
    fn a_city_gets_both_names_and_the_three_search_columns_from_core_text() {
        // Plan 06 worked example: Kot Radha Kishan.
        let p = prepare(&city(" 12 ", "کوٹ رادھا کشن", "Kot Radha Kishan")).unwrap();
        assert_eq!(p.code.as_deref(), Some("12"));
        let f = fields(&p);
        assert!(f.contains(&("name_ur", &ColValue::text("کوٹ رادھا کشن"))));
        assert!(f.contains(&("name_en", &ColValue::text("Kot Radha Kishan"))));
        assert!(f.contains(&("search_en", &ColValue::text("kot radha kishan"))));
        assert!(f.contains(&("search_phonetic", &ColValue::text("ktrdhkshn"))));
        assert!(p.names.is_some() && p.account.is_none());
    }

    #[test]
    fn the_look_alike_urdu_letters_are_unified_in_the_search_column() {
        let p = prepare(&city("", "كوٹ", "Kot")).unwrap();
        assert!(fields(&p).contains(&("search_ur", &ColValue::text("کوٹ"))));
        assert_eq!(p.code, None, "an empty code asks for the next one");
    }

    #[test]
    fn wrong_fields_come_back_together_with_their_paths() {
        let errors = prepare(&city("12a", "Kot", "کوٹ")).unwrap_err();
        let got: Vec<(String, ErrorCode)> = errors.into_iter().map(|e| (e.field, e.code)).collect();
        assert_eq!(
            got,
            [
                ("code".to_owned(), ErrorCode::ErrFormatInvalid),
                ("name_ur".to_owned(), ErrorCode::ErrNameUrScript),
                ("name_en".to_owned(), ErrorCode::ErrNameEnScript),
            ]
        );
        let long_code = prepare(&city("12345678901", "کوٹ", "Kot")).unwrap_err();
        assert_eq!(long_code[0].code, ErrorCode::ErrFormatInvalid);
    }

    #[test]
    fn an_edit_needs_its_version() {
        let mut input = city("1", "کوٹ", "Kot");
        if let MasterInput::City(c) = &mut input {
            c.id = Some(3);
        }
        let errors = prepare(&input).unwrap_err();
        assert_eq!(
            (errors[0].field.as_str(), errors[0].code),
            ("version", ErrorCode::ErrRequired)
        );
        if let MasterInput::City(c) = &mut input {
            c.version = Some(2);
        }
        let p = prepare(&input).unwrap();
        assert_eq!((p.id, p.version), (Some(3), Some(2)));
    }

    #[test]
    fn item_choices_packing_weight_and_references() {
        let input = |group: &str, sold: &str, packing: Option<i64>| {
            MasterInput::Item(ItemInput {
                id: None,
                version: None,
                code: "0041".into(),
                name_ur: "کیلا".into(),
                name_en: "Banana".into(),
                item_group: group.into(),
                default_packing_id: packing,
                sold_by: sold.into(),
                confirmed_rules_words: Vec::new(),
            })
        };
        let p = prepare(&input("fruit", "unit", Some(4))).unwrap();
        assert!(fields(&p).contains(&("item_group", &ColValue::text("fruit"))));
        assert_eq!(
            p.references,
            [Reference {
                field: "default_packing_id",
                table: "packings",
                id: 4
            }]
        );
        let bad = prepare(&input("tree", "", Some(0))).unwrap_err();
        let got: Vec<(&str, ErrorCode)> = bad.iter().map(|e| (e.field.as_str(), e.code)).collect();
        assert_eq!(
            got,
            [
                ("item_group", ErrorCode::ErrFormatInvalid),
                ("sold_by", ErrorCode::ErrRequired),
                ("default_packing_id", ErrorCode::ErrNotFound),
            ]
        );
    }

    #[test]
    fn an_item_packing_or_staff_code_is_a_full_code_of_its_type() {
        // Plan 32 R2: 004, 005, 006 and a running number with no padding.
        let item = |code: &str| {
            MasterInput::Item(ItemInput {
                id: None,
                version: None,
                code: code.into(),
                name_ur: "کیلا".into(),
                name_en: "Banana".into(),
                item_group: "fruit".into(),
                default_packing_id: None,
                sold_by: "unit".into(),
                confirmed_rules_words: Vec::new(),
            })
        };
        assert_eq!(
            prepare(&item("0040")).unwrap().code.as_deref(),
            Some("0040")
        );
        assert_eq!(
            prepare(&item("00410")).unwrap().code.as_deref(),
            Some("00410")
        );
        assert_eq!(
            prepare(&item("")).unwrap().code,
            None,
            "empty asks for the next one"
        );
        for bad in ["0050", "0060", "40", "004", "00401", "0040a"] {
            let errors = prepare(&item(bad)).unwrap_err();
            assert_eq!(errors[0].field, "code", "{bad}");
            assert!(
                matches!(
                    errors[0].code,
                    ErrorCode::ErrCodeInvalid | ErrorCode::ErrFormatInvalid
                ),
                "{bad}: {:?}",
                errors[0].code
            );
        }
    }

    #[test]
    fn packing_weight_is_kg_with_up_to_three_decimals() {
        let input = |kg: &str| {
            MasterInput::Packing(PackingInput {
                id: None,
                version: None,
                code: "0051".into(),
                name_ur: "پیٹی".into(),
                name_en: "Box".into(),
                approx_kg: kg.into(),
                confirmed_rules_words: Vec::new(),
            })
        };
        let grams = |kg: &str| {
            prepare(&input(kg)).map(|p| {
                p.fields
                    .into_iter()
                    .find(|(k, _)| *k == "approx_grams")
                    .map(|(_, v)| v)
            })
        };
        assert_eq!(grams("12.5").unwrap(), Some(ColValue::Int(12_500)));
        assert_eq!(grams("").unwrap(), Some(ColValue::Null));
        assert_eq!(grams("0.001").unwrap(), Some(ColValue::Int(1)));
        assert!(grams("1.2345").is_err());
        assert!(grams("0").is_err());
        assert!(grams("abc").is_err());
        // The biggest weight one input holds, and one gram more.
        assert_eq!(grams("100000").unwrap(), Some(ColValue::Int(100_000_000)));
        assert!(grams("100000.001").is_err());
        assert!(grams("12,5").is_err() && grams("12.").is_err());
        // Urdu digits, and a field that holds only an invisible character is empty.
        assert_eq!(grams("۱۲٫۵").unwrap(), Some(ColValue::Int(12_500)));
        assert_eq!(grams("\u{200b}").unwrap(), Some(ColValue::Null));
    }

    #[test]
    fn a_marka_is_kept_as_typed_and_has_no_names_to_check() {
        let p = prepare(&MasterInput::Marka(MarkaInput {
            id: None,
            version: None,
            code: "".into(),
            mark_text: " AR ".into(),
            owner_account_id: Some(7),
        }))
        .unwrap();
        assert!(p.names.is_none());
        assert!(fields(&p).contains(&("mark_text", &ColValue::text("AR"))));
    }

    #[test]
    fn narrations_are_free_text_in_their_own_script_with_a_fixed_used_in_order() {
        let input = |ur: &str, en: &str, used: &[&str]| {
            MasterInput::Narration(NarrationInput {
                id: None,
                version: None,
                text_ur: ur.into(),
                text_en: en.into(),
                used_in: used.iter().map(|s| (*s).to_owned()).collect(),
            })
        };
        let p = prepare(&input(
            "بجلی کا بل، ماہ اکتوبر",
            "Electricity bill, October",
            &["payment", "receipt", "payment"],
        ))
        .unwrap();
        assert!(
            fields(&p).contains(&("used_in_json", &ColValue::text("[\"receipt\",\"payment\"]")))
        );
        assert!(p.names.is_none(), "free text is not spell checked");
        let none = prepare(&input("بل", "Bill", &[])).unwrap();
        assert!(fields(&none).contains(&("used_in_json", &ColValue::Null)));
        assert!(prepare(&input("Bill", "Bill", &[])).is_err());
        assert!(prepare(&input("بل", "بل", &[])).is_err());
        assert!(prepare(&input("بل", "Bill", &["loan"])).is_err());
        assert!(prepare(&input("", "Bill", &[])).is_err());
    }

    #[test]
    fn an_expense_head_makes_its_account_under_0501_with_the_same_names() {
        let p = prepare(&MasterInput::ExpenseHead(ExpenseHeadInput {
            id: None,
            version: None,
            code: "".into(),
            name_ur: "بجلی".into(),
            name_en: "Electricity".into(),
            confirmed_rules_words: Vec::new(),
        }))
        .unwrap();
        let account = p.account.unwrap();
        assert_eq!(account.control_code, "0501");
        assert_eq!(
            (account.name_ur.as_str(), account.name_en.as_str()),
            ("بجلی", "Electricity")
        );
        assert_eq!(account.search_en, "electricity");
    }

    #[test]
    fn staff_have_a_job_and_an_optional_phone() {
        let input = |job: &str, phone: &str| {
            MasterInput::Staff(StaffInput {
                id: None,
                version: None,
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                code: String::new(),
                job: job.into(),
                phone: phone.into(),
                confirmed_rules_words: Vec::new(),
            })
        };
        let p = prepare(&input("collector", "0300-1234567")).unwrap();
        assert!(fields(&p).contains(&("phone", &ColValue::text("03001234567"))));
        assert!(fields(&p).contains(&("role", &ColValue::text("collector"))));
        let p = prepare(&input("labour", "")).unwrap();
        assert!(fields(&p).contains(&("phone", &ColValue::Null)));
        assert!(
            prepare(&input("owner", "")).is_err(),
            "a job, not a login role"
        );
        assert!(prepare(&input("munshi", "12345")).is_err());
    }
}
