//! The steps of a random run: one variant per kind of posting, each through the real service
//! function. A refusal (`ServiceError::Rejected`) is a legal answer to a random step and leaves
//! nothing behind (checked by the caller). A refusal that no valid input can earn
//! (`ERR_INTERNAL`, an unbalanced journal, overflow ...) and any other kind of error is a bug.

use mandi_services::bikri;
use mandi_services::core::arrivals::{ArrivalCancelInput, ArrivalInput};
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, CarriedLotInput, ChuktaInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, LaterSaleSaveInput, LineChargeMethod, MixedSaleSaveInput, StockListInput,
    UndoInput, WriteOffInput,
};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::dead::{DeadMarkInput, DeadReason, DeadUndoInput};
use mandi_services::core::edit::{
    DayFixCloseInput, DayFixOpenInput, GardenEntryEditInput, GardenHeaderEditInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_services::core::fruit::{
    FruitCancelInput, FruitEditInput, FruitInput, FruitLotsInput, FruitSource,
};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryCancelInput, GardenEntryInput, GardenInput, GardenOtherSaleInput,
    GardenPaymentInput, GardenRefundInput, GardenReopenInput, GardenSetPartnersInput,
};
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::khata::{KhataInInput, KhataOutInput};
use mandi_services::core::lent::{LentReturnMode, MoneyLentInput, MoneyLentReturnInput};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::purchases::{
    PurchaseGetInput, PurchaseInput, PurchaseWriteOffInput, PurchaseWriteOffUndoInput,
};
use mandi_services::core::trips::{OtherSaleInput, TripCostInput, TripEntryCancelInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, DayReopenInput, PaymentInput, ReceiptInput, RemovedPaymentFields,
    VoucherCancelInput, VoucherEditInput, VoucherEntry,
};
use mandi_services::{
    ServiceError, day as days, dead, edit, fruit, gardens, partners, purchases, trips, vouchers,
};

use crate::checks;
use crate::common::raw;
use crate::support;
use crate::universe::{U, make_second_garden, new_account_with_opening, partner_inputs};

/// One line of a new بکری: which buyer, how many, at what rate, and what is paid on the spot.
#[derive(Debug, Clone)]
pub struct Line {
    pub buyer: u8,
    pub qty: u8,
    pub rate: u16,
    pub paid: bool,
}

/// One step of a run. `day` picks 10, 11 or 12 October 2026 (all open; today is the 12th).
#[derive(Debug, Clone)]
pub enum Step {
    Sale {
        seller: u8,
        item: u8,
        lines: Vec<Line>,
        extra: u8,
        cash: u16,
        freight: u16,
        adjust: bool,
        day: u8,
    },
    Later {
        pick: u8,
        qty: u8,
        rate: u16,
        buyer: u8,
        day: u8,
    },
    Mixed {
        pick: u8,
        qty: u8,
        rate: u16,
        new_qty: u8,
        new_rate: u16,
        day: u8,
    },
    EditSale {
        pick: u8,
        change: u8,
        value: u16,
    },
    CancelSale {
        pick: u8,
    },
    UndoSale {
        pick: u8,
    },
    WriteOff {
        pick: u8,
    },
    Receipt {
        who: u8,
        amount: u16,
        day: u8,
    },
    Payment {
        who: u8,
        amount: u16,
        day: u8,
    },
    Advance {
        who: u8,
        amount: u16,
        day: u8,
    },
    Expense {
        amount: u16,
        day: u8,
    },
    Take {
        partner: u8,
        amount: u16,
        day: u8,
    },
    Return {
        partner: u8,
        amount: u16,
        day: u8,
    },
    CapitalIn {
        partner: u8,
        amount: u16,
        day: u8,
    },
    CapitalOut {
        partner: u8,
        amount: u16,
        day: u8,
    },
    General {
        amount: u16,
        day: u8,
    },
    Lend {
        person: u8,
        amount: u16,
        day: u8,
    },
    LendBack {
        person: u8,
        amount: u16,
        salary: bool,
        day: u8,
    },
    KhataOut {
        khata: u8,
        amount: u16,
        day: u8,
    },
    KhataIn {
        khata: u8,
        amount: u16,
        day: u8,
    },
    VoucherEdit {
        pick: u8,
        amount: u16,
    },
    VoucherCancel {
        pick: u8,
    },
    TripBuy {
        qty: u8,
        rate: u16,
        day: u8,
    },
    TripCost {
        amount: u16,
        day: u8,
    },
    OwnSale {
        pick: u8,
        qty: u8,
        rate: u16,
        day: u8,
    },
    TripOther {
        pick: u8,
        qty: u8,
        amount: u16,
        day: u8,
    },
    TripWriteOff {
        pick: u8,
        qty: u8,
    },
    TripWriteOffUndo {
        pick: u8,
    },
    TripCancel {
        pick: u8,
    },
    GInstall {
        amount: u16,
        day: u8,
    },
    GCost {
        amount: u16,
        day: u8,
    },
    GOther {
        qty: u8,
        amount: u16,
        day: u8,
    },
    GOwnSale {
        qty: u8,
        rate: u16,
        day: u8,
    },
    GRelief {
        cut: u16,
    },
    GRefund {
        amount: u16,
        day: u8,
    },
    GCancel {
        pick: u8,
    },
    GClose {
        day: u8,
    },
    GReopen,
    Fruit {
        kind: u8,
        pick: u8,
        qty: u8,
        rate: u16,
        day: u8,
    },
    FruitEdit {
        pick: u8,
        qty: u8,
    },
    FruitCancel {
        pick: u8,
    },
    NewAccount {
        seller: bool,
        debit: bool,
        amount: u16,
        advance: bool,
    },
    Dead {
        pick: u8,
    },
    DeadUndo {
        pick: u8,
    },
    Shares {
        weights: [u16; 3],
    },
    /// Closes the days up to the 10th or the 11th (PRD FR-21.1).
    DayClose {
        day: u8,
    },
    /// A plain reopen of the 10th or the 11th with a sign: `sign` 0 to 2 are the Owner's sign in
    /// three spellings, 3 a wrong sign, 4 no sign (PRD FR-21.2, plan 80 Q-R2-2).
    Reopen {
        day: u8,
        sign: u8,
    },
    /// Reopen to fix (PRD FR-21.4): reopen the 10th or the 11th with a sign, edit a بکری inside the
    /// fix, close the same days again.
    FixCycle {
        day: u8,
        pick: u8,
        change: u8,
        value: u16,
        sign: u8,
    },
    /// A new بکری, then an undo straight after it (must work); with `edit` the بکری is edited first
    /// and the undo is refused (PRD FR-7.47, plan 80 R2-2).
    SaleUndo {
        seller: u8,
        item: u8,
        qty: u8,
        rate: u16,
        day: u8,
        edit: bool,
    },
    /// The rent of the garden is raised (PRD FR-12.51, plan 80 R4-3: refused when the owner's
    /// refunds would be above what he could owe).
    GRaise {
        by: u16,
    },
    /// An installment of the garden is edited to another amount (same rule).
    GInstallEdit {
        pick: u8,
        amount: u16,
    },
    /// A second garden: its shares change while it has no entry, goods arrive, and a change is then
    /// refused (PRD FR-12.29, plan 80 Q-R4-1).
    GArrivalLock {
        weights: [u16; 3],
    },
}

impl Step {
    /// The name the tally counts under.
    pub fn name(&self) -> &'static str {
        match self {
            Step::Sale { .. } => "sale",
            Step::Later { .. } => "later_sale",
            Step::Mixed { .. } => "mixed_carried_and_new",
            Step::EditSale { .. } => "edit_sale",
            Step::CancelSale { .. } => "cancel_sale",
            Step::UndoSale { .. } => "undo_sale",
            Step::WriteOff { .. } => "write_off_lot",
            Step::Receipt { .. } => "receipt",
            Step::Payment { .. } => "payment",
            Step::Advance { .. } => "advance",
            Step::Expense { .. } => "shop_expense",
            Step::Take { .. } => "partner_take",
            Step::Return { .. } => "partner_return",
            Step::CapitalIn { .. } => "capital_in",
            Step::CapitalOut { .. } => "capital_out",
            Step::General { .. } => "general",
            Step::Lend { .. } => "money_lent",
            Step::LendBack { .. } => "money_lent_return",
            Step::KhataOut { .. } => "khata_out",
            Step::KhataIn { .. } => "khata_in",
            Step::VoucherEdit { .. } => "voucher_edit",
            Step::VoucherCancel { .. } => "voucher_cancel",
            Step::TripBuy { .. } => "trip_purchase",
            Step::TripCost { .. } => "trip_cost",
            Step::OwnSale { .. } => "trip_own_sale",
            Step::TripOther { .. } => "trip_other_shop_sale",
            Step::TripWriteOff { .. } => "trip_write_off",
            Step::TripWriteOffUndo { .. } => "trip_write_off_undo",
            Step::TripCancel { .. } => "trip_entry_cancel",
            Step::GInstall { .. } => "garden_installment",
            Step::GCost { .. } => "garden_cost",
            Step::GOther { .. } => "garden_other_shop_sale",
            Step::GOwnSale { .. } => "garden_own_sale",
            Step::GRelief { .. } => "garden_rent_relief",
            Step::GRefund { .. } => "garden_refund",
            Step::GCancel { .. } => "garden_entry_cancel",
            Step::GClose { .. } => "garden_close",
            Step::GReopen => "garden_reopen",
            Step::Fruit { .. } => "fruit_taken",
            Step::FruitEdit { .. } => "fruit_edit",
            Step::FruitCancel { .. } => "fruit_cancel",
            Step::NewAccount { .. } => "new_account_opening",
            Step::Dead { .. } => "dead_amount",
            Step::DeadUndo { .. } => "dead_undo",
            Step::Shares { .. } => "share_change",
            Step::DayClose { .. } => "day_close",
            Step::Reopen { sign, .. } => {
                if sign_text(*sign).1 {
                    "day_reopen"
                } else {
                    "day_reopen_wrong_sign"
                }
            }
            Step::FixCycle { sign, .. } => {
                if sign_text(*sign).1 {
                    "day_reopen_and_fix"
                } else {
                    "day_reopen_and_fix_wrong_sign"
                }
            }
            Step::SaleUndo { edit, .. } => {
                if *edit {
                    "sale_edit_then_undo"
                } else {
                    "sale_then_undo"
                }
            }
            Step::GRaise { .. } => "garden_rent_raise",
            Step::GInstallEdit { .. } => "garden_installment_edit",
            Step::GArrivalLock { .. } => "garden_arrival_locks_shares",
        }
    }
}

/// How a step ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// It saved.
    Done,
    /// Nothing to act on yet (nothing to cancel, no lot with stock ...).
    Skipped,
    /// The service refused it, with this code.
    Refused(ErrorCode),
}

/// Codes that no valid input of this run can earn: a refusal with one of them is a bug of the
/// code, not an answer to a random step.
const NEVER: &[ErrorCode] = &[
    ErrorCode::ErrInternal,
    ErrorCode::ErrJournalUnbalanced,
    ErrorCode::ErrTrialNotBalanced,
    ErrorCode::ErrOverflow,
    ErrorCode::ErrSharesNot100,
    ErrorCode::ErrNotImplemented,
    ErrorCode::ErrNotLoggedIn,
    ErrorCode::ErrSessionLocked,
    ErrorCode::ErrDbBusy,
    ErrorCode::ErrDataCheckFailed,
    ErrorCode::ErrGeneralNotBalanced,
    ErrorCode::ErrVersionChanged,
];

type R = Result<bool, ServiceError>;

fn day(d: u8) -> &'static str {
    ["10-10-2026", "11-10-2026", "12-10-2026"][usize::from(d % 3)]
}

fn pick<T: Copy>(items: &[T], at: u8) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        items.get(usize::from(at) % items.len()).copied()
    }
}

fn ids(u: &U, sql: &str) -> Vec<i64> {
    let conn = raw(&u.w.dir);
    let mut s = conn.prepare(sql).unwrap();
    s.query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Debits less credits of an account, in whole rupees.
fn net_dr(u: &U, account: i64) -> i64 {
    raw(&u.w.dir)
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa - cr_paisa), 0) / 100 FROM journal_lines WHERE account_id = ?1",
            [account],
            |r| r.get(0),
        )
        .unwrap()
}

/// An amount that fits `open` when something is open (so the step can save), else the typed one
/// (so a return with nothing to return is still tried, and refused).
fn sized(typed: u16, open: i64) -> String {
    if open > 0 {
        (1 + i64::from(typed) % open).to_string()
    } else {
        typed.to_string()
    }
}

fn version(u: &U, table: &str, id: i64) -> i64 {
    raw(&u.w.dir)
        .query_row(
            &format!("SELECT version FROM {table} WHERE id = {id}"),
            [],
            |r| r.get(0),
        )
        .unwrap()
}

/// The day of an ISO date as typed.
fn typed(iso: &str) -> String {
    format!("{}-{}-{}", &iso[8..10], &iso[5..7], &iso[0..4])
}

/// Runs one step and says how it ended. Panics on anything that is not a legal answer.
pub fn exec(u: &mut U, step: &Step) -> Out {
    u.allowed = None;
    let result = run(u, step);
    // A step may name ONE refusal that it has just proved to be the right answer (an undo of a
    // بکری that had a later sale, ERR_VERSION_CHANGED, PRD FR-7.47). Nothing else is excused.
    let allowed = u.allowed.take();
    match result {
        Ok(true) => Out::Done,
        Ok(false) => Out::Skipped,
        Err(ServiceError::Rejected(items)) => {
            let code = items.first().map_or(ErrorCode::ErrInternal, |i| i.code);
            assert!(
                !items
                    .iter()
                    .any(|i| NEVER.contains(&i.code) && Some(i.code) != allowed),
                "{}: a refusal no valid input can earn: {items:?}",
                step.name()
            );
            Out::Refused(code)
        }
        Err(other) => panic!("{}: not a refusal but a failure: {other:?}", step.name()),
    }
}

#[allow(clippy::too_many_lines)]
fn run(u: &mut U, step: &Step) -> R {
    match step {
        Step::Sale {
            seller,
            item,
            lines,
            extra,
            cash,
            freight,
            adjust,
            day: d,
        } => sale(
            u, *seller, *item, lines, *extra, *cash, *freight, *adjust, *d,
        ),
        Step::Later {
            pick: p,
            qty,
            rate,
            buyer,
            day: d,
        } => later(u, *p, *qty, *rate, *buyer, *d),
        Step::Mixed {
            pick: p,
            qty,
            rate,
            new_qty,
            new_rate,
            day: d,
        } => mixed(u, *p, *qty, *rate, *new_qty, *new_rate, *d),
        Step::EditSale {
            pick: p,
            change,
            value,
        } => edit_sale(u, *p, *change, *value),
        Step::CancelSale { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            let v = version(u, "bikri_entries", id);
            bikri::cancel(
                &u.w.env.ctx,
                &u.w.owner,
                &CancelInput {
                    id,
                    version: v,
                    reason: "x".into(),
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::UndoSale { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            undo_checked(u, id)
        }
        Step::WriteOff { pick: p } => {
            let stock = bikri::stock_list(&u.w.env.ctx, &StockListInput::default())?;
            let Some(row) = pick(&stock.rows.iter().map(|r| r.lot_id).collect::<Vec<_>>(), *p)
            else {
                return Ok(false);
            };
            bikri::write_off(
                &u.w.env.ctx,
                &u.w.owner,
                &WriteOffInput {
                    lot_id: row,
                    qty: "1".into(),
                    reason: "خراب".into(),
                },
            )?;
            Ok(true)
        }
        Step::Receipt {
            who,
            amount,
            day: d,
        } => {
            let Some(account) = pick(&u.parties(), *who) else {
                return Ok(false);
            };
            vouchers::save_receipt(
                &u.w.env.ctx,
                &u.w.owner,
                &ReceiptInput {
                    date: day(*d).into(),
                    account_id: account,
                    amount: rupees(*amount),
                    removed: RemovedPaymentFields::default(),
                    collector_staff_id: None,
                    narration: String::new(),
                },
            )?;
            Ok(true)
        }
        Step::Payment {
            who,
            amount,
            day: d,
        } => {
            let Some(account) = pick(&u.parties(), *who) else {
                return Ok(false);
            };
            vouchers::save_payment(
                &u.w.env.ctx,
                &u.w.owner,
                &PaymentInput {
                    date: day(*d).into(),
                    account_id: account,
                    amount: rupees(*amount),
                    removed: RemovedPaymentFields::default(),
                    reason: "x".into(),
                    narration: String::new(),
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::Advance {
            who,
            amount,
            day: d,
        } => {
            let Some(seller) = pick(&u.sellers(), *who) else {
                return Ok(false);
            };
            vouchers::save_advance(
                &u.w.env.ctx,
                &u.w.owner,
                &AdvanceInput {
                    date: day(*d).into(),
                    seller_account_id: seller,
                    amount: rupees(*amount),
                    removed: RemovedPaymentFields::default(),
                    expected_goods: String::new(),
                    narration: String::new(),
                    confirm_cash_below_zero: true,
                    reason: None,
                },
            )?;
            Ok(true)
        }
        Step::Expense { amount, day: d } => {
            vouchers::save_shop_expense(
                &u.w.env.ctx,
                &u.w.owner,
                &ShopExpenseInput {
                    date: day(*d).into(),
                    expense_head_id: Some(u.head_id),
                    amount: rupees(*amount),
                    paid_to: None,
                    narration: None,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::Take {
            partner,
            amount,
            day: d,
        } => {
            vouchers::save_personal_expense(
                &u.w.env.ctx,
                &u.w.owner,
                &PersonalExpenseInput {
                    date: day(*d).into(),
                    partner_id: pick(&u.w.partners, *partner),
                    amount: rupees(*amount),
                    reason: "home".into(),
                    return_by: None,
                    taken_by: None,
                    narration: None,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::Return {
            partner,
            amount,
            day: d,
        } => {
            // A partner who has money out when there is one, else the one picked (refused).
            let open_of = |p: i64| -> i64 {
                raw(&u.w.dir)
                    .query_row(
                        "SELECT COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) / 100 FROM journal_lines l
                         JOIN partners p ON p.drawings_account_id = l.account_id WHERE p.id = ?1",
                        [p],
                        |r| r.get::<_, i64>(0),
                    )
                    .unwrap()
            };
            let owing: Vec<i64> =
                u.w.partners
                    .iter()
                    .copied()
                    .filter(|p| open_of(*p) > 0)
                    .collect();
            let partner_id = if owing.is_empty() {
                pick(&u.w.partners, *partner)
            } else {
                pick(&owing, *partner)
            };
            let open = partner_id.map_or(0, open_of);
            vouchers::save_personal_return(
                &u.w.env.ctx,
                &u.w.owner,
                &PersonalReturnInput {
                    // On the last day, so no day of the run ends with returns above the takes.
                    date: day(if open > 0 { 2 } else { *d }).into(),
                    partner_id,
                    amount: sized(*amount, open),
                    narration: None,
                },
            )?;
            Ok(true)
        }
        Step::CapitalIn {
            partner,
            amount,
            day: d,
        }
        | Step::CapitalOut {
            partner,
            amount,
            day: d,
        } => {
            let input = CapitalInput {
                date: day(*d).into(),
                partner_id: pick(&u.w.partners, *partner),
                amount: rupees(*amount),
                narration: None,
                confirm_cash_below_zero: true,
            };
            if matches!(step, Step::CapitalIn { .. }) {
                vouchers::save_capital_in(&u.w.env.ctx, &u.w.owner, &input)?;
            } else {
                vouchers::save_capital_out(&u.w.env.ctx, &u.w.owner, &input)?;
            }
            Ok(true)
        }
        Step::General { amount, day: d } => {
            let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
                account_id: Some(account),
                dr: dr.into(),
                cr: cr.into(),
                narration: None,
            };
            let text = rupees(*amount);
            vouchers::general_save(
                &u.w.env.ctx,
                &u.w.owner,
                &GeneralInput {
                    date: day(*d).into(),
                    narration: None,
                    lines: vec![
                        line(u.head_account, &text, ""),
                        line(support::id_of(&u.w.env, "01010001"), "", &text),
                    ],
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::Lend {
            person,
            amount,
            day: d,
        } => {
            vouchers::save_money_lent(
                &u.w.env.ctx,
                &u.w.owner,
                &MoneyLentInput {
                    date: day(*d).into(),
                    person_account_id: pick(&u.persons, *person),
                    amount: rupees(*amount),
                    reason: "need".into(),
                    return_by: None,
                    narration: None,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::LendBack {
            person,
            amount,
            salary,
            day: d,
        } => {
            let person_id = pick(&u.persons, *person);
            let open = person_id.map_or(0, |p| net_dr(u, p));
            vouchers::save_money_lent_return(
                &u.w.env.ctx,
                &u.w.owner,
                &MoneyLentReturnInput {
                    date: day(if open > 0 { 2 } else { *d }).into(),
                    person_account_id: person_id,
                    amount: sized(*amount, open),
                    // A salary deduction is not allowed for a loan linked to a partner (FR-16.13).
                    mode: if *salary && usize::from(*person) % 2 == 0 {
                        LentReturnMode::Salary
                    } else {
                        LentReturnMode::Cash
                    },
                    salary_head_id: None,
                    narration: None,
                },
            )?;
            Ok(true)
        }
        Step::KhataOut {
            khata,
            amount,
            day: d,
        } => {
            vouchers::save_khata_out(
                &u.w.env.ctx,
                &u.w.owner,
                &KhataOutInput {
                    date: day(*d).into(),
                    khata_account_id: pick(&u.khatas, *khata),
                    amount: rupees(*amount),
                    reason: "family".into(),
                    narration: None,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::KhataIn {
            khata,
            amount,
            day: d,
        } => {
            vouchers::save_khata_in(
                &u.w.env.ctx,
                &u.w.owner,
                &KhataInInput {
                    date: day(*d).into(),
                    khata_account_id: pick(&u.khatas, *khata),
                    amount: rupees(*amount),
                    reason: "back".into(),
                    narration: None,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::VoucherEdit { pick: p, amount } => voucher_edit(u, *p, *amount),
        Step::VoucherCancel { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM vouchers WHERE status = 'saved' ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            let v = version(u, "vouchers", id);
            vouchers::cancel(
                &u.w.env.ctx,
                &u.w.owner,
                &VoucherCancelInput {
                    id,
                    version: v,
                    reason: "mistake".into(),
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::TripBuy { qty, rate, day: d } => {
            let saved = purchases::save(
                &u.w.env.ctx,
                &u.w.owner,
                &PurchaseInput {
                    // Half of the goods are set aside for another shop (PRD FR-12.41, v1.52): a
                    // sale in another shop takes only from that part, a sale in our shop only from
                    // the rest.
                    other_qty: (*qty >= 2).then(|| (*qty / 2).to_string()),
                    shop_name: (*qty >= 2).then(|| "Haji Traders".to_owned()),
                    sell_date: None,
                    trip_id: Some(u.trip),
                    date: day(*d).into(),
                    supplier: None,
                    item_id: Some(u.orange),
                    packing_id: Some(u.w.box_),
                    qty: qty.to_string(),
                    weight: None,
                    rate: rate.to_string(),
                    freight: None,
                    labour: None,
                    other_cost: None,
                    note: None,
                    parts: Vec::new(),
                    confirm_cash_below_zero: true,
                },
            )?;
            u.purchases.push(saved.id);
            Ok(true)
        }
        Step::TripCost { amount, day: d } => {
            trips::cost_save(
                &u.w.env.ctx,
                &u.w.owner,
                &TripCostInput {
                    trip_id: Some(u.trip),
                    date: day(*d).into(),
                    amount: rupees(*amount),
                    reason: "truck".into(),
                    parts: Vec::new(),
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::OwnSale {
            pick: p,
            qty,
            rate,
            day: d,
        } => own_sale(u, *p, *qty, *rate, *d),
        Step::TripOther {
            pick: p,
            qty,
            amount,
            day: d,
        } => {
            let live = live_purchases(u);
            let Some(purchase) = pick(&live, *p) else {
                return Ok(false);
            };
            // A sale in another shop takes only the goods set aside for it when the purchase was
            // made (PRD FR-12.41, v1.52); with none set aside the step has nothing to act on.
            let row = purchases::get(&u.w.env.ctx, &PurchaseGetInput { id: purchase })?;
            let left = row.qty_other_left;
            if left == 0 {
                return Ok(false);
            }
            trips::other_sale_save(
                &u.w.env.ctx,
                &u.w.owner,
                &OtherSaleInput {
                    trip_id: Some(u.trip),
                    date: day(*d).into(),
                    shop: Some("Haji Traders".into()),
                    purchase_id: Some(purchase),
                    qty: sized(u16::from(*qty), left),
                    amount: rupees(*amount),
                    note: None,
                },
            )?;
            Ok(true)
        }
        Step::TripWriteOff { pick: p, qty } => {
            let live = live_purchases(u);
            let Some(purchase) = pick(&live, *p) else {
                return Ok(false);
            };
            let left = purchases::get(&u.w.env.ctx, &PurchaseGetInput { id: purchase })?.qty_left;
            purchases::write_off(
                &u.w.env.ctx,
                &u.w.owner,
                &PurchaseWriteOffInput {
                    purchase_id: purchase,
                    qty: sized(u16::from(*qty), left),
                    reason: "خراب".into(),
                },
            )?;
            Ok(true)
        }
        Step::TripWriteOffUndo { pick: p } => {
            let live = live_purchases(u);
            let mut moves: Vec<i64> = Vec::new();
            for purchase in live {
                let row = purchases::get(&u.w.env.ctx, &PurchaseGetInput { id: purchase })?;
                moves.extend(row.write_offs.iter().map(|w| w.move_id));
            }
            let Some(move_id) = pick(&moves, *p) else {
                return Ok(false);
            };
            purchases::write_off_undo(
                &u.w.env.ctx,
                &u.w.owner,
                &PurchaseWriteOffUndoInput {
                    move_id,
                    reason: "undo".into(),
                },
            )?;
            Ok(true)
        }
        Step::TripCancel { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM venture_entries WHERE status = 'saved' AND kind IN ('cost', 'other_sale')
                 AND venture_id = (SELECT id FROM ventures WHERE kind = 'trip') ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            let kind: String = raw(&u.w.dir)
                .query_row(
                    "SELECT kind FROM venture_entries WHERE id = ?1",
                    [id],
                    |r| r.get(0),
                )
                .unwrap();
            let input = TripEntryCancelInput {
                id,
                version: version(u, "venture_entries", id),
                reason: "cancel".into(),
                confirm_cash_below_zero: true,
            };
            if kind == "cost" {
                trips::cost_cancel(&u.w.env.ctx, &u.w.owner, &input)?;
            } else {
                trips::other_sale_cancel(&u.w.env.ctx, &u.w.owner, &input)?;
            }
            Ok(true)
        }
        Step::GInstall { amount, day: d } | Step::GCost { amount, day: d } => {
            let payment = GardenPaymentInput {
                garden_id: Some(u.garden),
                date: day(*d).into(),
                amount: rupees(*amount),
                reason: "payment".into(),
                parts: Vec::new(),
                confirm_cash_below_zero: true,
            };
            let entry = if matches!(step, Step::GInstall { .. }) {
                GardenEntryInput::Installment(payment)
            } else {
                GardenEntryInput::Cost(payment)
            };
            gardens::entry_save(&u.w.env.ctx, &u.w.owner, &entry)?;
            Ok(true)
        }
        Step::GOther {
            qty,
            amount,
            day: d,
        } => {
            gardens::entry_save(
                &u.w.env.ctx,
                &u.w.owner,
                &GardenEntryInput::OtherSale(GardenOtherSaleInput {
                    arrival_id: None,
                    garden_id: Some(u.garden),
                    date: day(*d).into(),
                    shop: Some("Haji Traders".into()),
                    qty: qty.to_string(),
                    amount: rupees(*amount),
                    note: None,
                }),
            )?;
            Ok(true)
        }
        Step::GOwnSale { qty, rate, day: d } => {
            let mut lot = support::with_sales(
                support::bare_lot(
                    &u.w,
                    "garden-1",
                    u.garden_account,
                    u.w.banana,
                    &qty.to_string(),
                ),
                vec![support::sale(
                    "s-1",
                    u.w.akram,
                    &qty.to_string(),
                    &rate.to_string(),
                )],
            );
            lot.adjust_advance = false;
            let mut draft = support::draft(vec![lot]);
            draft.header.entry_date = day(*d).into();
            bikri::save(
                &u.w.env.ctx,
                &u.w.owner,
                &BikriSaveInput {
                    draft,
                    confirm_cash_below_zero: true,
                },
            )?;
            Ok(true)
        }
        Step::GRelief { cut } => garden_relief(u, *cut),
        Step::GRefund { amount, day: d } => {
            let owes = gardens::get(
                &u.w.env.ctx,
                &mandi_services::core::gardens::GardenGetInput {
                    id: u.garden,
                    history_offset: 0,
                    history_limit: 1,
                },
            )?
            .row
            .owner_owes_paisa
            .rupees();
            gardens::entry_save(
                &u.w.env.ctx,
                &u.w.owner,
                &GardenEntryInput::Refund(GardenRefundInput {
                    garden_id: Some(u.garden),
                    date: day(if owes > 0 { 2 } else { *d }).into(),
                    amount: sized(*amount, owes),
                    note: None,
                }),
            )?;
            Ok(true)
        }
        Step::GCancel { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM venture_entries WHERE status = 'saved'
                 AND kind IN ('installment', 'cost', 'other_sale', 'refund')
                 AND venture_id = (SELECT id FROM ventures WHERE kind = 'garden' ORDER BY id LIMIT 1)
                 ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            garden_entry_cancel(u, id)
        }
        Step::GClose { day: d } => garden_close(u, *d),
        Step::GReopen => {
            gardens::reopen(
                &u.w.env.ctx,
                &u.w.owner,
                &GardenReopenInput {
                    id: u.garden,
                    version: version(u, "ventures", u.garden),
                    reason: "undo".into(),
                },
            )?;
            Ok(true)
        }
        Step::Fruit {
            kind,
            pick: p,
            qty,
            rate,
            day: d,
        } => fruit_take(u, *kind, *p, *qty, *rate, *d),
        Step::FruitEdit { pick: p, qty } => fruit_edit(u, *p, *qty),
        Step::FruitCancel { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM fruit_takes WHERE status = 'saved' ORDER BY id",
            );
            let Some(id) = pick(&all, *p) else {
                return Ok(false);
            };
            fruit::cancel(
                &u.w.env.ctx,
                &u.w.owner,
                &FruitCancelInput {
                    id,
                    version: version(u, "fruit_takes", id),
                    reason: "cancel".into(),
                },
            )?;
            Ok(true)
        }
        Step::NewAccount {
            seller,
            debit,
            amount,
            advance,
        } => {
            if u.made >= 8 {
                return Ok(false);
            }
            new_account_with_opening(u, *seller, *debit, i64::from(*amount), *advance)?;
            Ok(true)
        }
        Step::Dead { pick: p } => {
            let mut all = u.parties();
            all.extend(u.persons);
            // Someone who owes the shop, when there is one (else any, which is refused).
            let owing: Vec<i64> = all.iter().copied().filter(|a| net_dr(u, *a) > 0).collect();
            let Some(account) = pick(if owing.is_empty() { &all } else { &owing }, *p) else {
                return Ok(false);
            };
            dead::mark(
                &u.w.env.ctx,
                &u.w.owner,
                &DeadMarkInput {
                    account_id: account,
                    entry_date: "12-10-2026".into(),
                    reason: DeadReason::Died,
                    note: String::new(),
                },
            )?;
            Ok(true)
        }
        Step::DeadUndo { pick: p } => {
            let all = ids(
                u,
                "SELECT id FROM dead_marks WHERE is_active = 1 ORDER BY id",
            );
            let Some(mark_id) = pick(&all, *p) else {
                return Ok(false);
            };
            dead::undo(
                &u.w.env.ctx,
                &u.w.owner,
                &DeadUndoInput {
                    mark_id,
                    reason: "back".into(),
                },
            )?;
            Ok(true)
        }
        Step::Shares { weights } => {
            if u.shares_changed {
                return Ok(false);
            }
            let total: u32 = weights.iter().map(|w| u32::from(*w)).sum();
            let mut bps: Vec<i64> = weights
                .iter()
                .map(|w| i64::from(*w) * 10_000 / i64::from(total))
                .collect();
            let used: i64 = bps.iter().sum();
            bps[0] += 10_000 - used;
            partners::set_shares(
                &u.w.env.ctx,
                &u.w.owner,
                &PartnersSetSharesInput {
                    effective_from: "12-10-2026".into(),
                    shares: u
                        .w
                        .partners
                        .iter()
                        .zip(&bps)
                        .map(|(id, b)| ShareInput {
                            partner_id: *id,
                            share: format!("{}.{:02}", b / 100, b % 100),
                        })
                        .collect(),
                },
            )?;
            u.shares_changed = true;
            Ok(true)
        }
        Step::DayClose { day: d } => day_close(u, *d),
        Step::Reopen { day: d, sign } => day_reopen(u, *d, *sign),
        Step::FixCycle {
            day: d,
            pick: p,
            change,
            value,
            sign,
        } => fix_cycle(u, *d, *p, *change, *value, *sign),
        Step::SaleUndo {
            seller,
            item,
            qty,
            rate,
            day: d,
            edit,
        } => sale_then_undo(u, *seller, *item, *qty, *rate, *d, *edit),
        Step::GRaise { by } => garden_raise(u, *by),
        Step::GInstallEdit { pick: p, amount } => garden_installment_edit(u, *p, *amount),
        Step::GArrivalLock { weights } => arrival_lock(u, weights),
    }
}

fn rupees(v: u16) -> String {
    v.to_string()
}

fn live_purchases(u: &U) -> Vec<i64> {
    ids(
        u,
        "SELECT id FROM purchases WHERE status = 'saved' ORDER BY id",
    )
}

#[allow(clippy::too_many_arguments)]
fn sale(
    u: &mut U,
    seller: u8,
    item: u8,
    lines: &[Line],
    extra: u8,
    cash: u16,
    freight: u16,
    adjust: bool,
    d: u8,
) -> R {
    let w = &u.w;
    let sellers = u.sellers();
    let Some(seller) = pick(&sellers, seller) else {
        return Ok(false);
    };
    let item = [w.banana, w.apple, w.mooli][usize::from(item) % 3];
    let total: i64 = lines.iter().map(|l| i64::from(l.qty)).sum::<i64>() + i64::from(extra);
    let mut lot = support::bare_lot(w, "lot-1", seller, item, &total.to_string());
    lot.freight = (freight > 0).then(|| freight.to_string());
    lot.cash_paid = (cash > 0).then(|| cash.to_string());
    lot.adjust_advance = adjust;
    let mut buyers = vec![w.akram, w.arshad, w.naqdi, w.chukta_khata];
    buyers.extend(&u.extra_buyers);
    for (i, l) in lines.iter().enumerate() {
        let buyer = pick(&buyers, l.buyer).unwrap_or(w.akram);
        let mut s = support::sale(
            &format!("s-{i}"),
            buyer,
            &l.qty.to_string(),
            &l.rate.to_string(),
        );
        if buyer == w.chukta_khata {
            s.chukta = Some(ChuktaInput {
                name_ur: "شاہد".into(),
                name_en: "Shahid".into(),
                phone: "0300-0000000".into(),
            });
        }
        if l.paid {
            s.paid = Some("50".into());
        }
        lot.sales.push(s);
    }
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = day(d).into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )?;
    Ok(true)
}

fn received_day(date: &str) -> u8 {
    date[8..10].parse().unwrap_or(10)
}

fn later(u: &mut U, p: u8, qty: u8, rate: u16, buyer: u8, d: u8) -> R {
    let stock = bikri::stock_list(&u.w.env.ctx, &StockListInput::default())?;
    let Some(row) = pick(&stock.rows.iter().collect::<Vec<_>>(), p) else {
        return Ok(false);
    };
    let date = format!(
        "{:02}-10-2026",
        received_day(&row.received_date).max(10 + d % 3)
    );
    let qty = 1 + i64::from(qty) % row.balance.max(1);
    let buyers = u.buyers();
    let buyer = pick(&buyers, buyer).unwrap_or(u.w.akram);
    let owner_bikri = bikri_of_lot(u, row.lot_id);
    bikri::later_sale_save(
        &u.w.env.ctx,
        &u.w.owner,
        &LaterSaleSaveInput {
            lot_id: row.lot_id,
            date,
            sales: vec![support::sale(
                "l-1",
                buyer,
                &qty.to_string(),
                &rate.to_string(),
            )],
            confirm_cash_below_zero: true,
        },
    )?;
    // The بکری of the lot has a posting its save did not make: it can no longer be undone (FR-7.47).
    u.touched.insert(owner_bikri);
    Ok(true)
}

fn mixed(u: &mut U, p: u8, qty: u8, rate: u16, new_qty: u8, new_rate: u16, d: u8) -> R {
    let stock = bikri::stock_list(&u.w.env.ctx, &StockListInput::default())?;
    let Some(row) = pick(&stock.rows.iter().collect::<Vec<_>>(), p) else {
        return Ok(false);
    };
    let date = format!(
        "{:02}-10-2026",
        received_day(&row.received_date).max(10 + d % 3)
    );
    let carried_qty = 1 + i64::from(qty) % row.balance.max(1);
    let carried_bikri = bikri_of_lot(u, row.lot_id);
    let new_total = i64::from(new_qty % 9) + 2;
    let w = &u.w;
    let new_lot = support::with_sales(
        support::bare_lot(w, "new-1", w.karim, w.apple, &new_total.to_string()),
        vec![support::sale(
            "n-1",
            w.arshad,
            &new_total.to_string(),
            &new_rate.to_string(),
        )],
    );
    bikri::mixed_save(
        &w.env.ctx,
        &w.owner,
        &MixedSaleSaveInput {
            date,
            carried: vec![CarriedLotInput {
                key: String::new(),
                lot_id: row.lot_id,
                sales: vec![support::sale(
                    "c-1",
                    w.akram,
                    &carried_qty.to_string(),
                    &rate.to_string(),
                )],
            }],
            new_lots: vec![new_lot],
            confirm_cash_below_zero: true,
        },
    )?;
    u.touched.insert(carried_bikri);
    Ok(true)
}

/// An edit of one posting of a saved بکری: a rate, a quantity, a removed or an added line, a typed
/// commission, the money paid on the spot, a charge of the lot, or the buyer of a line.
fn edit_sale(u: &mut U, p: u8, change: u8, value: u16) -> R {
    let all = ids(
        u,
        "SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id",
    );
    let Some(bikri_id) = pick(&all, p) else {
        return Ok(false);
    };
    let detail = bikri::get(&u.w.env.ctx, bikri_id)?;
    let live: Vec<_> = detail.postings.iter().filter(|x| x.editable).collect();
    let Some(posting) = pick(&live, p.wrapping_add(change)) else {
        return Ok(false);
    };
    let opened = bikri::open_for_edit(
        &u.w.env.ctx,
        &EditOpenInput {
            bikri_id,
            entry_id: posting.entry_id,
        },
    )?;
    let mut d = opened.draft.clone();
    let Some(li) = (!d.lots.is_empty()).then(|| usize::from(change) % d.lots.len()) else {
        return Ok(false);
    };
    let first_posting = posting.posting_type == "bikri_first";
    let posting_day = posting.entry_date.clone();
    let buyers = u.buyers();
    let lot = &mut d.lots[li];
    if lot.sales.is_empty() {
        return Ok(false);
    }
    let si = usize::from(value) % lot.sales.len();
    match change % 9 {
        0 => lot.sales[si].seller_rate = (100 + value % 900).to_string(),
        1 => {
            let qty: i64 = lot.sales[si].qty.parse().unwrap_or(1);
            if qty > 1 {
                lot.sales[si].qty = (qty - 1).to_string();
            }
        }
        2 => {
            if lot.sales.len() > 1 {
                lot.sales.remove(si);
            }
        }
        3 => {
            let buyer = pick(&buyers, (value % 7) as u8).unwrap_or(u.w.akram);
            lot.sales.push(support::sale(
                &format!("n-{value}"),
                buyer,
                "1",
                &(100 + value % 900).to_string(),
            ));
        }
        4 => {
            if first_posting {
                let qty: i64 = lot.qty_received.parse().unwrap_or(1);
                lot.qty_received = (qty + 1 + i64::from(value % 3)).to_string();
            }
        }
        5 => lot.sales[si].seller_comm = support::typed(LineChargeMethod::Percent, "4"),
        6 => lot.sales[si].paid = value.is_multiple_of(2).then(|| "50".to_owned()),
        7 => {
            if first_posting {
                lot.cash_paid = Some((value % 3000).to_string()).filter(|s| s != "0");
                lot.freight = Some((value % 2000).to_string()).filter(|s| s != "0");
            }
        }
        _ => {
            // The buyer of a line (PRD FR-7.70, E2E-68), to an ordinary buyer.
            let buyer = pick(&buyers, (value % 11) as u8).unwrap_or(u.w.akram);
            lot.sales[si].buyer_account_id = Some(buyer);
            lot.sales[si].chukta = None;
        }
    }
    let preview = bikri::edit_preview(
        &u.w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: d.clone(),
        },
    )?;
    if !preview.errors.is_empty() {
        if std::env::var("T1_SHOW").is_ok() {
            eprintln!("edit_sale preview errors: {:?}", preview.errors);
        }
        return Ok(false);
    }
    let done = bikri::edit_save(
        &u.w.env.ctx,
        &u.w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft: d,
            reason: Some("edit".to_owned()),
            confirm_cash_below_zero: true,
        },
    )?;
    assert_eq!(
        done.preview, preview,
        "edit_sale: the save is not the preview (Code Standards B5)"
    );
    u.touched.insert(bikri_id);
    u.last_edit_day = Some(posting_day);
    Ok(true)
}

/// An edit of a saved voucher to another amount (same account, same day, same type).
fn voucher_edit(u: &mut U, p: u8, amount: u16) -> R {
    type VoucherLine = (i64, String, Option<i64>, Option<i64>, String);
    let rows: Vec<VoucherLine> = {
        let conn = raw(&u.w.dir);
        let mut s = conn
            .prepare(
                "SELECT id, type, account_id, partner_id, entry_date FROM vouchers
                 WHERE status = 'saved' ORDER BY id",
            )
            .unwrap();
        s.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let Some((id, kind, account, partner, iso)) =
        pick(&rows.iter().collect::<Vec<_>>(), p).cloned()
    else {
        return Ok(false);
    };
    let date = typed(&iso);
    let amount = rupees(amount);
    let entry = match kind.as_str() {
        "receipt" => VoucherEntry::Receipt(ReceiptInput {
            date,
            account_id: account.unwrap_or_default(),
            amount,
            removed: RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        }),
        "payment" => VoucherEntry::Payment(PaymentInput {
            date,
            account_id: account.unwrap_or_default(),
            amount,
            removed: RemovedPaymentFields::default(),
            reason: "x".into(),
            narration: String::new(),
            confirm_cash_below_zero: true,
        }),
        "advance" => VoucherEntry::Advance(AdvanceInput {
            date,
            seller_account_id: account.unwrap_or_default(),
            amount,
            removed: RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
            confirm_cash_below_zero: true,
            reason: None,
        }),
        "shop_expense" => VoucherEntry::ShopExpense(ShopExpenseInput {
            date,
            expense_head_id: Some(u.head_id),
            amount,
            paid_to: None,
            narration: None,
            confirm_cash_below_zero: true,
        }),
        "personal_expense" => VoucherEntry::PersonalExpense(PersonalExpenseInput {
            date,
            partner_id: partner,
            amount,
            reason: "home".into(),
            return_by: None,
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: true,
        }),
        "money_lent" => VoucherEntry::MoneyLent(MoneyLentInput {
            date,
            person_account_id: account,
            amount,
            reason: "need".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: true,
        }),
        "capital_in" | "capital_out" => {
            let input = CapitalInput {
                date,
                partner_id: partner,
                amount,
                narration: None,
                confirm_cash_below_zero: true,
            };
            if kind == "capital_in" {
                VoucherEntry::CapitalIn(input)
            } else {
                VoucherEntry::CapitalOut(input)
            }
        }
        "khata_out" => VoucherEntry::KhataOut(KhataOutInput {
            date,
            khata_account_id: account,
            amount,
            reason: "family".into(),
            narration: None,
            confirm_cash_below_zero: true,
        }),
        _ => return Ok(false),
    };
    vouchers::edit(
        &u.w.env.ctx,
        &u.w.owner,
        &VoucherEditInput {
            id,
            version: version(u, "vouchers", id),
            entry,
            reason: "edit".into(),
            sign: "adnan".into(),
            confirm_cash_below_zero: true,
        },
    )?;
    Ok(true)
}

/// A sale of trip goods in a بکری: a lot of اپنا مال taking its goods from a purchase.
fn own_sale(u: &mut U, p: u8, qty: u8, rate: u16, d: u8) -> R {
    let live = live_purchases(u);
    let Some(purchase_id) = pick(&live, p) else {
        return Ok(false);
    };
    let left = purchases::get(&u.w.env.ctx, &PurchaseGetInput { id: purchase_id })?.qty_our_left;
    if left == 0 {
        return Ok(false);
    }
    let qty = (i64::from(qty) % left) + 1;
    let w = &u.w;
    let lot = mandi_services::core::bikri::LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, &qty.to_string())
    };
    let lot = support::with_sales(
        lot,
        vec![support::sale(
            "s-1",
            w.akram,
            &qty.to_string(),
            &rate.to_string(),
        )],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = day(d).into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )?;
    Ok(true)
}

fn fruit_input(u: &U, kind: u8, lot: Option<i64>, qty: i64, rate: u16, d: u8) -> FruitInput {
    let guests = kind % 2 == 1;
    let shop = (kind / 2).is_multiple_of(2);
    FruitInput {
        for_guests: guests,
        partner_id: if guests {
            None
        } else {
            Some(u.w.partners[usize::from(kind / 4) % 3])
        },
        date: day(d).into(),
        source: if shop {
            FruitSource::Shop
        } else {
            FruitSource::OtherShop
        },
        lot_id: if shop { lot } else { None },
        agent_account_id: if shop { None } else { Some(u.agent) },
        item_id: Some(u.w.banana),
        packing_id: Some(u.w.box_),
        qty: qty.to_string(),
        weight: None,
        rate: rate.to_string(),
        note: None,
        deduction: None,
        editing_id: None,
    }
}

fn fruit_take(u: &mut U, mut kind: u8, p: u8, qty: u8, rate: u16, d: u8) -> R {
    let shop = (kind / 2).is_multiple_of(2);
    let mut lot = None;
    let mut qty = i64::from(qty) + 1;
    if shop {
        let lots = fruit::lots(
            &u.w.env.ctx,
            &FruitLotsInput {
                search: None,
                offset: 0,
                limit: 100,
            },
        )?;
        match pick(&lots.rows.iter().collect::<Vec<_>>(), p) {
            Some(row) => {
                lot = Some(row.lot_id);
                qty = 1 + (qty - 1) % row.balance.max(1);
            }
            // No lot has stock: take from the other agent's shop instead.
            None => kind = (kind & !2) | 2,
        }
    }
    let input = fruit_input(u, kind, lot, qty, rate.max(1), d);
    fruit::save(&u.w.env.ctx, &u.w.owner, &input)?;
    Ok(true)
}

fn fruit_edit(u: &mut U, p: u8, qty: u8) -> R {
    let all = ids(
        u,
        "SELECT id FROM fruit_takes WHERE status = 'saved' ORDER BY id",
    );
    let Some(id) = pick(&all, p) else {
        return Ok(false);
    };
    let row = raw(&u.w.dir)
        .query_row(
            "SELECT partner_id, source, lot_id, credit_account_id, entry_date, rate_paisa / 100 FROM fruit_takes WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, Option<i64>>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            },
        )
        .unwrap();
    let (partner, source, lot, credit, iso, rate) = row;
    let input = FruitInput {
        for_guests: partner.is_none(),
        partner_id: partner,
        date: typed(&iso),
        source: if source == "shop" {
            FruitSource::Shop
        } else {
            FruitSource::OtherShop
        },
        lot_id: lot,
        agent_account_id: (source != "shop").then_some(credit),
        item_id: Some(u.w.banana),
        packing_id: Some(u.w.box_),
        qty: (i64::from(qty) % 5 + 1).to_string(),
        weight: None,
        rate: rate.to_string(),
        note: None,
        deduction: None,
        editing_id: Some(id),
    };
    fruit::edit(
        &u.w.env.ctx,
        &u.w.owner,
        &FruitEditInput {
            id,
            version: version(u, "fruit_takes", id),
            entry: input,
            reason: "edit".into(),
        },
    )?;
    Ok(true)
}

// ---- The steps of the plan 80 gate fixes (R2-2, Q-R2-2, Q-R4-1, Q-R4-2, R4-3) ----

/// The بکری a lot belongs to.
fn bikri_of_lot(u: &U, lot_id: i64) -> i64 {
    raw(&u.w.dir)
        .query_row(
            &format!("SELECT bikri_id FROM bikri_lots WHERE id = {lot_id}"),
            [],
            |r| r.get(0),
        )
        .unwrap()
}

/// An undo, with the answer kept: see [`undo_checked`].
fn undo_full(u: &mut U, id: i64) -> Result<mandi_services::core::bikri::UndoResult, ServiceError> {
    // "Straight after the save" is the harness's own list: a بکری is touched once a later sale, a
    // carried posting or an edit was saved on it (the only things that raise its version).
    let untouched = !u.touched.contains(&id);
    let result = bikri::undo(
        &u.w.env.ctx,
        &u.w.owner,
        &UndoInput {
            id,
            confirm_cash_below_zero: true,
        },
    );
    match result {
        Ok(done) => {
            assert!(
                untouched,
                "undo_sale: a بکری that had a later sale, a carried posting or an edit was undone (PRD FR-7.47, plan 80 R2-2)"
            );
            assert_eq!(done.bikri_id, id);
            Ok(done)
        }
        Err(ServiceError::Rejected(items))
            if items.iter().any(|i| i.code == ErrorCode::ErrVersionChanged) =>
        {
            // The refusal that no valid input earns is right here, and only here: after a later
            // sale, a carried posting or an edit (PRD FR-7.47, plan 80 R2-2).
            assert!(
                !untouched,
                "undo_sale: a بکری nothing happened to since its save was refused as changed (PRD FR-7.47): {items:?}"
            );
            assert_eq!(items.len(), 1, "undo_sale: {items:?}");
            assert_eq!(
                items[0].field.as_deref(),
                Some("id"),
                "undo_sale: {items:?}"
            );
            u.allowed = Some(ErrorCode::ErrVersionChanged);
            Err(ServiceError::Rejected(items))
        }
        Err(other) => Err(other),
    }
}

/// An undo with its answer checked against PRD FR-7.47 (plan 80 R2-2): it works straight after a
/// save, and it is refused with ERR_VERSION_CHANGED once the بکری had a later sale, a carried
/// posting or an edit. Refused or not, nothing else is excused (see `exec`).
fn undo_checked(u: &mut U, id: i64) -> R {
    undo_full(u, id).map(|_| true)
}

/// A new بکری of one line and an undo straight after it. The undo must work. With `edit_first` the
/// بکری is edited before the undo, and the undo is then refused (when the edit saved).
fn sale_then_undo(
    u: &mut U,
    seller: u8,
    item: u8,
    qty: u8,
    rate: u16,
    d: u8,
    edit_first: bool,
) -> R {
    // An open day: the 12th when the day asked for is closed.
    let d = if i8::try_from(d).unwrap_or(2) <= u.closed_level {
        2
    } else {
        d
    };
    let qty = 1 + qty % 7;
    let rate = 100 + rate % 900;
    let line = Line {
        buyer: 0,
        qty,
        rate,
        paid: false,
    };
    if !sale(
        u,
        seller,
        item,
        std::slice::from_ref(&line),
        0,
        0,
        0,
        false,
        d,
    )? {
        return Ok(false);
    }
    let id: i64 = raw(&u.w.dir)
        .query_row("SELECT MAX(id) FROM bikri_entries", [], |r| r.get(0))
        .unwrap();
    assert!(!u.touched.contains(&id), "a new بکری is untouched");
    if edit_first {
        let all = ids(
            u,
            "SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id",
        );
        let at = all
            .iter()
            .position(|x| *x == id)
            .expect("the new بکری is saved");
        let p = u8::try_from(at).expect("a run has few بکری");
        // Change 0: the seller rate of its one line, to another rate.
        match edit_sale(u, p, 0, (rate - 100 + 1) % 900) {
            Ok(_) => {}
            Err(ServiceError::Rejected(items)) => legal_refusal("sale_edit_then_undo", u, &items),
            Err(other) => return Err(other),
        }
    }
    let before = (checks::fingerprint(u), checks::audit_rows(u));
    let must_work = !u.touched.contains(&id);
    match undo_full(u, id) {
        Ok(done) => {
            assert!(
                must_work,
                "sale_then_undo: an edited بکری was undone (PRD FR-7.47)"
            );
            let status: String = raw(&u.w.dir)
                .query_row(
                    &format!("SELECT status FROM bikri_entries WHERE id = {id}"),
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(status, "cancelled", "the undone بکری is cancelled");
            // The draft it gives back has the same line (PRD FR-7.47: "the same values").
            assert_eq!(done.draft.lots.len(), 1, "the draft has the one lot");
            let sales = &done.draft.lots[0].sales;
            assert_eq!(sales.len(), 1, "the draft has the one line");
            assert_eq!(sales[0].qty.replace(',', ""), qty.to_string());
            assert_eq!(sales[0].seller_rate.replace(',', ""), rate.to_string());
        }
        Err(ServiceError::Rejected(items)) => {
            assert!(
                !must_work,
                "sale_then_undo: an undo straight after the save was refused (PRD FR-7.47): {items:?}"
            );
            assert!(
                items.iter().all(|i| i.code == ErrorCode::ErrVersionChanged),
                "sale_then_undo: the undo of an edited بکری must be ERR_VERSION_CHANGED: {items:?}"
            );
            assert_eq!(
                before,
                (checks::fingerprint(u), checks::audit_rows(u)),
                "sale_then_undo: the refused undo changed something (NFR-REL-1)"
            );
        }
        Err(other) => return Err(other),
    }
    // The sale (and its edit) saved, so the step is done whatever the undo said.
    Ok(true)
}

/// A refusal of a step inside a compound step: the same rule as `exec`.
fn legal_refusal(step: &str, u: &U, items: &[mandi_services::Rejection]) {
    assert!(
        !items
            .iter()
            .any(|i| NEVER.contains(&i.code) && Some(i.code) != u.allowed),
        "{step}: a refusal no valid input can earn: {items:?}"
    );
}

/// The Owner's sign in three spellings (0, 1, 2), a wrong sign (3) and a blank one (4). Gives the
/// typed text and whether the sign is the right one (PRD FR-21.4, plan 80 Q-R2-2).
pub fn sign_text(kind: u8) -> (&'static str, bool) {
    match kind % 5 {
        0 => ("adnan", true),
        1 => ("عدنان", true),
        2 => (" ADNAN ", true),
        3 => ("munshi", false),
        _ => ("  ", false),
    }
}

/// What the harness looks at before a day step, to prove a refusal changed nothing.
fn day_snapshot(u: &U) -> (Vec<i64>, i64, Option<String>) {
    (
        checks::fingerprint(u),
        checks::audit_rows(u),
        checks::closed_up_to(u),
    )
}

/// The last audit row, as its action and every text of it joined.
fn last_audit(u: &U) -> (String, String) {
    raw(&u.w.dir)
        .query_row(
            "SELECT action, COALESCE(entity, '') || ' ' || COALESCE(old_json, '') || ' ' ||
                    COALESCE(new_json, '') || ' ' || COALESCE(reason, '')
             FROM audit_log ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}

/// A refusal of the sign of a reopen (PRD FR-21.2, FR-21.4, plan 80 Q-R2-2): nothing opens; a wrong
/// sign is audited once with no typed text; a blank sign is refused first and writes nothing.
fn refused_sign(
    u: &U,
    kind: u8,
    items: Vec<mandi_services::Rejection>,
    before: &(Vec<i64>, i64, Option<String>),
    wrong_action: &str,
) -> R {
    let (typed, right) = sign_text(kind);
    assert!(
        !right,
        "the Owner's sign was refused: {items:?} (PRD FR-21.4)"
    );
    let want = if typed.trim().is_empty() {
        ErrorCode::ErrSignRequired
    } else {
        ErrorCode::ErrSignWrong
    };
    assert!(
        items.len() == 1 && items[0].code == want && items[0].field.as_deref() == Some("sign"),
        "a {typed:?} sign must give {want:?} at sign: {items:?}"
    );
    let now = day_snapshot(u);
    assert_eq!(before.0, now.0, "a refused sign changed the books");
    assert_eq!(before.2, now.2, "a refused sign moved the closed-up-to day");
    if want == ErrorCode::ErrSignWrong {
        assert_eq!(
            now.1,
            before.1 + 1,
            "a wrong sign writes exactly one audit row (PRD FR-21.4)"
        );
        let (action, text) = last_audit(u);
        assert_eq!(action, wrong_action);
        assert!(
            !text.contains(typed.trim()),
            "the audit row of a wrong sign holds the typed text: {text}"
        );
        // An audited refusal wrote a row: the step counts as done.
        Ok(true)
    } else {
        assert_eq!(now.1, before.1, "a blank sign wrote an audit row");
        Err(ServiceError::Rejected(items))
    }
}

/// The day of a level of the run, `YYYY-MM-DD`.
fn iso_of_level(level: i8) -> &'static str {
    ["2026-10-09", "2026-10-10", "2026-10-11", "2026-10-12"]
        [usize::try_from(level + 1).unwrap_or(0)]
}

/// Closes the days up to the 10th or the 11th (PRD FR-21.1). A day that is closed is refused; one
/// that is open, with books that balance, must close.
fn day_close(u: &mut U, d: u8) -> R {
    let level = i8::try_from(d % 2).unwrap_or(0);
    let input = DayCloseInput {
        up_to: day(d % 2).into(),
    };
    let result = days::close(&u.w.env.ctx, &u.w.owner, &input);
    if u.closed_level >= level {
        match result {
            Ok(_) => panic!("day_close: a day that is closed was closed again (PRD FR-21.1)"),
            Err(e) => return Err(e),
        }
    }
    result.unwrap_or_else(|e| {
        panic!("day_close: closing an open day of balanced books was refused: {e:?} (PRD FR-21.1)")
    });
    u.closed_level = level;
    Ok(true)
}

/// A plain reopen with a sign (PRD FR-21.2, plan 80 Q-R2-2).
fn day_reopen(u: &mut U, d: u8, kind: u8) -> R {
    let level = i8::try_from(d % 2).unwrap_or(0);
    let (typed, right) = sign_text(kind);
    let before = day_snapshot(u);
    let result = days::reopen(
        &u.w.env.ctx,
        &u.w.owner,
        &DayReopenInput {
            date: day(d % 2).into(),
            reason: "fix".into(),
            sign: typed.into(),
        },
    );
    match result {
        Ok(status) => {
            assert!(
                right,
                "day_reopen: a {typed:?} sign opened the day (PRD FR-21.2, plan 80 Q-R2-2)"
            );
            assert!(
                u.closed_level >= level,
                "day_reopen: a day that is not closed was reopened"
            );
            assert_eq!(
                checks::level_of(status.closed_up_to.as_deref()),
                level - 1,
                "day_reopen: the closed-up-to day is not the day before the one reopened"
            );
            u.closed_level = level - 1;
            // The audit row says the right sign was given, never the sign itself (FR-21.4 v1.58).
            let (action, text) = last_audit(u);
            assert_eq!(action, "day_reopen");
            assert!(
                text.contains("\"sign_given\":true"),
                "the audit row does not say the sign was given: {text}"
            );
            assert!(
                !text.contains("adnan") && !text.contains("عدنان") && !text.contains("ADNAN"),
                "the audit row holds the sign: {text}"
            );
            Ok(true)
        }
        Err(ServiceError::Rejected(items)) if !right => {
            refused_sign(u, kind, items, &before, "day_reopen_sign_wrong")
        }
        Err(ServiceError::Rejected(items)) => {
            // The right sign: the only refusal is a day that is not closed.
            assert!(
                u.closed_level < level,
                "day_reopen: a closed day was not reopened with the Owner's sign: {items:?}"
            );
            assert!(
                items.len() == 1 && items[0].code == ErrorCode::ErrDayNotClosed,
                "day_reopen: a day that is not closed must give ERR_DAY_NOT_CLOSED: {items:?}"
            );
            assert_eq!(
                before,
                day_snapshot(u),
                "a refused reopen changed something"
            );
            Err(ServiceError::Rejected(items))
        }
        Err(other) => Err(other),
    }
}

/// Reopen to fix (PRD FR-21.4): reopen with a sign, edit a بکری inside the fix, close the same
/// days again. The days must close again, the sign must be on the edit's audit row exactly when the
/// edited posting is on a day the fix opened (PRD FR-7.71), and nothing else of the closed days
/// moves (checked by the run after the step).
fn fix_cycle(u: &mut U, d: u8, p: u8, change: u8, value: u16, kind: u8) -> R {
    let level = i8::try_from(d % 2).unwrap_or(0);
    let (typed, right) = sign_text(kind);
    let before = day_snapshot(u);
    let opened = days::fix_open(
        &u.w.env.ctx,
        &u.w.owner,
        &DayFixOpenInput {
            date: day(d % 2).into(),
            reason: "fix".into(),
            sign: typed.into(),
        },
    );
    let opened = match opened {
        Ok(opened) => opened,
        Err(ServiceError::Rejected(items)) if !right => {
            return refused_sign(u, kind, items, &before, "day_fix_sign_wrong");
        }
        Err(ServiceError::Rejected(items)) => {
            assert!(
                u.closed_level < level,
                "day_reopen_and_fix: a closed day was not reopened with the Owner's sign: {items:?}"
            );
            assert!(
                items.len() == 1 && items[0].code == ErrorCode::ErrDayNotClosed,
                "day_reopen_and_fix: a day that is not closed must give ERR_DAY_NOT_CLOSED: {items:?}"
            );
            assert_eq!(before, day_snapshot(u), "a refused fix changed something");
            return Err(ServiceError::Rejected(items));
        }
        Err(other) => return Err(other),
    };
    assert!(
        right && u.closed_level >= level,
        "day_reopen_and_fix: opened with a {typed:?} sign or a day that is not closed"
    );
    let closed_before = u.closed_level;
    assert_eq!(
        opened.days.len(),
        usize::try_from(closed_before - level + 1).unwrap_or(0),
        "day_reopen_and_fix: the days the fix reopened are not the ones from the day to the closed one"
    );
    assert_eq!(
        checks::level_of(opened.closed_up_to.as_deref()),
        level - 1,
        "day_reopen_and_fix: the closed-up-to day is not the day before the one reopened"
    );
    u.closed_level = level - 1;
    // A بکری edit inside the fix. Whatever it says, the days are closed again after it.
    u.last_edit_day = None;
    let audits = checks::audit_rows(u);
    match edit_sale(u, p, change, value) {
        Ok(_) => {}
        Err(ServiceError::Rejected(items)) => legal_refusal("day_reopen_and_fix", u, &items),
        Err(other) => panic!("day_reopen_and_fix: the edit failed: {other:?}"),
    }
    if let Some(edited) = u.last_edit_day.take() {
        // The edit wrote its audit row in its own save. The sign is on it iff the posting is on a day
        // the fix reopened (PRD FR-7.71); an edit of a day that was never closed has no sign.
        assert!(checks::audit_rows(u) > audits);
        let (action, text) = raw(&u.w.dir)
            .query_row(
                "SELECT action, COALESCE(new_json, '') FROM audit_log
                 WHERE action = 'edit' AND entity = 'bikri' ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .unwrap();
        assert_eq!(action, "edit");
        let reopened = edited.as_str() >= iso_of_level(level)
            && edited.as_str() <= iso_of_level(closed_before);
        assert_eq!(
            text.contains("\"sign_given\":true"),
            reopened,
            "the edit of {edited} says the sign was given iff its day was reopened by the fix: {text}"
        );
        assert!(
            !text.contains("adnan"),
            "the audit row holds the sign: {text}"
        );
    }
    let closed = days::fix_close(
        &u.w.env.ctx,
        &u.w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap_or_else(|e| panic!("day_reopen_and_fix: closing the days again was refused: {e:?}"));
    assert!(
        closed.stayed_open.is_empty(),
        "day_reopen_and_fix: days stayed open after the fix: {:?}",
        closed.stayed_open
    );
    assert_eq!(
        checks::level_of(closed.closed_up_to.as_deref()),
        closed_before,
        "day_reopen_and_fix: the days are not closed up to where they were"
    );
    u.closed_level = closed_before;
    Ok(true)
}

/// A garden's rent, installments and refunds in force, in paisa, from plain SQL.
fn garden_sums(u: &U, garden: i64) -> (i64, i64, i64) {
    let conn = raw(&u.w.dir);
    let sum = |kind: &str| -> i64 {
        conn.query_row(
            &format!(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entries
                 WHERE venture_id = {garden} AND kind = '{kind}' AND status = 'saved'"
            ),
            [],
            |r| r.get(0),
        )
        .unwrap()
    };
    let rent: i64 = conn
        .query_row(
            &format!("SELECT rent_paisa FROM ventures WHERE id = {garden}"),
            [],
            |r| r.get(0),
        )
        .unwrap();
    (rent, sum("installment"), sum("refund"))
}

/// What the owner could still owe: the installments over the rent, never below 0 (PRD FR-12.51).
fn could_owe(installments: i64, rent: i64) -> i64 {
    (installments - rent).max(0)
}

/// Closes the garden (PRD FR-12.38). While the owner still owes money back (PRD FR-12.51) the
/// closing without the answer of the box is refused with ERR_OWNER_OWES_NOT_CONFIRMED and the amount,
/// and with the answer it closes (plan 80 Q-R4-2). Without a debt it closes at once.
fn garden_close(u: &mut U, d: u8) -> R {
    let (rent, installments, refunds) = garden_sums(u, u.garden);
    let owes = (could_owe(installments, rent) - refunds).max(0);
    let input = GardenCloseInput {
        id: u.garden,
        version: version(u, "ventures", u.garden),
        date: day(d).into(),
    };
    let before = (checks::fingerprint(u), checks::audit_rows(u));
    match gardens::close(&u.w.env.ctx, &u.w.owner, &input) {
        Ok(_) => {
            assert_eq!(
                owes, 0,
                "garden_close: closed with no answer to the owner-owes box while the owner owes {owes} paisa (PRD FR-12.38, plan 80 Q-R4-2)"
            );
            Ok(true)
        }
        Err(ServiceError::Rejected(items))
            if items
                .iter()
                .any(|i| i.code == ErrorCode::ErrOwnerOwesNotConfirmed) =>
        {
            assert!(
                owes > 0,
                "garden_close: asked the owner-owes question while the owner owes nothing: {items:?}"
            );
            let asked = items
                .iter()
                .find(|i| i.code == ErrorCode::ErrOwnerOwesNotConfirmed)
                .expect("found above");
            assert_eq!(asked.field.as_deref(), Some("id"));
            assert_eq!(
                asked.values.get("owes_paisa"),
                Some(&ErrorValue::Number(owes)),
                "garden_close: the box names an amount that is not what the owner owes"
            );
            assert_eq!(
                before,
                (checks::fingerprint(u), checks::audit_rows(u)),
                "garden_close: the refused closing wrote something (NFR-REL-1)"
            );
            // With the answer "yes" the same closing goes through.
            gardens::close_with_confirm(&u.w.env.ctx, &u.w.owner, &input, true).unwrap_or_else(
                |e| panic!("garden_close: the closing was refused after the owner said yes: {e:?}"),
            );
            Ok(true)
        }
        Err(other) => Err(other),
    }
}

/// Cancels an entry of the garden. A cancelled installment may leave the owner's refunds above what
/// he could owe: then it is refused with ERR_RETURN_EXCEEDS_OPEN, and only then (PRD FR-12.51,
/// plan 80 R4-3).
fn garden_entry_cancel(u: &mut U, id: i64) -> R {
    let (rent, installments, refunds) = garden_sums(u, u.garden);
    let (kind, amount): (String, i64) = raw(&u.w.dir)
        .query_row(
            &format!("SELECT kind, amount_paisa FROM venture_entries WHERE id = {id}"),
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let expected_refused =
        kind == "installment" && refunds > could_owe(installments - amount, rent);
    let result = gardens::entry_cancel(
        &u.w.env.ctx,
        &u.w.owner,
        &GardenEntryCancelInput {
            id,
            version: version(u, "venture_entries", id),
            reason: "cancel".into(),
            confirm_cash_below_zero: true,
        },
    );
    refund_cap_answer("garden_entry_cancel", result, expected_refused)
}

/// The answer of a step that can leave the owner's refunds above what he could owe: it saved only
/// when it could not, and ERR_RETURN_EXCEEDS_OPEN came only when it could (PRD FR-12.51, R4-3).
fn refund_cap_answer<T>(step: &str, result: Result<T, ServiceError>, expected_refused: bool) -> R {
    match result {
        Ok(_) => {
            assert!(
                !expected_refused,
                "{step}: saved, and the owner's refunds are now above what he could owe (PRD FR-12.51, plan 80 R4-3)"
            );
            Ok(true)
        }
        Err(ServiceError::Rejected(items))
            if items
                .iter()
                .any(|i| i.code == ErrorCode::ErrReturnExceedsOpen) =>
        {
            assert!(
                expected_refused,
                "{step}: refused with ERR_RETURN_EXCEEDS_OPEN though the refunds stay covered: {items:?}"
            );
            Err(ServiceError::Rejected(items))
        }
        Err(other) => Err(other),
    }
}

/// The rent of the garden is changed with a reason and the Owner's sign (PRD FR-12.51). The oracle:
/// it is refused with ERR_RETURN_EXCEEDS_OPEN exactly when the owner's refunds would be above what he
/// could owe after the new rent (plan 80 R4-3), and the preview says the same as the save (B5).
fn garden_rent_edit(u: &mut U, new_rent: i64) -> R {
    let (_, installments, refunds) = garden_sums(u, u.garden);
    let expected_refused = refunds > could_owe(installments, new_rent * 100);
    let partners_now = {
        let conn = raw(&u.w.dir);
        let mut s = conn
            .prepare(
                "SELECT partner_id, share_bps, source FROM venture_partners WHERE venture_id = ?1 ORDER BY partner_id",
            )
            .unwrap();
        s.query_map([u.garden], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
    };
    let input = GardenHeaderEditInput {
        id: u.garden,
        version: version(u, "ventures", u.garden),
        garden: GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(u.w.banana),
            item_ids: Vec::new(),
            rent: new_rent.to_string(),
            season_end: None,
            note: None,
            partners: partners_now
                .iter()
                .map(|(partner_id, bps, source)| VenturePartnerInput {
                    partner_id: *partner_id,
                    share: format!("{}.{:02}", bps / 100, bps % 100),
                    source: if source == "pocket" {
                        MoneySource::Pocket
                    } else {
                        MoneySource::Shop
                    },
                })
                .collect(),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
        reason: "relief from the owner".into(),
        sign: Some("adnan".into()),
    };
    let preview = edit::garden_header_preview(&u.w.env.ctx, &input)?;
    if !preview.errors.is_empty() {
        let capped = preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen);
        if capped {
            assert!(
                expected_refused,
                "garden_rent_edit: the preview refused a rent that keeps the refunds covered: {:?}",
                preview.errors
            );
            return Err(ServiceError::from(preview.errors));
        }
        // Another refusal (the garden is closed, the day is closed ...): nothing to judge here. A save
        // that goes through while the refunds would end above what the owner could owe is caught below.
        return Ok(false);
    }
    let result = edit::garden_header_save(&u.w.env.ctx, &u.w.owner, &input);
    refund_cap_answer("garden_rent_edit", result, expected_refused)
}

/// The rent is cut (relief from its owner, PRD FR-12.51). Aimed at the installments paid so far, so
/// that the owner sometimes owes money back, else cut a little.
fn garden_relief(u: &mut U, cut: u16) -> R {
    let (rent, paid, _) = garden_sums(u, u.garden);
    let (rent, paid) = (rent / 100, paid / 100);
    let new_rent = if paid > 1 && i64::from(cut) % 3 != 0 {
        (1 + i64::from(cut) % paid).min(rent)
    } else {
        (rent - i64::from(cut)).max(1)
    };
    garden_rent_edit(u, new_rent)
}

/// The rent is raised, and now and then to the very edge of what the refunds allow: the installments
/// less the refunds is allowed, one rupee more is refused (PRD FR-12.51, plan 80 R4-3).
fn garden_raise(u: &mut U, by: u16) -> R {
    let (rent, installments, refunds) = garden_sums(u, u.garden);
    let new_rent = if refunds > 0 && by.is_multiple_of(2) {
        (installments - refunds) / 100 + i64::from(by % 3)
    } else {
        rent / 100 + 1 + i64::from(by) % 20_000
    };
    garden_rent_edit(u, new_rent.max(1))
}

/// An installment of the garden is edited to another amount. The refunds of the owner must stay
/// covered (PRD FR-12.51, plan 80 R4-3).
fn garden_installment_edit(u: &mut U, p: u8, amount: u16) -> R {
    let all = ids(
        u,
        &format!(
            "SELECT id FROM venture_entries WHERE status = 'saved' AND kind = 'installment'
             AND venture_id = {} ORDER BY id",
            u.garden
        ),
    );
    let Some(id) = pick(&all, p) else {
        return Ok(false);
    };
    let (rent, installments, refunds) = garden_sums(u, u.garden);
    let (old, iso): (i64, String) = raw(&u.w.dir)
        .query_row(
            &format!("SELECT amount_paisa, entry_date FROM venture_entries WHERE id = {id}"),
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let new = i64::from(amount);
    let expected_refused = refunds > could_owe(installments - old + new * 100, rent);
    let result = edit::garden_entry_save(
        &u.w.env.ctx,
        &u.w.owner,
        &GardenEntryEditInput {
            id,
            version: version(u, "venture_entries", id),
            entry: GardenEntryInput::Installment(GardenPaymentInput {
                garden_id: Some(u.garden),
                date: typed(&iso),
                amount: new.to_string(),
                reason: "payment".into(),
                parts: Vec::new(),
                confirm_cash_below_zero: true,
            }),
            reason: "edit".into(),
            confirm_cash_below_zero: true,
        },
    );
    refund_cap_answer("garden_installment_edit", result, expected_refused)
}

/// A second garden (PRD FR-12.29, plan 80 Q-R4-1): its shares can change while it has no entry and
/// no goods arrived; once goods arrived (even when the arrival is cancelled) they cannot, by the
/// service and by the database.
fn arrival_lock(u: &mut U, weights: &[u16; 3]) -> R {
    if u.garden2_done {
        return Ok(false);
    }
    u.garden2_done = true;
    let first: [u32; 3] = weights.map(u32::from);
    let second: [u32; 3] = [first[2], first[0], first[1]];
    let third: [u32; 3] = [first[1], first[2], first[0]];
    let garden = make_second_garden(u, &first);
    let set = |u: &U, w: &[u32; 3]| {
        gardens::set_partners(
            &u.w.env.ctx,
            &u.w.owner,
            &GardenSetPartnersInput {
                id: garden,
                version: version(u, "ventures", garden),
                partners: partner_inputs(u, w),
            },
        )
    };
    // 1. No entry, no goods arrived, no posted line: the shares change.
    set(u, &second).unwrap_or_else(|e| {
        panic!(
            "garden_arrival_locks_shares: a garden with no entry could not change its shares: {e:?}"
        )
    });
    // 2. Goods arrive.
    let arrival = gardens::arrivals::save(
        &u.w.env.ctx,
        &u.w.owner,
        &ArrivalInput {
            garden_id: Some(garden),
            date: "12-10-2026".into(),
            sell_date: None,
            item_id: Some(u.w.banana),
            packing_id: Some(u.w.box_),
            qty: "5".into(),
            weight: None,
            note: None,
        },
    )
    .unwrap_or_else(|e| panic!("garden_arrival_locks_shares: the goods did not arrive: {e:?}"));
    let locked = |u: &U, w: &[u32; 3], why: &str| {
        let before = (checks::fingerprint(u), checks::audit_rows(u));
        match set(u, w) {
            Ok(_) => panic!(
                "garden_arrival_locks_shares: the shares of a garden changed {why} (PRD FR-12.29, plan 80 Q-R4-1)"
            ),
            Err(ServiceError::Rejected(items)) => {
                assert!(
                    items.len() == 1
                        && items[0].code == ErrorCode::ErrVentureSharesLocked
                        && items[0].field.as_deref() == Some("partners"),
                    "garden_arrival_locks_shares: {why}: {items:?}"
                );
            }
            Err(other) => panic!("garden_arrival_locks_shares: {other:?}"),
        }
        assert_eq!(
            before,
            (checks::fingerprint(u), checks::audit_rows(u)),
            "garden_arrival_locks_shares: the refused change wrote something"
        );
    };
    locked(u, &third, "after goods arrived");
    // 3. The same by the database, whoever asks: the three triggers of migration 26.
    let conn = raw(&u.w.dir);
    let blocked = |sql: String| {
        let error = conn
            .execute(&sql, [])
            .expect_err("the database must refuse it");
        assert!(
            error.to_string().contains("ERR_VENTURE_SHARES_LOCKED"),
            "garden_arrival_locks_shares: {sql}: {error}"
        );
    };
    blocked(format!(
        "UPDATE venture_partners SET share_bps = share_bps + 1 WHERE venture_id = {garden}"
    ));
    blocked(format!(
        "DELETE FROM venture_partners WHERE venture_id = {garden}"
    ));
    blocked(format!(
        "INSERT INTO venture_partners (venture_id, partner_id, share_bps, source)
         VALUES ({garden}, {}, 1, 'shop')",
        u.w.partners[0]
    ));
    drop(conn);
    // 4. A cancelled arrival counts too, like a cancelled entry.
    gardens::arrivals::cancel(
        &u.w.env.ctx,
        &u.w.owner,
        &ArrivalCancelInput {
            id: arrival.row.id,
            version: arrival.row.version,
            reason: "wrong".into(),
        },
    )
    .unwrap_or_else(|e| panic!("garden_arrival_locks_shares: the arrival did not cancel: {e:?}"));
    locked(u, &first, "after its goods arrived entry was cancelled");
    Ok(true)
}
