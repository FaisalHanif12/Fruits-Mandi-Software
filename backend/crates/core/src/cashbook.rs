//! The روکڑ بک (PRD 16.1, FR-16.1, FR-19.11, plan 17): the shop's one cash account, every cash in
//! and out from all modules, one page at a time. There is no cash count box (FR-16.1).
//!
//! Command `cashbook_get`: input `CashbookInput`, answer `Cashbook`. Read only.
//!
//! Worked numbers (PRD E2E-8): after Events 1 to 5 the closing cash is 50,000 − 5,000 + 12,800 +
//! 10,000 − 4,316 − 3,000 − 2,000 = 58,484.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::auth::Language;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};
use crate::printing::Paper;
use crate::row_source::RowSource;

/// Input of `cashbook_get`. Dates are `DD-MM-YYYY` or a shortcut; null or empty means no limit
/// (from the first entry, to today).
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CashbookInput {
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `posting_type`, `number`, `narration`,
    /// `in_paisa` or `out_paisa`. Null is by day and entry, oldest first (plan 38 R4). Only the day
    /// order (either way) has the running cash: for any other column `running_paisa` is null.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// What a cash row is, for the rows that start the book or move a partner's capital (plan 78 E,
/// PRD FR-16.14). The posting type says what the entry is; this says it in the words the Owner
/// reads: the opening cash, a partner's opening capital, and capital put in or taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CashbookKind {
    /// The opening balance typed on the cash account itself (ابتدائی روکڑ).
    OpeningCash,
    /// A partner's opening capital that came into cash (`partner` says whose).
    OpeningCapital,
    /// Capital a partner put in (سرمایہ جمع).
    CapitalIn,
    /// Capital a partner took back (سرمایہ واپسی).
    CapitalOut,
    /// Any other row: a sale, a voucher, an expense and the rest. `posting_type` says which.
    Other,
}

/// One cash movement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookRow {
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting type in its stored form: `bikri_first`, `bikri_later`, `reversal`, `receipt`,
    /// `payment`, `advance`, `shop_expense`, `personal_expense`, `personal_return`, `money_lent`,
    /// `money_lent_return`, `capital_in`, `capital_out`, `khata_out`, `khata_in`, `purchase`,
    /// `trip_cost`, `garden_payment`, `venture_receipt`, `garden_close`, `fruit_taken`, `general`,
    /// `opening`, `month_close`. The screen turns it into words with Fluent.
    pub posting_type: String,
    /// The number of the source (a بکری or a voucher), or null.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The other account of the entry (for example the buyer of a وصولی), or null when there are
    /// several.
    pub account: Option<RecordRef>,
    /// What the row is when it opens the book or moves a partner's capital (plan 78 E). `other` for
    /// every other row.
    pub kind: CashbookKind,
    /// The partner of an opening capital, capital in or capital out row. Null for every other row.
    /// `code` is empty: a partner has no code.
    pub partner: Option<RecordRef>,
    /// The note. Empty if none.
    pub narration: String,
    /// Cash in (0 if the row is cash out).
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Cash out (0 if the row is cash in).
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
    /// Cash in hand after this row (signed: cash can go below 0, PRD FR-6.7). Null when the list
    /// is sorted by a column other than the day: the running cash shows in day order only (plan
    /// 38 B2). The screen then says "balance shows in date order only".
    #[ts(type = "Paisa | null")]
    pub running_paisa: Option<Paisa>,
    /// The record the row opens, so a row opens its entry (PRD FR-19.4): a بکری, a voucher, a
    /// trip purchase, a trip or a garden (for its cost, installment, sale in another shop or
    /// closing), fruit taken, a month close or an account (an opening). Null only if the entry has
    /// no source. An entry that belongs to no screen of its own (`kind` `entry`) opens itself.
    pub source: Option<RowSource>,
}

/// What the range adds up to, over the whole range (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookTotals {
    /// Σ cash in.
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Σ cash out.
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
}

/// The cash book of a range, one page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Cashbook {
    /// Cash in hand at the start of the range (signed).
    #[ts(type = "Paisa")]
    pub opening_paisa: Paisa,
    /// Cash in hand at the end of the range (signed).
    #[ts(type = "Paisa")]
    pub closing_paisa: Paisa,
    /// The day of the first entry that ever moved cash (not only in this range), `YYYY-MM-DD`, or
    /// null when cash never moved. The window screen goes back to it (plan 78 E, PRD FR-16.14).
    pub first_entry_date: Option<String>,
    /// The page, by day and posting order.
    pub rows: Vec<CashbookRow>,
    /// How many rows there are in the range.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole range.
    pub totals: CashbookTotals,
    /// The count and subtotals of the months THIS PAGE touches (each whole month, not only its rows
    /// on the page), in the order of the list, when asked with `group_by_month` and the list is in
    /// day order. Figures: `in_paisa` and `out_paisa`.
    pub months: Vec<MonthSubtotal>,
}

// ---- Where the capital is (PRD FR-16.14 v1.43 and v1.50, plans 78 E and 87) ----

/// Input of `cashbook_capital_where`. The whole input may be left out: the card is then as of today.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CapitalWhereInput {
    /// The day the card is read at: every figure counts entries dated on or before it. `DD-MM-YYYY`
    /// or a shortcut. Null or empty is today. A day after today is `ERR_DATE_IN_FUTURE`. Field
    /// `as_of`.
    #[serde(default)]
    #[ts(optional)]
    pub as_of: Option<String>,
}

/// Input of `cashbook_capital_print_data`: the day and the print language (PRD FR-19.10, FR-7.36).
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct CapitalPrintInput {
    /// The day the card is read at, as in [`CapitalWhereInput`]. Null or empty is today.
    #[serde(default)]
    #[ts(optional)]
    pub as_of: Option<String>,
    /// The print language.
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    #[ts(optional)]
    pub paper: Option<Paper>,
}

/// The block of the left side a line is in (PRD FR-16.14 v1.50): what the shop has or is owed (لینا ہے
/// / پاس ہے), or what it owes (دینا ہے). No line is shown with a minus sign: a source that is below
/// 0 sits in the other block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalSide {
    /// لینا ہے / پاس ہے.
    Have,
    /// دینا ہے.
    Owe,
}

/// What a line of "where the partners' money is now" is (the left side of the card).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalWhereKind {
    /// روکڑ: cash in hand. On the `owe` side it is cash below zero: "روکڑ کم" (cash short), never a
    /// minus figure.
    Cash,
    /// What buyers owe the shop (the buyers with a debit balance, the dead amount and چکتا left
    /// out). It is the tile گاہکوں کے ذمے and the total of R20.
    BuyersOwe,
    /// What the shop holds for buyers who paid ahead (the buyers with a credit balance). `owe` side.
    BuyerCredits,
    /// ڈوبی رقم: what the people in the dead amount still owe, on its own line (PRD FR-10.10). It
    /// is the tile ڈوبی رقم and the total of R38.
    DeadAmount,
    /// Advances with sellers, still open (PRD FR-15.8).
    SellerAdvances,
    /// What sellers owe beyond their open advances (PRD FR-15.9).
    SellersOweBeyond,
    /// What the shop owes sellers (their open payables). `owe` side.
    SellersPayable,
    /// ادھار رقم: money lent and not yet returned.
    MoneyLent,
    /// What is owed to labour, transport, the munshi and stores (head 02): one line for each such
    /// account, `account` says which. `owe` side.
    OwedForCharges,
    /// الگ کھاتے: separate khatas (`have`: the shop paid out for them; `owe`: it holds money for them).
    SeparateKhatas,
    /// The gardens' money held until closing. `owe` side.
    GardensHeld,
    /// Linked money already in the partners' totals (the contra account). `owe` side.
    LinkedInPartnerTotals,
    /// Any other account of the left side. Only when not 0.
    Other,
}

/// The page a line opens (PRD FR-16.14 v1.50): the frontend maps each to its route. The line's
/// `account` or `partner` says which record, where the page needs one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalLink {
    /// روکڑ بک.
    Cashbook,
    /// The buyer dues page (FR-9.4).
    BuyerDues,
    /// The buyers' ledgers (پارٹی کھاتے, buyers).
    BuyerLedgers,
    /// The seller dues page (FR-9.6): the side is told by the line's `side`.
    SellerDues,
    /// R38, the dead amount.
    DeadAmount,
    /// R26, the advances.
    Advances,
    /// R32, money lent.
    MoneyLent,
    /// The separate khatas list (الگ کھاتہ).
    SeparateKhatas,
    /// The gardens list.
    Gardens,
    /// R13, the ledger of the line's `account`.
    AccountLedger,
    /// The statement of the line's `partner` (R31, FR-17.12).
    PartnerStatement,
    /// The partner overview.
    Partners,
    /// R18, the trial balance.
    TrialBalance,
}

/// One line of the left side. `amount_paisa` is never below 0: the block says which way it goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhereLine {
    /// What it is.
    pub kind: CapitalWhereKind,
    /// The block it is in.
    pub side: CapitalSide,
    /// The amount, 0 or more.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The page it opens.
    pub link: CapitalLink,
    /// The account of a line that is one account (`owed_for_charges`), else null.
    pub account: Option<RecordRef>,
}

/// What a line of "whose it is" (the right side of the card) is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalWhoseKind {
    /// A partner's کل رقم (`partner` says whose). It can be below 0 (he took more than he has).
    PartnerTotal,
    /// ابتدائی سرمایہ (the firm's, account 06010001).
    OpeningCapital,
    /// The partners' parts of the advances to sellers still open (PRD FR-17.13).
    AdvancesOpen,
    /// Any other difference. Only when not 0; the books balance, so it shows a bug.
    Other,
}

/// One line of the right side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhoseLine {
    /// What it is.
    pub kind: CapitalWhoseKind,
    /// The partner of a `partner_total` line. `code` is empty.
    pub partner: Option<RecordRef>,
    /// The account of an `opening_capital` line (06010001), the target of its ledger link; null for
    /// the others.
    pub account: Option<RecordRef>,
    /// The amount (signed: a partner who took more than he has shows below 0).
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// This line's share of `whose_lines_total_paisa`, in basis points (hundredths of a percent, 3,000
    /// is 30%), worked out with the shared `hundredths` (half up). Null when the line or the total is
    /// not above 0. Only for a bar and a label; it is never posted and the shares need not add to
    /// 10,000 (a line below 0 has none).
    #[ts(type = "number | null")]
    pub share_bps: Option<i32>,
    /// The page it opens: the partner's statement, the ledger of ابتدائی سرمایہ, or R26.
    pub link: CapitalLink,
}

/// Which side of ابتدائی فرق an amount of it stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalDifferenceSide {
    /// بنام: capital or money counted in an opening whose other side was not placed anywhere (for
    /// example a partner's opening capital that came in against ابتدائی فرق: "رقم کا مقام درج نہیں").
    Dr,
    /// جمع: the openings of the old books left more owed to the shop than the capital explains
    /// ("پرانے کھاتوں کا فرق").
    Cr,
}

/// What made an amount on ابتدائی فرق.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalDifferenceKind {
    /// A partner's opening capital (his 0602 account). The fix `partners_place_opening_capital`
    /// moves these into روکڑ.
    PartnerCapital,
    /// The opening of any other account: a buyer's, a seller's, the cash account's.
    AccountOpening,
    /// An entry that is not an opening (a جنرل line on ابتدائی فرق).
    OtherEntry,
}

/// One entry's part of ابتدائی فرق.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalDifferencePart {
    /// The account whose opening it is (null for an entry that is not an opening).
    pub account: Option<RecordRef>,
    /// The partner, for a `partner_capital` part. `code` is empty.
    pub partner: Option<RecordRef>,
    /// The day of the entry, `YYYY-MM-DD`.
    pub date: String,
    /// The amount, 0 or more.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Which side of ابتدائی فرق it is on.
    pub side: CapitalDifferenceSide,
    /// True for a partner opening capital that the fix can move into روکڑ.
    pub can_place: bool,
    /// The page it opens: R13 of `account`.
    pub link: CapitalLink,
}

/// The parts of one kind, with what they come to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalDifferenceGroup {
    /// What made them.
    pub kind: CapitalDifferenceKind,
    /// What the parts come to (their net), 0 or more. Groups that net to 0 are left out.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Which side the net is on.
    pub side: CapitalDifferenceSide,
    /// Each entry, by day then account.
    pub parts: Vec<CapitalDifferencePart>,
}

/// ابتدائی فرق when it is not 0 (PRD FR-16.14 v1.50): a warning, not a row of the card.
///
/// Example (Umar's data, 20-11-2026): two partner opening capitals of 500,000 each are on ابتدائی فرق
/// بنام 1,000,000, and the party openings (buyers 105,000, sellers 40,000) come to 65,000 جمع:
/// the net is 935,000 بنام.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalDifference {
    /// The net, 0 or more.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Which side the net is on.
    pub side: CapitalDifferenceSide,
    /// The kinds, partner capital first.
    pub groups: Vec<CapitalDifferenceGroup>,
    /// The partners whose opening capital sits on ابتدائی فرق: the fix button
    /// `partners_place_opening_capital` offers them. Empty when none.
    pub placeable_partners: Vec<RecordRef>,
}

/// The card on the روکڑ page: where the partners' money is, and whose it is. Behind the partner code
/// (`ERR_PARTNER_LOCKED`). The two sides are always equal, because the trial balance balances: the
/// answer is refused (`ERR_INTERNAL`, a data error in the log) when they are not.
///
/// Example (after Events 1 to 5): have 58,484 + 21,564 = 80,048; owe 10,160 + 4,400 = 14,560; صافی
/// 65,488. Whose: partners −1,004 + 746 + 746 and ابتدائی سرمایہ 65,000 = 65,488.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhere {
    /// The day the card is read at, `YYYY-MM-DD`.
    pub as_of: String,
    /// لینا ہے / پاس ہے: the lines, in the order of the card.
    pub have_lines: Vec<CapitalWhereLine>,
    /// Σ of the `have` lines.
    #[ts(type = "Paisa")]
    pub have_total_paisa: Paisa,
    /// دینا ہے: the lines, in the order of the card.
    pub owe_lines: Vec<CapitalWhereLine>,
    /// Σ of the `owe` lines.
    #[ts(type = "Paisa")]
    pub owe_total_paisa: Paisa,
    /// صافی, the headline of the card: the difference between `have_total_paisa` and
    /// `owe_total_paisa`, never below 0 (the card shows no minus figure). `net_side` says which way:
    /// `have` when the shop holds more than it owes, `owe` when it holds less.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
    /// `have` when `have_total_paisa` is at least `owe_total_paisa`, else `owe`.
    pub net_side: CapitalSide,
    /// Whose it is: the partners, ابتدائی سرمایہ, the open advances, in that order.
    pub whose_lines: Vec<CapitalWhoseLine>,
    /// Σ of `whose_lines`, never below 0; `whose_lines_side` says which way. A partner's own line
    /// can still be below 0 (see `CapitalWhoseLine`).
    #[ts(type = "Paisa")]
    pub whose_lines_total_paisa: Paisa,
    /// `have` when the sum of the lines is 0 or more, else `owe`.
    pub whose_lines_side: CapitalSide,
    /// ابتدائی فرق when it is not 0, else null.
    pub difference: Option<CapitalDifference>,
    /// The whole right side: the lines plus ابتدائی فرق (جمع is plus, بنام is minus). Always equal to
    /// `net_paisa`, on the side `net_side`.
    #[ts(type = "Paisa")]
    pub whose_total_paisa: Paisa,
    /// Always true: a card whose two sides differ is refused, never shown.
    pub sides_equal: bool,
}
