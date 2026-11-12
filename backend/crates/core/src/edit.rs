//! The one edit model (PRD FR-7.70, FR-7.71, FR-21.4; plan 44): the flags every saved-entry row
//! carries, the change list and effect every edit preview returns, the stamp of an edited entry,
//! the inputs and result of the edits that are new in plan 44, and the reopen-and-fix pair.
//!
//! Pure types. The services fill them; the screens only show them (CLAUDE.md rule 1). Dates are
//! `YYYY-MM-DD`; times are UTC with the Pakistan day and clock time beside them (Code Standards G9).
//!
//! Example (PRD E2E-45): بکری 1 is edited, اکرم's rate 600 to 610. The preview's `changes` has one
//! line (the line of اکرم, field `seller_rate`, before 600, after 610), its `effect` has اکرم's
//! balance 12,880 → 13,092 بنام, and each partner's share of the posting: الف 1,580 → 1,590,
//! ب 1,184 → 1,191, ج 1,184 → 1,191.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{Balance, RecordRef};
use crate::error::{ErrorCode, FieldError};
use crate::gardens::{GardenEntryInput, GardenInput};
use crate::money::Paisa;
use crate::purchases::PurchaseInput;
use crate::trips::{OtherSaleInput, TripCostInput, TripInput};

/// Why an entry cannot be edited or cancelled (plan 44 Section 8.3 and 9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LockedReason {
    /// The goods it touched were sold or written off (a trip purchase, trip cost, sale in another
    /// shop, garden entry or arrival).
    GoodsMoved,
    /// A later posting, a write-off or fruit rests on a lot, so its seller
    /// cannot change (A74).
    LotSellerLocked,
    /// The garden of the entry is closed (PRD FR-12.38).
    GardenClosed,
    /// The entry's month is month-closed (PRD FR-21.2); reverse the month close first.
    MonthClosed,
    /// The entry is cancelled already.
    Cancelled,
    /// A take or a loan with returns standing on it (PRD FR-16.2, FR-16.5).
    ReturnsStand,
}

/// When and by whom an entry was last edited (PRD FR-7.71). Read from the audit log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EditStamp {
    /// The time of the last edit, UTC, `YYYY-MM-DDTHH:MM:SS.sssZ`.
    pub at: String,
    /// The same moment as a Pakistan date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The same moment as a Pakistan clock time, 24-hour `HH:MM`. The screen writes "4:35 شام".
    pub local_time: String,
    /// Who made it: the display name of the signed-in user (PRD A75).
    pub by: String,
    /// How many times the entry was edited (the latest is this one).
    #[ts(type = "number")]
    pub count: i64,
    /// The reason typed with the last edit. Null when none was given (a today's edit).
    pub reason: Option<String>,
    /// The Owner's sign given when the day of the entry was reopened to fix it (PRD FR-21.4, FR-7.71,
    /// plan 73). Null for an edit of an open day.
    #[serde(default)]
    pub sign: Option<String>,
}

/// Pakistan time is UTC plus five hours all year (no daylight saving).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

impl EditStamp {
    /// The stamp of an edit made at `at` (UTC, RFC 3339, as the audit log stores it), with the
    /// Pakistan date and clock time beside it. `None` when `at` is not a time.
    ///
    /// Example: `2026-10-10T11:35:00.000Z` is 16:35 on 2026-10-10 in Pakistan.
    #[must_use]
    pub fn from_utc(at: &str, by: String, count: i64, reason: Option<String>) -> Option<Self> {
        let parsed = chrono::DateTime::parse_from_rfc3339(at)
            .ok()?
            .with_timezone(&chrono::Utc);
        let local = parsed.checked_add_signed(chrono::Duration::hours(PAKISTAN_OFFSET_HOURS))?;
        Some(Self {
            at: at.to_owned(),
            local_date: local.format("%Y-%m-%d").to_string(),
            local_time: local.format("%H:%M").to_string(),
            by,
            count,
            reason,
            sign: None,
        })
    }
}

impl EditStamp {
    /// The same stamp with the sign of the fix it was made in (PRD FR-21.4, plan 73).
    #[must_use]
    pub fn with_sign(mut self, sign: Option<String>) -> Self {
        self.sign = sign;
        self
    }
}

/// What every saved-entry row says about being edited or cancelled (plan 44 Section 8.3). It is
/// flattened into the row, so a row has `can_edit`, `can_cancel`, `locked_reason`, `needs_reopen`
/// and `edited` as its own fields. It replaces the old `editable` and `can_change` fields.
///
/// `can_edit` is true when the Edit button works. On an entry of a closed day it is still true
/// and `needs_reopen` is true: the button then reads "دن کھولیں اور درست کریں" (PRD FR-21.4).
/// When `locked_reason` is set, `can_edit` (and, for the reasons that stop a cancel too,
/// `can_cancel`) is false.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EntryFlags {
    /// The Edit button works (alone, or after a reopen).
    pub can_edit: bool,
    /// The Cancel button works (alone, or after a reopen).
    pub can_cancel: bool,
    /// Why not, when not. Null when both work.
    pub locked_reason: Option<LockedReason>,
    /// The entry's day is closed: Edit and Cancel go through "reopen and fix" (PRD FR-21.4).
    pub needs_reopen: bool,
    /// The last edit. Null when the entry was never edited.
    pub edited: Option<EditStamp>,
}

impl EntryFlags {
    /// The flags of a saved entry nothing stops: both buttons work, no reopen, never edited.
    #[must_use]
    pub fn open() -> Self {
        Self {
            can_edit: true,
            can_cancel: true,
            locked_reason: None,
            needs_reopen: false,
            edited: None,
        }
    }

    /// Flags with only the two buttons set: no reason, no reopen, never edited. The services use
    /// it until the full rules of plan 44 T2b fill `locked_reason`, `needs_reopen` and `edited`.
    #[must_use]
    pub fn plain(can_edit: bool, can_cancel: bool) -> Self {
        Self {
            can_edit,
            can_cancel,
            locked_reason: None,
            needs_reopen: false,
            edited: None,
        }
    }

    /// Flags with nothing allowed and no reason: what the database layer returns before the
    /// service marks the row (it knows the day, the goods and the stamp; the repository does not).
    #[must_use]
    pub fn unmarked() -> Self {
        Self {
            can_edit: false,
            can_cancel: false,
            locked_reason: None,
            needs_reopen: false,
            edited: None,
        }
    }

    /// The flags of an entry that cannot be changed, for `reason`.
    #[must_use]
    pub fn locked(reason: LockedReason) -> Self {
        Self {
            can_edit: false,
            can_cancel: false,
            locked_reason: Some(reason),
            needs_reopen: false,
            edited: None,
        }
    }
}

/// What kind of place a change is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EditPlaceKind {
    /// The whole entry (a voucher's amount, a trip cost's reason).
    Entry,
    /// The header of a بکری or of a trip or garden.
    Header,
    /// A lot of a بکری.
    Lot,
    /// A sale line of a بکری.
    Line,
    /// A typed lot charge (کرایہ, مزدوری, منشیانہ, فون, نقد, کرایہ سٹور).
    Charge,
    /// A partner's part of a payment (a trip or garden payment).
    Part,
}

/// Where a change is, so the screen can write "رحیم، لائن 1: ..." (plan 44).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EditPlace {
    /// The kind of place.
    pub kind: EditPlaceKind,
    /// The lot number, for a lot, a charge or a line.
    #[ts(type = "number | null")]
    pub lot_no: Option<i64>,
    /// The counter of the lot's name (PRD FR-12.45), for a lot, a charge or a line. With
    /// `label_ur` and `label_en` (the seller) and `item_ur` and `item_en` it writes "احسان، آم-2".
    #[serde(default)]
    #[ts(type = "number | null")]
    pub lot_counter: Option<i64>,
    /// The item of the lot, Urdu, for the lot name. Null when there is no lot.
    #[serde(default)]
    pub item_ur: Option<String>,
    /// The item of the lot, English.
    #[serde(default)]
    pub item_en: Option<String>,
    /// The line number inside the posting, for a line.
    #[ts(type = "number | null")]
    pub line_no: Option<i64>,
    /// A name that tells the place apart: the seller of the lot, the buyer of the line, the
    /// partner of the part. Urdu. Null when there is none.
    pub label_ur: Option<String>,
    /// The same name in English.
    pub label_en: Option<String>,
}

/// One side of a change, typed so the screen formats it without working anything out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeValue {
    /// Nothing (a field that was empty, or a line that was not there).
    Empty,
    /// Text as typed.
    Text {
        /// The text.
        text: String,
    },
    /// A whole number (a quantity).
    Number {
        /// The value.
        #[ts(type = "number")]
        value: i64,
    },
    /// Money in whole rupees (a rate, an amount, a charge).
    Money {
        /// The amount.
        #[ts(type = "Paisa")]
        paisa: Paisa,
    },
    /// A weight in grams, shown as kilos.
    Weight {
        /// The weight.
        #[ts(type = "number")]
        grams: i64,
    },
    /// A rate in basis points (a commission or لاگا percent).
    Bps {
        /// 1% is 100.
        #[ts(type = "number")]
        bps: i64,
    },
    /// A date.
    Date {
        /// `YYYY-MM-DD`.
        iso: String,
    },
    /// A record in both scripts (a party, an item, a packing, a partner).
    Names {
        /// Urdu name.
        ur: String,
        /// English name.
        en: String,
    },
    /// A yes or no (a tick).
    Flag {
        /// The value.
        on: bool,
    },
    /// One of a fixed list, by its stable word (`credit`, `cash`, `firm`, `seller`); the screen
    /// takes its Fluent message from it.
    Choice {
        /// The word.
        word: String,
    },
}

/// One change of an edit: where, which field, what it was and what it will be (PRD FR-7.70
/// "Shown before save": "رحیم، لائن 1: ریٹ 600 تھا، اب 610").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EditChange {
    /// Where.
    pub place: EditPlace,
    /// A stable key of the field, for example `seller_rate`, `buyer_rate`, `qty`, `item`, `seller`,
    /// `labour`, `amount`, `narration`. The screen takes its label from Fluent
    /// (`edit-field-<key>`).
    pub field: String,
    /// The value now.
    pub before: ChangeValue,
    /// The value after the edit.
    pub after: ChangeValue,
}

/// A party's balance before and after an edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EffectParty {
    /// The account.
    pub account: RecordRef,
    /// His balance now.
    pub before: Balance,
    /// His balance after the edit.
    pub after: Balance,
}

/// What an edit does to cash in hand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EffectCash {
    /// Cash in hand now (it can be below 0).
    #[ts(type = "Paisa")]
    pub before_paisa: Paisa,
    /// Cash in hand after the edit.
    #[ts(type = "Paisa")]
    pub after_paisa: Paisa,
    /// True if cash ends below 0 and the edit lowers it: the save then asks to be confirmed
    /// (PRD FR-6.7).
    pub needs_confirm: bool,
}

/// What an edit does to one partner's share of the entry's income or expense.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EffectPartner {
    /// The partner.
    pub partner: RecordRef,
    /// His figure from the posting being edited now: his share rows of it plus his own accounts'
    /// lines in it (income positive, expense and money out negative).
    #[ts(type = "Paisa")]
    pub before_paisa: Paisa,
    /// The same figure from the posting that takes its place.
    #[ts(type = "Paisa")]
    pub after_paisa: Paisa,
}

/// The effect of an edit, worked out by the same function as the save (Code Standards B5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditEffect {
    /// Each party whose balance changes.
    pub parties: Vec<EffectParty>,
    /// Cash in hand, when the edit moves it. Null when it does not.
    pub cash: Option<EffectCash>,
    /// Each partner whose share changes. Empty while the partner section is locked.
    pub partners: Vec<EffectPartner>,
    /// True while the partner section is locked (PRD FR-17.11): `partners` is then empty and the
    /// screen shows "کوڈ درج کریں" in its place.
    pub partners_locked: bool,
}

/// The review of an edit before it is saved: what changes and what that does (PRD FR-7.70).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditReview {
    /// The date of the entry, which never changes. `YYYY-MM-DD`.
    pub entry_date: String,
    /// Each change, in the order of the screen.
    pub changes: Vec<EditChange>,
    /// What the changes do.
    pub effect: EditEffect,
    /// True when the save needs a reason: the entry is not of today (PRD FR-7.70, A78).
    pub reason_required: bool,
    /// True when the entry's day is closed, so the save goes through "reopen and fix".
    pub needs_reopen: bool,
}

/// The answer of the preview of a new edit (trip purchase, cost, sale in another shop, garden entry
/// or arrival, trip or garden header). The entry's own numbers come from its own preview command;
/// this one holds the review and every error the save would return, all together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditPreview {
    /// Every mistake of the edit, each at its field. Empty when it can be saved.
    pub errors: Vec<FieldError>,
    /// The review.
    pub review: EditReview,
}

/// The answer of the save of a new edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EntryEdited {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number (it never changes). Null for a header.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The new version.
    #[ts(type = "number")]
    pub version: i64,
    /// The posting that replaced the old one. Null when the edit posts nothing.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// The reversal of the old posting. Null when the edit posts nothing.
    #[ts(type = "number | null")]
    pub reversal_entry_id: Option<i64>,
    /// What changed, as the preview showed it.
    pub review: EditReview,
    /// The stamp the entry now carries.
    pub edited: EditStamp,
}

/// Edit of a trip purchase (`purchases_edit_preview`, `purchases_edit_save`): the same fields as a
/// new one. The date and the trip never change (`ERR_EDIT_DATE_FIXED`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseEditInput {
    /// The purchase.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The purchase as it should be. Its `confirm_cash_below_zero` answers the cash question of
    /// PRD FR-6.7.
    pub purchase: PurchaseInput,
    /// Why. Required when the purchase is not of today (`ERR_REASON_REQUIRED` on `reason`).
    #[serde(default)]
    pub reason: String,
}

/// Edit of a trip cost (`trips_cost_edit_preview`, `trips_cost_edit_save`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripCostEditInput {
    /// The cost.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The cost as it should be. Its `confirm_cash_below_zero` answers the cash question of PRD
    /// FR-6.7.
    pub cost: TripCostInput,
    /// Why. Required when the cost is not of today.
    #[serde(default)]
    pub reason: String,
}

/// Edit of a sale in another shop of a trip (`trips_other_sale_edit_preview`,
/// `trips_other_sale_edit_save`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripOtherSaleEditInput {
    /// The sale.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The sale as it should be.
    pub sale: OtherSaleInput,
    /// Why. Required when the sale is not of today.
    #[serde(default)]
    pub reason: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟". Needed only when the edit lowers the
    /// money received and so takes روکڑ below 0 (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Edit of a garden entry: an installment, a running cost or a sale in another shop
/// (`gardens_entry_edit_preview`, `gardens_entry_edit_save`). The kind of the entry never changes.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenEntryEditInput {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The entry as it should be. Where it moves cash, its `confirm_cash_below_zero` answers the
    /// cash question of PRD FR-6.7.
    pub entry: GardenEntryInput,
    /// Why. Required when the entry is not of today.
    #[serde(default)]
    pub reason: String,
    /// The cash question of a sale in another shop (PRD FR-6.7): true after the user said yes to "کیا
    /// یہ اندراج درست ہے؟" for an edit that lowers the money received and takes روکڑ below 0. A
    /// payment answers it inside the entry.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Edit of a garden arrival (`gardens_arrival_edit_preview`, `gardens_arrival_edit_save`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenArrivalEditInput {
    /// The arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The arrival as it should be.
    pub arrival: crate::arrivals::ArrivalInput,
    /// Why. Required when the arrival is not of today.
    #[serde(default)]
    pub reason: String,
}

/// Edit of the header of a trip (`trips_header_edit_preview`, `trips_header_edit_save`): the
/// market, the person sent and the note. The date and the shares never change (FR-12.29).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripHeaderEditInput {
    /// The trip.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The trip as it should be. `date` must be the trip's own date and `partners` its own
    /// partners and shares, else `ERR_EDIT_DATE_FIXED` or `ERR_VENTURE_SHARES_LOCKED`.
    pub trip: TripInput,
    /// Why. Required when the trip is not of today.
    #[serde(default)]
    pub reason: String,
}

/// Edit of the header of a garden (`gardens_header_edit_preview`, `gardens_header_edit_save`): the
/// names, the item, the rent, the season end and the note. The date and the shares never change.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenHeaderEditInput {
    /// The garden.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read.
    #[ts(type = "number")]
    pub version: i64,
    /// The garden as it should be. `date` must be the garden's own date; `paid_now` and `parts`
    /// are ignored (the first installment is an entry of its own).
    pub garden: GardenInput,
    /// Why. Required when the garden is not of today.
    #[serde(default)]
    pub reason: String,
}

/// The audit entity names an edit stamp and a history can be asked for. Each stands for the
/// `entity` of the `audit_log` rows of that kind of entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EditEntity {
    /// A بکری (`bikri`; the id is the بکری's).
    Bikri,
    /// A voucher of any type (`voucher`).
    Voucher,
    /// Fruit taken (`fruit_take`).
    FruitTake,
    /// A trip purchase (`purchase`).
    Purchase,
    /// A trip cost (`trip_cost`).
    TripCost,
    /// A sale in another shop of a trip (`trip_other_sale`).
    TripOtherSale,
    /// A garden installment, cost or sale in another shop (`garden_entry`).
    GardenEntry,
    /// A garden arrival (`garden_arrival`).
    GardenArrival,
    /// A trip (`trip`).
    Trip,
    /// A garden (`garden`).
    Garden,
    /// An account (`account`).
    Account,
    /// A commission rule (`charge_rule`).
    ChargeRule,
}

impl EditEntity {
    /// The `entity` text of its audit rows.
    #[must_use]
    pub const fn audit_entity(self) -> &'static str {
        match self {
            Self::Bikri => "bikri",
            Self::Voucher => "voucher",
            Self::FruitTake => "fruit_take",
            Self::Purchase => "purchase",
            Self::TripCost => "trip_cost",
            Self::TripOtherSale => "trip_other_sale",
            Self::GardenEntry => "garden_entry",
            Self::GardenArrival => "garden_arrival",
            Self::Trip => "trip",
            Self::Garden => "garden",
            Self::Account => "account",
            Self::ChargeRule => "charge_rule",
        }
    }
}

/// Input of `day_fix_open` (PRD FR-21.4): reopen the date and every later closed date to fix an
/// entry.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DayFixOpenInput {
    /// The date of the entry to fix, as typed (`DD-MM-YYYY`).
    pub date: String,
    /// Why. Required (as FR-21.2).
    pub reason: String,
    /// The Owner's sign (دستخط): only his sign opens the day (PRD FR-21.4, plan 73). Required;
    /// compared after trimming, ignoring case. Left out of a request, it is empty and refused.
    #[serde(default)]
    pub sign: String,
}

/// The answer of `day_fix_open`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayFixOpened {
    /// The token for `day_fix_close`. Opaque.
    pub token: String,
    /// The dates that were reopened, oldest first, `YYYY-MM-DD`.
    pub days: Vec<String>,
    /// The "closed up to" date now, `YYYY-MM-DD`. Null when no day is closed.
    pub closed_up_to: Option<String>,
}

/// Input of `day_fix_close` (PRD FR-21.4).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DayFixCloseInput {
    /// The token `day_fix_open` gave.
    pub token: String,
}

/// A day that stayed open when the days were closed again, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayFixStayedOpen {
    /// The date, `YYYY-MM-DD`.
    pub date: String,
    /// Why: the errors of the checks of FR-21.1 (trial balance not 0, and so on).
    pub errors: Vec<FieldError>,
}

/// The answer of `day_fix_close`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayFixClosed {
    /// The dates that were closed again, oldest first.
    pub closed: Vec<String>,
    /// The dates that stayed open, with the reason. The attention list says the same.
    pub stayed_open: Vec<DayFixStayedOpen>,
    /// The "closed up to" date now, `YYYY-MM-DD`. Null when no day is closed.
    pub closed_up_to: Option<String>,
}

/// The code a refused date is checked against: the same text as [`ErrorCode::ErrEditDateFixed`].
#[must_use]
pub const fn date_fixed_code() -> ErrorCode {
    ErrorCode::ErrEditDateFixed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flags_of_a_free_entry_and_of_a_locked_one() {
        let free = EntryFlags::open();
        assert!(free.can_edit && free.can_cancel && free.locked_reason.is_none());
        let locked = EntryFlags::locked(LockedReason::GoodsMoved);
        assert!(!locked.can_edit && !locked.can_cancel);
        assert_eq!(
            serde_json::to_value(locked.locked_reason).unwrap(),
            serde_json::json!("goods_moved")
        );
    }

    #[test]
    fn every_entity_names_its_audit_rows() {
        let names = [
            (EditEntity::Bikri, "bikri"),
            (EditEntity::Voucher, "voucher"),
            (EditEntity::FruitTake, "fruit_take"),
            (EditEntity::Purchase, "purchase"),
            (EditEntity::TripCost, "trip_cost"),
            (EditEntity::TripOtherSale, "trip_other_sale"),
            (EditEntity::GardenEntry, "garden_entry"),
            (EditEntity::GardenArrival, "garden_arrival"),
            (EditEntity::Trip, "trip"),
            (EditEntity::Garden, "garden"),
            (EditEntity::Account, "account"),
            (EditEntity::ChargeRule, "charge_rule"),
        ];
        for (entity, text) in names {
            assert_eq!(entity.audit_entity(), text);
            assert_eq!(
                serde_json::to_value(entity).unwrap(),
                serde_json::json!(text)
            );
        }
    }

    #[test]
    fn a_change_value_is_tagged_by_its_type() {
        let value = ChangeValue::Money {
            paisa: Paisa::from_rupees(610).unwrap(),
        };
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::json!({ "type": "money", "paisa": 61_000 })
        );
        assert_eq!(date_fixed_code(), ErrorCode::ErrEditDateFixed);
    }
}
