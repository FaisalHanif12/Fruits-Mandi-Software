//! A saved بکری: what the save sends back, what the reads send back, and the پختہ کھاتہ lists
//! (plan 13 contract). Money is paisa. Names are `RecordRef` with both scripts, so the screen
//! shows them through `displayName` (Code Standards F38).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::input::{BikriDraft, FreightPaidBy, LotChargeMethod, PaymentType, StoredMethod};
use super::preview::BikriPreview;
use crate::accounts::RecordRef;
use crate::money::{MoneyError, Paisa};

/// Input of `bikri_save` (PRD FR-12.8, FR-6.7).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriSaveInput {
    /// The typed بکری. The backend works it out again with the same function as the preview.
    pub draft: BikriDraft,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟". Needed only when the entry takes
    /// روکڑ below 0 (PRD FR-6.7); without it the save is refused with
    /// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` and nothing is written.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What a save gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriSaved {
    /// The new بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number, given at this first save and never used again (PRD 12.2).
    #[ts(type = "number")]
    pub number: i64,
    /// The journal entry of its first posting.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The numbers the save wrote. They are the same as what `bikri_preview` gave for this draft
    /// just before (Code Standards B5).
    pub preview: BikriPreview,
}

/// Whether a بکری is in force or was cancelled (`bikri_entries.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum BikriStatus {
    /// محفوظ.
    Saved,
    /// منسوخ.
    Cancelled,
}

impl BikriStatus {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            BikriStatus::Saved => "saved",
            BikriStatus::Cancelled => "cancelled",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "saved" => Some(BikriStatus::Saved),
            "cancelled" => Some(BikriStatus::Cancelled),
            _ => None,
        }
    }
}

/// Input of `bikri_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriGetInput {
    /// The بکری.
    #[ts(type = "number")]
    pub id: i64,
}

/// The one-time person kept on a line or a lot of the پختہ کھاتہ accounts (PRD FR-13.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedPukhta {
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone as 11 digits. Empty if none.
    pub phone: String,
    /// The key that ties his lines together (PRD FR-13.5).
    pub person_key: String,
}

/// A commission or لاگا of a saved sale line: the snapshot of PRD FR-11.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedCharge {
    /// The method kept with the line.
    pub method: StoredMethod,
    /// The percent, in basis points, when the method is a percent.
    #[ts(type = "number | null")]
    pub value_bps: Option<i32>,
    /// The amount per نگ or per lot.
    #[ts(type = "Paisa | null")]
    pub value_paisa: Option<Paisa>,
    /// The amount, as stored.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// The rule that decided, or null.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
}

/// A lot charge of a saved lot (مزدوری, منشیانہ, فون).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedLotCharge {
    /// The method.
    pub method: LotChargeMethod,
    /// What the field shows: per نگ, per lot, or the amount itself.
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// The amount charged for the lot.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// The rule that suggested it, or null.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
}

/// Whether a saved sale line is in force, and if not, how it left (plan 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaleState {
    /// In force: it counts in the totals, the books and the stock.
    InForce,
    /// An edit wrote a new line in its place (PRD FR-12.19). History only.
    Replaced,
    /// An edit took the line out. History only.
    Removed,
    /// The whole بکری was cancelled while the line was in force (PRD FR-12.22). The saved view of a
    /// cancelled بکری shows these lines with the cancelled mark.
    Cancelled,
}

impl SaleState {
    /// The stored form of `bikri_sales.ended`; `None` is in force.
    pub const fn ended(self) -> Option<&'static str> {
        match self {
            SaleState::InForce => None,
            SaleState::Replaced => Some("replaced"),
            SaleState::Removed => Some("removed"),
            SaleState::Cancelled => Some("cancelled"),
        }
    }

    /// Reads the stored form.
    pub fn parse(ended: Option<&str>) -> Option<Self> {
        match ended {
            None => Some(SaleState::InForce),
            Some("replaced") => Some(SaleState::Replaced),
            Some("removed") => Some(SaleState::Removed),
            Some("cancelled") => Some(SaleState::Cancelled),
            Some(_) => None,
        }
    }
}

/// One saved sale line with its stored values (PRD FR-11.6, Code Standards B20).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedSale {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Position in the lot, from 1.
    #[ts(type = "number")]
    pub line_no: i64,
    /// The posting this line belongs to.
    #[ts(type = "number")]
    pub journal_entry_id: i64,
    /// The day of that posting, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The buyer account (own account, نقدی منڈی or پختہ کھاتہ).
    pub buyer: RecordRef,
    /// The one-time buyer, for the پختہ کھاتہ account.
    pub pukhta: Option<SavedPukhta>,
    /// Quantity in نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight in grams, for a weight item.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The seller's rate.
    #[ts(type = "Paisa")]
    pub seller_rate: Paisa,
    /// C1 with the seller rate.
    #[ts(type = "Paisa")]
    pub seller_amount: Paisa,
    /// The seller's commission.
    pub seller_comm: SavedCharge,
    /// The buyer's rate.
    #[ts(type = "Paisa")]
    pub buyer_rate: Paisa,
    /// C1 with the buyer rate.
    #[ts(type = "Paisa")]
    pub buyer_value: Paisa,
    /// The buyer's commission.
    pub buyer_comm: SavedCharge,
    /// لاگا.
    pub laga: SavedCharge,
    /// C5.
    #[ts(type = "Paisa")]
    pub buyer_total: Paisa,
    /// C19.
    #[ts(type = "Paisa")]
    pub rate_diff: Paisa,
    /// C21: the cost of the trip goods this line sold, stored at save and never posted (PRD
    /// FR-12.26). 0 on a line of any other lot.
    #[serde(default)]
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// How the buyer pays.
    pub payment_type: PaymentType,
    /// False once an edit or a cancel ended this line (plan 16). `state` says how.
    pub is_active: bool,
    /// In force, replaced or removed by an edit, or cancelled with its بکری. A cancelled بکری
    /// shows the lines in state `cancelled`; a saved one shows the lines `in_force`.
    pub state: SaleState,
    /// A note.
    pub note: Option<String>,
}

/// The lines of PRD 12.5 for one saved lot or for a whole saved بکری, as sums of the stored
/// values (Code Standards B20): the same fields as `SummaryTotals` without the workings. The
/// screen shows them as they are; it never adds money (CLAUDE.md rule 1).
///
/// Example (PRD 4.4): Event 1 has gross 29,400, deductions 10,084, net 19,316, seller credit
/// 24,316, buyer commission 1,764, لاگا 400, buyer bills 31,564 and shared income 3,948.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedTotals {
    /// خام بکری.
    #[ts(type = "Paisa")]
    pub gross: Paisa,
    /// کرایہ: the freight paid by the firm (0 when the seller paid it).
    #[ts(type = "Paisa")]
    pub freight: Paisa,
    /// کمیشن: the sum of the seller commissions of the lines in force.
    #[ts(type = "Paisa")]
    pub seller_comm: Paisa,
    /// مزدوری.
    #[ts(type = "Paisa")]
    pub labour: Paisa,
    /// منشیانہ.
    #[ts(type = "Paisa")]
    pub munshiana: Paisa,
    /// نقد.
    #[ts(type = "Paisa")]
    pub cash_paid: Paisa,
    /// کرایہ سٹور.
    #[ts(type = "Paisa")]
    pub store_rent: Paisa,
    /// فون.
    #[ts(type = "Paisa")]
    pub phone: Paisa,
    /// کل اخراجات (C6).
    #[ts(type = "Paisa")]
    pub deductions: Paisa,
    /// صافی بکری (C7). Can be negative.
    #[ts(type = "Paisa")]
    pub net: Paisa,
    /// What the posting credits the seller (C8). Can be negative.
    #[ts(type = "Paisa")]
    pub seller_credit: Paisa,
    /// گاہک کمیشن.
    #[ts(type = "Paisa")]
    pub buyer_comm: Paisa,
    /// لاگا.
    #[ts(type = "Paisa")]
    pub laga: Paisa,
    /// ریٹ فرق. Can be negative.
    #[ts(type = "Paisa")]
    pub rate_diff: Paisa,
    /// ادھاری: buyer bills of credit lines in force.
    #[ts(type = "Paisa")]
    pub buyer_credit: Paisa,
    /// نقدی: buyer bills of cash lines in force.
    #[ts(type = "Paisa")]
    pub buyer_cash: Paisa,
    /// All buyer bills together.
    #[ts(type = "Paisa")]
    pub buyer_total: Paisa,
    /// The income shared between the partners (C9): commission, لاگا and فون only, also for a trip
    /// lot (PRD FR-12.26).
    #[ts(type = "Paisa")]
    pub shared_income: Paisa,
}

impl SavedTotals {
    /// The totals of one saved lot from its stored columns and its lines in force. Every sum is
    /// checked; nothing is rounded or worked out again except the two formulas of C8 and C9,
    /// which are sums of stored amounts.
    ///
    /// # Errors
    /// `MoneyError::Overflow` if a sum does not fit.
    pub fn of_lot(lot: &SavedLot) -> Result<SavedTotals, MoneyError> {
        let freight = match lot.freight_paid_by {
            FreightPaidBy::Firm => lot.freight,
            FreightPaidBy::Seller => Paisa::ZERO,
        };
        let (mut buyer_comm, mut laga, mut rate_diff) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let (mut buyer_credit, mut buyer_cash) = (Paisa::ZERO, Paisa::ZERO);
        // The lines in force; for a cancelled بکری the lines that were in force when it was
        // cancelled, so its numbers still add up (plan 16).
        for sale in lot
            .sales
            .iter()
            .filter(|s| s.is_active || s.state == SaleState::Cancelled)
        {
            buyer_comm = buyer_comm.checked_add(sale.buyer_comm.amount)?;
            laga = laga.checked_add(sale.laga.amount)?;
            rate_diff = rate_diff.checked_add(sale.rate_diff)?;
            match sale.payment_type {
                PaymentType::Credit => buyer_credit = buyer_credit.checked_add(sale.buyer_total)?,
                PaymentType::Cash => buyer_cash = buyer_cash.checked_add(sale.buyer_total)?,
            }
        }
        let (labour, munshiana, phone) =
            (lot.labour.amount, lot.munshiana.amount, lot.phone.amount);
        // C8: gross less the commission and the lot charges that come off the seller's credit
        // (نقد is a separate debit line).
        let charges = freight
            .checked_add(labour)?
            .checked_add(munshiana)?
            .checked_add(lot.store_rent)?
            .checked_add(phone)?;
        let seller_credit = lot
            .gross
            .checked_sub(lot.seller_comm)?
            .checked_sub(charges)?;
        // C9: the income of the posting. The seller credit of a trip lot is not income: it goes to
        // the trip's partners (PRD FR-12.26).
        let shared_income = lot
            .seller_comm
            .checked_add(buyer_comm)?
            .checked_add(laga)?
            .checked_add(phone)?
            .checked_add(rate_diff)?;
        Ok(SavedTotals {
            gross: lot.gross,
            freight,
            seller_comm: lot.seller_comm,
            labour,
            munshiana,
            cash_paid: lot.cash_paid,
            store_rent: lot.store_rent,
            phone,
            deductions: lot.deductions,
            net: lot.net,
            seller_credit,
            buyer_comm,
            laga,
            rate_diff,
            buyer_credit,
            buyer_cash,
            buyer_total: buyer_credit.checked_add(buyer_cash)?,
            shared_income,
        })
    }

    /// This and another total together (the lots of a بکری make the whole بکری).
    ///
    /// # Errors
    /// `MoneyError::Overflow` if a sum does not fit.
    pub fn plus(&self, other: &SavedTotals) -> Result<SavedTotals, MoneyError> {
        Ok(SavedTotals {
            gross: self.gross.checked_add(other.gross)?,
            freight: self.freight.checked_add(other.freight)?,
            seller_comm: self.seller_comm.checked_add(other.seller_comm)?,
            labour: self.labour.checked_add(other.labour)?,
            munshiana: self.munshiana.checked_add(other.munshiana)?,
            cash_paid: self.cash_paid.checked_add(other.cash_paid)?,
            store_rent: self.store_rent.checked_add(other.store_rent)?,
            phone: self.phone.checked_add(other.phone)?,
            deductions: self.deductions.checked_add(other.deductions)?,
            net: self.net.checked_add(other.net)?,
            seller_credit: self.seller_credit.checked_add(other.seller_credit)?,
            buyer_comm: self.buyer_comm.checked_add(other.buyer_comm)?,
            laga: self.laga.checked_add(other.laga)?,
            rate_diff: self.rate_diff.checked_add(other.rate_diff)?,
            buyer_credit: self.buyer_credit.checked_add(other.buyer_credit)?,
            buyer_cash: self.buyer_cash.checked_add(other.buyer_cash)?,
            buyer_total: self.buyer_total.checked_add(other.buyer_total)?,
            shared_income: self.shared_income.checked_add(other.shared_income)?,
        })
    }
}

/// The purchase an own lot took its goods from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedPurchaseRef {
    /// The purchase.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number (خ-1, PUR-1).
    #[ts(type = "number")]
    pub number: i64,
    /// Its trip.
    #[ts(type = "number")]
    pub trip_id: i64,
    /// The trip's number (سفر-1, TRP-1).
    #[ts(type = "number")]
    pub trip_number: i64,
}

/// One trip partner's part of the seller credit of a saved trip lot (PRD C25), read from the lines of
/// the posting. Positive is credited to his 0604 account, negative is debited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedCredit {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His 0604 account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// His share of the trip in bps.
    pub share_bps: i32,
    /// His part, signed.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// What a saved lot of trip goods shows besides its lines (PRD FR-12.26, FR-12.27): stored values
/// only, never worked out again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedOwnLot {
    /// C21: the cost of the goods the lines in force sold, stored on the lines, never posted.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// The seller credit of the lot (C8) that went to the trip's partners.
    #[ts(type = "Paisa")]
    pub credit_paisa: Paisa,
    /// The result: the seller credit less the cost.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
    /// Each trip partner's part of the seller credit.
    pub credits: Vec<SavedCredit>,
}

/// One saved lot with its stored totals and lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedLot {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The lot number, from 1.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The seller account. For an own lot it is the اپنا مال account (030119998).
    pub seller: RecordRef,
    /// The purchase an own lot took its goods from (PRD FR-12.26). `null` for any other lot.
    #[serde(default)]
    pub purchase: Option<SavedPurchaseRef>,
    /// The cost, the result and the partners' parts of a trip lot. `null` for any other lot.
    #[serde(default)]
    pub own: Option<SavedOwnLot>,
    /// The one-time seller, for the پختہ کھاتہ account.
    pub pukhta: Option<SavedPukhta>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// The مارکہ row, if any.
    #[ts(type = "number | null")]
    pub marka_id: Option<i64>,
    /// The مارکہ text as typed, if any.
    pub marka_text: Option<String>,
    /// The store, if any.
    #[ts(type = "number | null")]
    pub store_id: Option<i64>,
    /// Quantity received.
    #[ts(type = "number")]
    pub qty_received: i64,
    /// Quantity sold, over all postings.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// Quantity written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// Weight received, in grams.
    #[ts(type = "number | null")]
    pub weight_received_grams: Option<i64>,
    /// Freight typed on the lot, even if the seller paid it.
    #[ts(type = "Paisa")]
    pub freight: Paisa,
    /// Who paid the driver.
    pub freight_paid_by: FreightPaidBy,
    /// مزدوری.
    pub labour: SavedLotCharge,
    /// منشیانہ.
    pub munshiana: SavedLotCharge,
    /// فون.
    pub phone: SavedLotCharge,
    /// نقد.
    #[ts(type = "Paisa")]
    pub cash_paid: Paisa,
    /// کرایہ سٹور.
    #[ts(type = "Paisa")]
    pub store_rent: Paisa,
    /// The tick "پیشگی سے کاٹیں" (PRD FR-12.23).
    pub adjust_advance: bool,
    /// The first posting of this lot.
    #[ts(type = "number | null")]
    pub first_entry_id: Option<i64>,
    /// خام بکری.
    #[ts(type = "Paisa")]
    pub gross: Paisa,
    /// The sum of the line commissions.
    #[ts(type = "Paisa")]
    pub seller_comm: Paisa,
    /// کل اخراجات (C6).
    #[ts(type = "Paisa")]
    pub deductions: Paisa,
    /// صافی بکری (C7). Can be negative.
    #[ts(type = "Paisa")]
    pub net: Paisa,
    /// The lines, in order.
    pub sales: Vec<SavedSale>,
    /// The lines of PRD 12.5 for this lot: sums of the stored values of its lines in force.
    pub totals: SavedTotals,
}

/// A posting of a بکری.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedPosting {
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// `bikri_first`, `bikri_later` or `reversal`.
    pub posting_type: String,
    /// Its day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The debit total of the entry.
    #[ts(type = "Paisa")]
    pub total: Paisa,
    /// True once an edit or a cancel took this posting back with a reversal (plan 16). A reversed
    /// posting is history: it cannot be edited and it holds no stock or money.
    pub reversed: bool,
    /// True if this posting can be edited now: a first or later posting that is not reversed, of
    /// a بکری that is saved, on a day that is still open (PRD FR-12.18). Set by the service.
    pub editable: bool,
}

/// One partner's part of the income of a saved بکری, summed over its postings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SavedSplit {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His part (income shares of the postings, reversals taken off).
    #[ts(type = "Paisa")]
    pub income_paisa: Paisa,
}

/// A saved بکری as `bikri_get` sends it: the stored values only, never worked out again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct BikriDetail {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The version of the row, to send back with an edit or a cancel (`ERR_VERSION_CHANGED` if
    /// the بکری changed since it was read).
    #[ts(type = "number")]
    pub version: i64,
    /// The number.
    #[ts(type = "number")]
    pub number: i64,
    /// The optional second number.
    pub book: Option<String>,
    /// The posting day of the first posting, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The bill day, `YYYY-MM-DD`.
    pub bill_date: String,
    /// The truck number.
    pub truck_no: Option<String>,
    /// The زمیندار.
    pub zamindar: Option<RecordRef>,
    /// The truck stand.
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// The driver's name.
    pub driver_name: Option<String>,
    /// The driver's phone.
    pub driver_phone: Option<String>,
    /// The total freight typed in the header.
    #[ts(type = "Paisa | null")]
    pub total_freight: Option<Paisa>,
    /// The default store.
    #[ts(type = "number | null")]
    pub store_id: Option<i64>,
    /// A note.
    pub note: Option<String>,
    /// Saved or cancelled.
    pub status: BikriStatus,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// How many times it was printed.
    #[ts(type = "number")]
    pub print_count: i64,
    /// The time it was saved, UTC.
    pub created_at: String,
    /// The lots.
    pub lots: Vec<SavedLot>,
    /// Its postings, oldest first.
    pub postings: Vec<SavedPosting>,
    /// The partner split of its income, from the saved share rows.
    pub split: Vec<SavedSplit>,
    /// The lines of PRD 12.5 for the whole بکری: the totals of its lots together. Stored values
    /// only; a line replaced or cancelled is not in them.
    pub totals: SavedTotals,
}

/// Input of `bikri_list` (PRD FR-12.7). Dates are `DD-MM-YYYY` or a shortcut; null or empty
/// means no limit. `date` is one day and wins over `from` and `to`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct BikriListInput {
    /// One day.
    #[serde(default)]
    pub date: Option<String>,
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only بکری that have a lot of this seller.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// Only بکری that have a lot of this item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// Only this status.
    #[serde(default)]
    pub status: Option<BikriStatus>,
}

/// One row of the list of بکری (PRD FR-12.7: number, time, truck, sellers, items, نگ, خام, صافی,
/// buyer total, status). All sums are of stored values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriListRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The number.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The time it was saved, UTC (the screen shows Pakistan time).
    pub created_at: String,
    /// The truck number.
    pub truck_no: Option<String>,
    /// The sellers of its lots, each once, in lot order. A one-time seller is the پختہ کھاتہ
    /// account; his name is in `pukhta_sellers`.
    pub sellers: Vec<RecordRef>,
    /// The names of one-time sellers, in lot order.
    pub pukhta_sellers: Vec<SavedPukhta>,
    /// The items of its lots, each once.
    pub items: Vec<RecordRef>,
    /// نگ received over all its lots.
    #[ts(type = "number")]
    pub qty_received: i64,
    /// نگ sold over all its lots.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// خام بکری.
    #[ts(type = "Paisa")]
    pub gross: Paisa,
    /// صافی بکری. Can be negative.
    #[ts(type = "Paisa")]
    pub net: Paisa,
    /// The buyer bills of its lines in force.
    #[ts(type = "Paisa")]
    pub buyer_total: Paisa,
    /// Saved or cancelled.
    pub status: BikriStatus,
}

/// Input of `bikri_find` (PRD FR-12.13, FR-7.50): every filter given must match. Newest first,
/// at most 50 rows.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct BikriFindInput {
    /// The بکری number.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The start of the truck number, ignoring letter case (the whole number finds only itself).
    #[serde(default)]
    pub truck_no: Option<String>,
    /// A seller of one of its lots.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// One day, `DD-MM-YYYY`.
    #[serde(default)]
    pub date: Option<String>,
    /// An item of one of its lots.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
}

/// Where `bikri_navigate` goes (PRD FR-12.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum NavigateTo {
    /// The lowest number.
    First,
    /// The number before the current one.
    Previous,
    /// The number after the current one.
    Next,
    /// The highest number.
    Last,
}

/// Input of `bikri_navigate`. The order is the بکری number; cancelled ones are included.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriNavigateInput {
    /// The بکری on the screen, if any (a new unsaved بکری has none). Previous of none is the last
    /// one, next of none is the first.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub current_id: Option<i64>,
    /// Where to go.
    pub to: NavigateTo,
}

/// Which side of the پختہ کھاتہ accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PukhtaSide {
    /// A one-time seller (the پختہ کھاتہ بیوپاری account).
    Seller,
    /// A one-time buyer (the پختہ کھاتہ خریدار account).
    Buyer,
}

/// Input of `search_pukhta_people` (PRD FR-7.29).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PukhtaPeopleInput {
    /// What was typed, raw. Matches either name or the phone. Empty lists the newest people.
    pub text: String,
    /// Which account the field belongs to.
    pub side: PukhtaSide,
}

/// A one-time person already recorded, for the one merged list of PRD FR-7.29.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PukhtaPerson {
    /// Urdu name, as last typed.
    pub name_ur: String,
    /// English name, as last typed.
    pub name_en: String,
    /// Phone as 11 digits. Empty if none.
    pub phone: String,
    /// The day of his last line, `YYYY-MM-DD`.
    pub last_date: String,
    /// The key that ties his lines together (PRD FR-13.5).
    pub person_key: String,
}

/// Paid or still due (PRD FR-13.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PaidState {
    /// Nothing is open.
    Paid,
    /// Something is open.
    Due,
}

/// Input of `pukhta_register` (PRD FR-13.6, R9). Dates are `DD-MM-YYYY` or a shortcut; empty
/// means no limit.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct PukhtaRegisterInput {
    /// First day.
    #[serde(default)]
    pub from: String,
    /// Last day.
    #[serde(default)]
    pub to: String,
    /// Only one side.
    #[serde(default)]
    pub side: Option<PukhtaSide>,
    /// Part of either name.
    #[serde(default)]
    pub name: Option<String>,
    /// Part of the phone.
    #[serde(default)]
    pub phone: Option<String>,
    /// Only paid or only due lines.
    #[serde(default)]
    pub state: Option<PaidState>,
}

/// One line of the پختہ کھاتہ register (PRD FR-13.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PukhtaRegisterRow {
    /// The day of the posting, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// Seller or buyer.
    pub side: PukhtaSide,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone as 11 digits. Empty if none.
    pub phone: String,
    /// The person key.
    pub person_key: String,
    /// The item.
    pub item: RecordRef,
    /// نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// A buyer line: what he pays (value, commission and لاگا). A seller lot: خام بکری.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// A buyer line: his commission. A seller lot: the seller commission.
    #[ts(type = "Paisa")]
    pub commission: Paisa,
    /// True when nothing is open on this line.
    pub paid: bool,
    /// What is still open on this line: the part of the bill not settled (buyer), or the part of
    /// his credit not yet paid out (seller). 0 when paid.
    #[ts(type = "Paisa")]
    pub open: Paisa,
    /// True if the بکری was cancelled. A cancelled line is neither paid nor due.
    pub cancelled: bool,
}

/// How a بکری was found by the global search (PRD FR-5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum BikriMatch {
    /// The typed digits are its number.
    Number,
    /// The typed text starts its truck number.
    Truck,
}

/// A بکری found by `search_global`. Opening it opens the بکری screen on this id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriHit {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The number.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The truck number.
    pub truck_no: Option<String>,
    /// Saved or cancelled.
    pub status: BikriStatus,
    /// Why it matched.
    pub matched: BikriMatch,
}
