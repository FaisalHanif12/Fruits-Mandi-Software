//! One lot (PRD 12.3, 12.5, money agent 6.2 and 6.3): its charges, and the totals of C6 to C9
//! for a lot or for the whole بکری.

use mandi_core::bikri::{
    ChargeSource, LotCharge, LotChargeMethod, RuleLabel, SummaryTotals, SummaryWorkings,
    TypedLotCharge,
};
use mandi_core::bikri::{LineCalc, PaymentType};
use mandi_core::money::{MoneyError, Paisa, Qty};
use mandi_core::rules::{ChargeSide, ChargeType, ChargeValue, RuleLevel, Working};

use super::lookups::ItemFacts;
use super::stored::StoredLotCharge;
use super::working;
use super::{CalcError, Env, PostingKind};
use crate::rules::{Query, Winner, charge_amount, resolve};

/// What a lot charge is looked up for.
pub(super) struct LotKey {
    pub seller_id: i64,
    pub item_id: i64,
    pub item: ItemFacts,
    pub qty_received: Qty,
}

/// A lot charge that is not charged in this posting (a later sale charges none, PRD FR-12.17).
fn not_charged() -> LotCharge {
    LotCharge {
        method: LotChargeMethod::Manual,
        value_paisa: Paisa::ZERO,
        amount: Paisa::ZERO,
        rule_id: None,
        rule_label: None,
        source: ChargeSource::None,
        working: Working {
            message_id: "working-per-lot-already-charged".to_owned(),
            args: Vec::new(),
            rule_id: None,
        },
    }
}

/// A lot charge the user did not change, as the lot stored it (PRD FR-12.20, FR-11.6): its method,
/// value and rule stay. A per-نگ charge is worked out again from its stored value and the
/// quantity received now (C3), which only differs from the stored amount when that quantity was
/// corrected; a per-lot or typed amount stays as it was.
fn from_stored(
    key: &LotKey,
    kind: ChargeType,
    stored: &StoredLotCharge,
) -> Result<LotCharge, CalcError> {
    let typed_by_hand = stored.rule_id.is_none();
    let value = match stored.method {
        LotChargeMethod::Manual => {
            // Nothing applied, or an amount typed by hand: the amount stays.
            return Ok(LotCharge {
                method: LotChargeMethod::Manual,
                value_paisa: stored.amount,
                amount: stored.amount,
                rule_id: None,
                rule_label: None,
                source: if stored.amount > Paisa::ZERO {
                    ChargeSource::Typed
                } else {
                    ChargeSource::None
                },
                working: working::typed(stored.amount),
            });
        }
        LotChargeMethod::PerUnit => ChargeValue::PerUnit(stored.value),
        LotChargeMethod::PerLot => ChargeValue::PerLot(stored.value),
    };
    let level = if typed_by_hand {
        RuleLevel::LineOverride
    } else {
        stored.level.unwrap_or(RuleLevel::Default)
    };
    let winner = Winner {
        level,
        rule_id: stored.rule_id,
        value,
        manual: typed_by_hand,
    };
    let charged = charge_amount(Some(&winner), kind, Paisa::ZERO, key.qty_received, true)?;
    Ok(LotCharge {
        method: stored.method,
        value_paisa: stored.value,
        amount: charged.amount,
        rule_id: stored.rule_id,
        rule_label: (!typed_by_hand).then_some(RuleLabel {
            level,
            rule_id: stored.rule_id,
        }),
        source: if typed_by_hand {
            ChargeSource::Typed
        } else if level == RuleLevel::Default {
            ChargeSource::Default
        } else {
            ChargeSource::Rule
        },
        working: charged.working,
    })
}

/// The charge of a lot for مزدوری, منشیانہ or فون (PRD 11.2): the value typed on the lot, else
/// the rule that wins for the lot's seller and item, else nothing. In an edit, a charge whose
/// typed value equals what was stored (or both are empty) is the stored charge (PRD FR-12.20). A typed lot charge is always
/// allowed (PRD FR-11.10 limits only commission and لاگا on a line). A per-نگ charge counts the
/// نگ received, not the نگ sold (PRD E2E-16).
///
/// Example (PRD 4.4): 5 per نگ on 50 نگ received is 250.
pub(super) fn lot_charge(
    env: &Env<'_>,
    key: &LotKey,
    kind: ChargeType,
    typed: Option<TypedLotCharge>,
    stored: Option<&StoredLotCharge>,
) -> Result<LotCharge, CalcError> {
    if env.kind == PostingKind::Later {
        return Ok(not_charged());
    }
    if let Some(stored) = stored.filter(|s| s.typed() == typed) {
        return from_stored(key, kind, stored);
    }
    if let Some(typed) = typed {
        return Ok(match typed.method {
            LotChargeMethod::Manual => LotCharge {
                method: LotChargeMethod::Manual,
                value_paisa: typed.value,
                amount: typed.value,
                rule_id: None,
                rule_label: None,
                source: ChargeSource::Typed,
                working: working::typed(typed.value),
            },
            method => {
                let value = if method == LotChargeMethod::PerUnit {
                    ChargeValue::PerUnit(typed.value)
                } else {
                    ChargeValue::PerLot(typed.value)
                };
                let winner = Winner {
                    level: RuleLevel::LineOverride,
                    rule_id: None,
                    value,
                    manual: true,
                };
                let charged =
                    charge_amount(Some(&winner), kind, Paisa::ZERO, key.qty_received, true)?;
                LotCharge {
                    method,
                    value_paisa: typed.value,
                    amount: charged.amount,
                    rule_id: None,
                    rule_label: None,
                    source: ChargeSource::Typed,
                    working: charged.working,
                }
            }
        });
    }
    let found = resolve(
        env.rules,
        &Query {
            side: ChargeSide::Seller,
            charge: kind,
            party_id: Some(key.seller_id),
            item_id: key.item_id,
            item_group: Some(key.item.group),
            date: env.date,
            line_override: None,
        },
    );
    let Some(winner) = found.winner else {
        let charged = charge_amount(None, kind, Paisa::ZERO, key.qty_received, true)?;
        return Ok(LotCharge {
            method: LotChargeMethod::Manual,
            value_paisa: Paisa::ZERO,
            amount: Paisa::ZERO,
            rule_id: None,
            rule_label: None,
            source: ChargeSource::None,
            working: charged.working,
        });
    };
    let method = match winner.value {
        ChargeValue::PerUnit(_) => LotChargeMethod::PerUnit,
        ChargeValue::PerLot(_) => LotChargeMethod::PerLot,
        // The rule set never holds a percent lot charge (PRD 11.2).
        ChargeValue::Percent(_) => {
            return Err(CalcError::Inconsistent("a percent rule for a lot charge"));
        }
    };
    let charged = charge_amount(Some(&winner), kind, Paisa::ZERO, key.qty_received, true)?;
    let value = match winner.value {
        ChargeValue::PerUnit(v) | ChargeValue::PerLot(v) => v,
        ChargeValue::Percent(_) => Paisa::ZERO,
    };
    Ok(LotCharge {
        method,
        value_paisa: value,
        amount: charged.amount,
        rule_id: winner.rule_id,
        rule_label: Some(RuleLabel {
            level: winner.level,
            rule_id: winner.rule_id,
        }),
        source: if winner.level == RuleLevel::Default {
            ChargeSource::Default
        } else {
            ChargeSource::Rule
        },
        working: charged.working,
    })
}

/// How the workings of a total are worded.
pub(super) enum Scope<'a> {
    /// One lot: lot charges have their own working; `cash_is_default` for a one-time seller's
    /// نقد (PRD FR-13.10).
    Lot {
        labour: &'a Working,
        munshiana: &'a Working,
        phone: &'a Working,
        cash_is_default: bool,
    },
    /// The whole بکری: every row is a sum of lots.
    Whole,
}

/// A running total over lines (a lot) or lots (the whole بکری). All sums are checked and are of
/// stored amounts: a total is never rounded again (money agent point b).
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Acc {
    /// Lines in a lot, or lots in the whole.
    pub count: i64,
    pub gross: Paisa,
    pub freight: Paisa,
    pub seller_comm: Paisa,
    pub labour: Paisa,
    pub munshiana: Paisa,
    pub cash_paid: Paisa,
    pub store_rent: Paisa,
    pub phone: Paisa,
    pub buyer_comm: Paisa,
    pub laga: Paisa,
    pub rate_diff: Paisa,
    pub buyer_credit: Paisa,
    pub buyer_cash: Paisa,
    pub buyer_total: Paisa,
}

impl Acc {
    /// Adds one complete line.
    pub(super) fn add_line(&mut self, line: &LineCalc) -> Result<(), MoneyError> {
        self.count = self.count.checked_add(1).ok_or(MoneyError::Overflow)?;
        self.gross = self.gross.checked_add(line.seller_amount)?;
        self.seller_comm = self.seller_comm.checked_add(line.seller_comm.amount)?;
        self.buyer_comm = self.buyer_comm.checked_add(line.buyer_comm.amount)?;
        self.laga = self.laga.checked_add(line.laga.amount)?;
        self.rate_diff = self.rate_diff.checked_add(line.rate_diff)?;
        match line.payment_type {
            PaymentType::Credit => {
                self.buyer_credit = self.buyer_credit.checked_add(line.buyer_total)?;
            }
            PaymentType::Cash => {
                self.buyer_cash = self.buyer_cash.checked_add(line.buyer_total)?;
            }
        }
        self.buyer_total = self.buyer_total.checked_add(line.buyer_total)?;
        Ok(())
    }

    /// Adds a finished lot to the total of the whole بکری.
    pub(super) fn merge(&mut self, lot: &Acc) -> Result<(), MoneyError> {
        self.count = self.count.checked_add(1).ok_or(MoneyError::Overflow)?;
        self.gross = self.gross.checked_add(lot.gross)?;
        self.freight = self.freight.checked_add(lot.freight)?;
        self.seller_comm = self.seller_comm.checked_add(lot.seller_comm)?;
        self.labour = self.labour.checked_add(lot.labour)?;
        self.munshiana = self.munshiana.checked_add(lot.munshiana)?;
        self.cash_paid = self.cash_paid.checked_add(lot.cash_paid)?;
        self.store_rent = self.store_rent.checked_add(lot.store_rent)?;
        self.phone = self.phone.checked_add(lot.phone)?;
        self.buyer_comm = self.buyer_comm.checked_add(lot.buyer_comm)?;
        self.laga = self.laga.checked_add(lot.laga)?;
        self.rate_diff = self.rate_diff.checked_add(lot.rate_diff)?;
        self.buyer_credit = self.buyer_credit.checked_add(lot.buyer_credit)?;
        self.buyer_cash = self.buyer_cash.checked_add(lot.buyer_cash)?;
        self.buyer_total = self.buyer_total.checked_add(lot.buyer_total)?;
        Ok(())
    }

    /// The lot charges that come off the seller's credit (everything of C6 except the seller
    /// commission and نقد): freight paid by the firm, مزدوری, منشیانہ, کرایہ سٹور, فون.
    fn charges(&self) -> Result<Paisa, MoneyError> {
        self.freight
            .checked_add(self.labour)?
            .checked_add(self.munshiana)?
            .checked_add(self.store_rent)?
            .checked_add(self.phone)
    }

    /// C8: `gross − commission − lot charges` (نقد is not in it; it posts in the cash part).
    pub(super) fn seller_credit(&self) -> Result<Paisa, MoneyError> {
        self.gross
            .checked_sub(self.seller_comm)?
            .checked_sub(self.charges()?)
    }

    /// C6: `freight + commission + labour + munshiana + نقد + store rent + phone`.
    pub(super) fn deductions(&self) -> Result<Paisa, MoneyError> {
        self.charges()?
            .checked_add(self.seller_comm)?
            .checked_add(self.cash_paid)
    }

    /// C7: `gross − deductions`. Can be negative.
    pub(super) fn net(&self) -> Result<Paisa, MoneyError> {
        self.gross.checked_sub(self.deductions()?)
    }

    /// C9: `seller commission + buyer commission + لاگا + فون + rate difference`.
    pub(super) fn shared_income(&self) -> Result<Paisa, MoneyError> {
        self.seller_comm
            .checked_add(self.buyer_comm)?
            .checked_add(self.laga)?
            .checked_add(self.phone)?
            .checked_add(self.rate_diff)
    }

    /// The lines of PRD 12.5 with the working of each.
    pub(super) fn totals(&self, scope: &Scope<'_>) -> Result<SummaryTotals, MoneyError> {
        let (deductions, net, credit, shared) = (
            self.deductions()?,
            self.net()?,
            self.seller_credit()?,
            self.shared_income()?,
        );
        let sum = |amount: Paisa| working::sum(self.count, amount);
        let typed = |amount: Paisa| match scope {
            Scope::Lot { .. } => working::typed(amount),
            Scope::Whole => working::sum(self.count, amount),
        };
        let (labour, munshiana, phone) = match scope {
            Scope::Lot {
                labour,
                munshiana,
                phone,
                ..
            } => ((*labour).clone(), (*munshiana).clone(), (*phone).clone()),
            Scope::Whole => (
                working::sum(self.count, self.labour),
                working::sum(self.count, self.munshiana),
                working::sum(self.count, self.phone),
            ),
        };
        let cash = match scope {
            Scope::Lot {
                cash_is_default: true,
                ..
            } => working::pukhta_cash(self.cash_paid),
            _ => typed(self.cash_paid),
        };
        Ok(SummaryTotals {
            gross: self.gross,
            freight: self.freight,
            seller_comm: self.seller_comm,
            labour: self.labour,
            munshiana: self.munshiana,
            cash_paid: self.cash_paid,
            store_rent: self.store_rent,
            phone: self.phone,
            deductions,
            net,
            seller_credit: credit,
            buyer_comm: self.buyer_comm,
            laga: self.laga,
            rate_diff: self.rate_diff,
            buyer_credit: self.buyer_credit,
            buyer_cash: self.buyer_cash,
            buyer_total: self.buyer_total,
            shared_income: shared,
            workings: SummaryWorkings {
                gross: sum(self.gross),
                freight: typed(self.freight),
                seller_comm: sum(self.seller_comm),
                labour,
                munshiana,
                cash_paid: cash,
                store_rent: typed(self.store_rent),
                phone,
                deductions: working::deductions(
                    self.freight,
                    self.seller_comm,
                    self.labour,
                    self.munshiana,
                    self.cash_paid,
                    self.store_rent,
                    self.phone,
                    deductions,
                ),
                net: working::net(self.gross, deductions, net),
                seller_credit: working::seller_credit(
                    self.gross,
                    self.seller_comm,
                    self.freight,
                    self.labour,
                    self.munshiana,
                    self.store_rent,
                    self.phone,
                    credit,
                ),
                buyer_comm: sum(self.buyer_comm),
                laga: sum(self.laga),
                rate_diff: sum(self.rate_diff),
                buyer_credit: sum(self.buyer_credit),
                buyer_cash: sum(self.buyer_cash),
                buyer_total: sum(self.buyer_total),
                shared_income: working::shared_income(
                    self.seller_comm,
                    self.buyer_comm,
                    self.laga,
                    self.phone,
                    self.rate_diff,
                    shared,
                ),
            },
        })
    }
}
