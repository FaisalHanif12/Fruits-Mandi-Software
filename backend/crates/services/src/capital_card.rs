//! The card "where the capital is" on the روکڑ page (PRD FR-16.14 v1.43 and v1.50, plan 87): where the
//! partners' money is now (two blocks, لینا ہے / پاس ہے and دینا ہے, each with its subtotal, then
//! صافی) and whose it is (each partner's کل رقم, ابتدائی سرمایہ, the open advances), with ابتدائی فرق as a
//! warning that names what it is made of.
//!
//! Every figure is read from the posted records in one read snapshot, with checked maths. The one
//! function builds the card for the screen, its print and its PDF (Code Standards B5). The two sides
//! are equal because every entry balances; the card is refused (`ServiceError::Data`, shown as
//! `ERR_INTERNAL`) when they are not, so a wrong total is never shown.
//!
//! Money identity (PRD FR-17.9 v1.43, with ابتدائی فرق): Σ کل رقم + ابتدائی سرمایہ + the open parts of
//! the advances + ابتدائی فرق (جمع less بنام) = the right side (heads 04 to 06, جمع less بنام) = the
//! left side (heads 01 to 03, بنام less جمع) = صافی. With ابتدائی فرق at 0 it is the identity of
//! FR-17.9.
//!
//! Example (after Events 1 to 5): have 58,484 + 21,564; owe 10,160 + 4,400 (three accounts); صافی
//! 65,488 = −1,004 + 746 + 746 + 65,000.

use std::collections::{BTreeMap, HashMap};

use mandi_db::accounts as account_repo;
use mandi_db::cashbook as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{NAQDI_MANDI_CODE, OPENING_DIFFERENCE_CODE, RecordRef};
use mandi_domain::core::cashbook::{
    CapitalDifference, CapitalDifferenceGroup, CapitalDifferenceKind, CapitalDifferencePart,
    CapitalDifferenceSide, CapitalLink, CapitalSide, CapitalWhere, CapitalWhereInput,
    CapitalWhereKind, CapitalWhereLine, CapitalWhoseKind, CapitalWhoseLine,
};
use mandi_domain::core::codes::{CHUKTA_KHATA_CODE, OLD_CLOSED_ACCOUNT_CODE};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::{Paisa, hundredths};
use mandi_domain::core::validate::{self, FieldErrors};

use crate::cash::CASH_ACCOUNT_CODE;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

/// The firm's opening capital account (PRD FR-10.4).
const OPENING_CAPITAL_CODE: &str = "06010001";
/// The control of a partner's capital account.
const PARTNER_CAPITAL_CONTROL: &str = mandi_domain::core::partners::CAPITAL_CONTROL;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn net_of(dr: i64, cr: i64) -> Result<Paisa, ServiceError> {
    paisa(dr)?.checked_sub(paisa(cr)?).map_err(money_error)
}

fn add(sum: &mut Paisa, part: Paisa) -> Result<(), ServiceError> {
    *sum = sum.checked_add(part).map_err(money_error)?;
    Ok(())
}

fn magnitude(value: Paisa) -> Result<Paisa, ServiceError> {
    if value.is_negative() {
        value.checked_neg().map_err(money_error)
    } else {
        Ok(value)
    }
}

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn record_of(row: &mandi_domain::core::accounts::AccountRow) -> RecordRef {
    RecordRef {
        id: row.id,
        code: row.code.clone(),
        name_ur: row.name_ur.clone(),
        name_en: row.name_en.clone(),
        tag: row.tag.clone(),
    }
}

/// The day the card is read at: empty is today; a day after today is refused.
fn read_as_of(
    input: &CapitalWhereInput,
    today: BusinessDate,
) -> Result<BusinessDate, ServiceError> {
    let text = input.as_of.as_deref().map(str::trim).unwrap_or_default();
    if text.is_empty() {
        return Ok(today);
    }
    let mut errors = FieldErrors::default();
    let day = errors.take(validate::date("as_of", text, today));
    errors.finish()?;
    let day = day.ok_or_else(|| ServiceError::reject(ErrorCode::ErrInternal))?;
    if day > today {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateInFuture,
            "as_of",
        ));
    }
    Ok(day)
}

/// One line before it is put in its block: the signed amount (the left side's sign: what the shop has
/// or is owed is plus, what it owes is minus).
struct Pending {
    kind: CapitalWhereKind,
    signed: Paisa,
    link: CapitalLink,
    account: Option<RecordRef>,
}

/// The sums of the left side, by kind.
#[derive(Default)]
struct Left {
    cash: Option<Paisa>,
    buyers_owe: Paisa,
    buyer_credits: Paisa,
    dead: Paisa,
    advances: Paisa,
    beyond: Paisa,
    payable: Paisa,
    lent: Paisa,
    khatas: Paisa,
    gardens: Paisa,
    contra: Paisa,
    charges: Vec<(i64, Paisa)>,
    all: Paisa,
}

/// Reads the card.
///
/// # Errors
/// `ERR_DATE_*` on `as_of`; `ServiceError::Db`; `ServiceError::Data` when the two sides differ;
/// `ERR_OVERFLOW`.
pub fn read(ctx: &Context, input: &CapitalWhereInput) -> Result<CapitalWhere, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let as_of = read_as_of(input, today)?;
    // Today's card reads the kept sums; a past day reads the dated ones.
    let at_text = as_of.to_iso();
    let at: Option<&str> = (as_of != today).then_some(at_text.as_str());
    ctx.db.read_snapshot(|conn| build(conn, at, &at_text))
}

fn build(conn: &Connection, at: Option<&str>, as_of: &str) -> Result<CapitalWhere, ServiceError> {
    let nets = match at {
        None => repo::account_nets(conn)?,
        Some(day) => repo::account_nets_as_of(conn, day)?,
    };
    let open_adv: HashMap<i64, i64> = repo::open_advances_by_seller_at(conn, at)?
        .into_iter()
        .collect();
    let dead = mandi_db::dead::dead_set(conn, at)?;
    let mut left = Left::default();
    let (mut right_all, mut opening_capital, mut difference) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for a in &nets {
        let net = net_of(a.dr_paisa, a.cr_paisa)?;
        match a.head_code.as_str() {
            "01" | "02" | "03" => add(&mut left.all, net)?,
            _ => add(&mut right_all, net)?,
        }
        if a.code == CASH_ACCOUNT_CODE {
            left.cash = Some(net);
        } else if a.code == OPENING_CAPITAL_CODE {
            add(&mut opening_capital, net)?;
        } else if a.code == OPENING_DIFFERENCE_CODE {
            add(&mut difference, net)?;
        } else if a.head_code == "02" {
            if net != Paisa::ZERO {
                left.charges.push((a.id, net));
            }
        } else {
            sort_left(&mut left, a, net, &open_adv, &dead)?;
        }
    }
    let right_total = right_all.checked_neg().map_err(money_error)?;
    // The books balance: every entry has the same debit and credit.
    if left.all != right_total {
        return Err(data("the two sides of the books are not equal"));
    }
    let (have_lines, have_total, owe_lines, owe_total) = left_blocks(conn, &left)?;
    let net_paisa = have_total.checked_sub(owe_total).map_err(money_error)?;
    if net_paisa != left.all {
        return Err(data(
            "the blocks of the card do not add up to the left side",
        ));
    }
    let (whose_lines, whose_lines_total, unrecorded) =
        whose_side(conn, at, opening_capital, difference, right_total)?;
    let diff = difference_of(conn, at, difference, unrecorded)?;
    let whose_total = whose_lines_total
        .checked_add(unrecorded)
        .map_err(money_error)?;
    if whose_total != net_paisa {
        return Err(data("the two sides of the card are not equal"));
    }
    Ok(CapitalWhere {
        as_of: as_of.to_owned(),
        have_lines,
        have_total_paisa: have_total,
        owe_lines,
        owe_total_paisa: owe_total,
        net_paisa: magnitude(net_paisa)?,
        net_side: side_of_net(net_paisa),
        whose_lines,
        whose_lines_total_paisa: magnitude(whose_lines_total)?,
        whose_lines_side: side_of_net(whose_lines_total),
        difference: diff,
        whose_total_paisa: magnitude(whose_total)?,
        sides_equal: true,
    })
}

/// Puts one account of heads 01 to 03 (not cash, not head 02) in its sum.
fn sort_left(
    left: &mut Left,
    a: &repo::AccountNet,
    net: Paisa,
    open_adv: &HashMap<i64, i64>,
    dead: &mandi_db::dead::DeadSet,
) -> Result<(), ServiceError> {
    let in_dead =
        dead.has_account(a.id) && matches!(a.control_code.as_str(), "0304" | "03011" | "0103");
    if in_dead {
        // His whole ledger balance: his advance and payables are netted into it (PRD FR-10.10). So
        // the advances line on the left leaves out a dead seller's advance, while the partners' open
        // advance parts on the right (PRD FR-17.13) still count it: the two lines differ by it and
        // the totals stay equal (test `a_dead_seller_with_an_open_advance`).
        return add(&mut left.dead, net);
    }
    // چکتا کھاتہ, its old closed account (0031, under the seller control) and نقدی منڈی are in no dues
    // list (PRD FR-13.9): the tile and R20 leave them out, so their balances fall in the "other" line.
    if matches!(
        a.code.as_str(),
        CHUKTA_KHATA_CODE | OLD_CLOSED_ACCOUNT_CODE | NAQDI_MANDI_CODE
    ) {
        return Ok(());
    }
    match a.control_code.as_str() {
        "0304" => {
            if net.is_negative() {
                add(
                    &mut left.buyer_credits,
                    net.checked_neg().map_err(money_error)?,
                )?;
            } else {
                add(&mut left.buyers_owe, net)?;
            }
        }
        "0103" => add(&mut left.lent, net)?,
        "0305" => add(&mut left.khatas, net)?,
        "0306" => add(&mut left.gardens, net)?,
        "0307" => add(&mut left.contra, net)?,
        "03011" => {
            // A seller from his ledger net n and his open advance a: advances a, beyond
            // max(0, n − a), payable max(0, a − n), so a + beyond − payable = n (PRD FR-16.14).
            let open = paisa(open_adv.get(&a.id).copied().unwrap_or(0))?;
            add(&mut left.advances, open)?;
            let over = net.checked_sub(open).map_err(money_error)?;
            if over.is_negative() {
                add(&mut left.payable, over.checked_neg().map_err(money_error)?)?;
            } else {
                add(&mut left.beyond, over)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// The two blocks of the left side, in the order of the card, with their totals.
fn left_blocks(
    conn: &Connection,
    left: &Left,
) -> Result<(Vec<CapitalWhereLine>, Paisa, Vec<CapitalWhereLine>, Paisa), ServiceError> {
    let neg = |p: Paisa| p.checked_neg().map_err(money_error);
    let line = |kind, signed, link, account: Option<RecordRef>| Pending {
        kind,
        signed,
        link,
        account,
    };
    let mut pending: Vec<Pending> = vec![
        line(
            CapitalWhereKind::Cash,
            left.cash.unwrap_or(Paisa::ZERO),
            CapitalLink::Cashbook,
            None,
        ),
        line(
            CapitalWhereKind::BuyersOwe,
            left.buyers_owe,
            CapitalLink::BuyerDues,
            None,
        ),
        line(
            CapitalWhereKind::BuyerCredits,
            neg(left.buyer_credits)?,
            CapitalLink::BuyerLedgers,
            None,
        ),
        line(
            CapitalWhereKind::DeadAmount,
            left.dead,
            CapitalLink::DeadAmount,
            None,
        ),
        line(
            CapitalWhereKind::SellerAdvances,
            left.advances,
            CapitalLink::Advances,
            None,
        ),
        line(
            CapitalWhereKind::SellersOweBeyond,
            left.beyond,
            CapitalLink::SellerDues,
            None,
        ),
        line(
            CapitalWhereKind::SellersPayable,
            neg(left.payable)?,
            CapitalLink::SellerDues,
            None,
        ),
        line(
            CapitalWhereKind::MoneyLent,
            left.lent,
            CapitalLink::MoneyLent,
            None,
        ),
    ];
    for (id, net) in &left.charges {
        let account = account_repo::get(conn, *id)?.map(|row| record_of(&row));
        pending.push(line(
            CapitalWhereKind::OwedForCharges,
            *net,
            CapitalLink::AccountLedger,
            account,
        ));
    }
    pending.push(line(
        CapitalWhereKind::SeparateKhatas,
        left.khatas,
        CapitalLink::SeparateKhatas,
        None,
    ));
    pending.push(line(
        CapitalWhereKind::GardensHeld,
        left.gardens,
        CapitalLink::Gardens,
        None,
    ));
    pending.push(line(
        CapitalWhereKind::LinkedInPartnerTotals,
        left.contra,
        CapitalLink::Partners,
        None,
    ));
    let mut listed = Paisa::ZERO;
    for line in &pending {
        add(&mut listed, line.signed)?;
    }
    // What no named line explains (چکتا and نقدی منڈی balances, bank, anything else of heads 01 to 03).
    let other = left.all.checked_sub(listed).map_err(money_error)?;
    pending.push(line(
        CapitalWhereKind::Other,
        other,
        CapitalLink::TrialBalance,
        None,
    ));
    let (mut have, mut owe) = (Vec::new(), Vec::new());
    let (mut have_total, mut owe_total) = (Paisa::ZERO, Paisa::ZERO);
    for line in pending {
        // Cash and the dead amount are always on the card, at 0 too; the rest only when not 0.
        let always = matches!(
            line.kind,
            CapitalWhereKind::Cash | CapitalWhereKind::DeadAmount
        );
        if line.signed == Paisa::ZERO && !always {
            continue;
        }
        let side = if line.signed.is_negative() {
            CapitalSide::Owe
        } else {
            CapitalSide::Have
        };
        let amount = magnitude(line.signed)?;
        let built = CapitalWhereLine {
            kind: line.kind,
            side,
            amount_paisa: amount,
            link: line.link,
            account: line.account,
        };
        match side {
            CapitalSide::Have => {
                add(&mut have_total, amount)?;
                have.push(built);
            }
            CapitalSide::Owe => {
                add(&mut owe_total, amount)?;
                owe.push(built);
            }
        }
    }
    Ok((have, have_total, owe, owe_total))
}

/// The right side: the partners' totals, ابتدائی سرمایہ, the open advances and, only if something is
/// left, an `other` line (it shows a bug). Returns the lines, their total and what ابتدائی فرق adds to
/// the right side (جمع is plus).
fn whose_side(
    conn: &Connection,
    at: Option<&str>,
    opening_capital: Paisa,
    difference: Paisa,
    right_total: Paisa,
) -> Result<(Vec<CapitalWhoseLine>, Paisa, Paisa), ServiceError> {
    let mut lines: Vec<CapitalWhoseLine> = Vec::new();
    let mut listed = Paisa::ZERO;
    // The advances to sellers are read and replayed once: for every partner's total and for the
    // advances still open (plan 80 gate R1-4).
    let advances = super::partners::money::read_advances(conn)?;
    for partner in mandi_db::partners::list(conn)? {
        let total = super::partners::money::total_with(conn, &partner, at, &advances)?;
        add(&mut listed, total)?;
        lines.push(CapitalWhoseLine {
            kind: CapitalWhoseKind::PartnerTotal,
            partner: Some(RecordRef {
                id: partner.id,
                code: String::new(),
                name_ur: partner.name_ur,
                name_en: partner.name_en,
                tag: None,
            }),
            account: None,
            amount_paisa: total,
            share_bps: None,
            link: CapitalLink::PartnerStatement,
        });
    }
    let opening = opening_capital.checked_neg().map_err(money_error)?;
    add(&mut listed, opening)?;
    let opening_account = match account_repo::id_by_code(conn, OPENING_CAPITAL_CODE)? {
        Some(id) => account_repo::get(conn, id)?.map(|row| record_of(&row)),
        None => None,
    };
    lines.push(CapitalWhoseLine {
        kind: CapitalWhoseKind::OpeningCapital,
        partner: None,
        account: opening_account,
        amount_paisa: opening,
        share_bps: None,
        link: CapitalLink::AccountLedger,
    });
    let open_parts = advances.open_parts_total(at).map_err(money_error)?;
    add(&mut listed, open_parts)?;
    lines.push(CapitalWhoseLine {
        kind: CapitalWhoseKind::AdvancesOpen,
        partner: None,
        account: None,
        amount_paisa: open_parts,
        share_bps: None,
        link: CapitalLink::Advances,
    });
    // ابتدائی فرق on the right side: جمع is plus, بنام is minus.
    let unrecorded = difference.checked_neg().map_err(money_error)?;
    let rest = right_total
        .checked_sub(listed)
        .and_then(|v| v.checked_sub(unrecorded))
        .map_err(money_error)?;
    if rest != Paisa::ZERO {
        add(&mut listed, rest)?;
        lines.push(CapitalWhoseLine {
            kind: CapitalWhoseKind::Other,
            partner: None,
            account: None,
            amount_paisa: rest,
            share_bps: None,
            link: CapitalLink::TrialBalance,
        });
    }
    // Each line's share of the partners' money on the card (the total of the lines), by the shared
    // "% of all" formula: only for a line above 0 over a total above 0 (a bar and a label, never
    // posted). It can be over 100% only if another line is below 0.
    for line in &mut lines {
        line.share_bps = share_of(line.amount_paisa, listed)?;
    }
    Ok((lines, listed, unrecorded))
}

/// `amount` as a share of `total` in basis points (hundredths of a percent, `hundredths`), or `None`
/// when either is not above 0.
fn share_of(amount: Paisa, total: Paisa) -> Result<Option<i32>, ServiceError> {
    if amount.is_negative() || amount == Paisa::ZERO || total.is_negative() || total == Paisa::ZERO
    {
        return Ok(None);
    }
    let bps = hundredths(i128::from(amount.as_paisa()), i128::from(total.as_paisa()))
        .map_err(money_error)?;
    i32::try_from(bps)
        .map(Some)
        .map_err(|_| data("a share of the card does not fit"))
}

/// `have` for 0 or more, `owe` below 0.
fn side_of_net(signed: Paisa) -> CapitalSide {
    if signed.is_negative() {
        CapitalSide::Owe
    } else {
        CapitalSide::Have
    }
}

fn side_of(signed: Paisa) -> CapitalDifferenceSide {
    // Dr is بنام: the account stands below 0 on the right side.
    if signed.is_negative() {
        CapitalDifferenceSide::Dr
    } else {
        CapitalDifferenceSide::Cr
    }
}

/// The warning of ابتدائی فرق: what it is made of, from the lines on that account. The parts add up to
/// the account's net, or the card is refused.
fn difference_of(
    conn: &Connection,
    at: Option<&str>,
    difference_net: Paisa,
    unrecorded: Paisa,
) -> Result<Option<CapitalDifference>, ServiceError> {
    if unrecorded == Paisa::ZERO {
        return Ok(None);
    }
    let id = account_repo::id_by_code(conn, OPENING_DIFFERENCE_CODE)?
        .ok_or_else(|| data("the account ابتدائی فرق is missing"))?;
    // Per kind: the parts and their net on the right side's sign (جمع is plus).
    let mut groups: BTreeMap<u8, (CapitalDifferenceKind, Paisa, Vec<CapitalDifferencePart>)> =
        BTreeMap::new();
    let mut placeable: Vec<RecordRef> = Vec::new();
    let mut parts_total = Paisa::ZERO;
    for line in repo::difference_lines(conn, id, at)? {
        let signed = paisa(line.cr_paisa)?
            .checked_sub(paisa(line.dr_paisa)?)
            .map_err(money_error)?;
        if signed == Paisa::ZERO {
            continue;
        }
        add(&mut parts_total, signed)?;
        let is_opening = line.posting_type == "opening";
        let kind = match &line.account {
            Some(a) if is_opening && a.control_code == PARTNER_CAPITAL_CONTROL => {
                CapitalDifferenceKind::PartnerCapital
            }
            Some(_) if is_opening => CapitalDifferenceKind::AccountOpening,
            _ => CapitalDifferenceKind::OtherEntry,
        };
        let side = side_of(signed);
        let partner = line.account.as_ref().and_then(|a| {
            a.partner.as_ref().map(|(pid, ur, en)| RecordRef {
                id: *pid,
                code: String::new(),
                name_ur: ur.clone(),
                name_en: en.clone(),
                tag: None,
            })
        });
        // The fix can move a partner's opening capital that is still in force and stands against
        // ابتدائی فرق on the debit side (his capital account has the credit).
        let can_place = kind == CapitalDifferenceKind::PartnerCapital
            && side == CapitalDifferenceSide::Dr
            && line
                .account
                .as_ref()
                .is_some_and(|a| a.opening_entry_id == Some(line.entry_id));
        if can_place
            && let Some(p) = &partner
            && !placeable.iter().any(|q| q.id == p.id)
        {
            placeable.push(p.clone());
        }
        let account = line.account.as_ref().map(|a| RecordRef {
            id: a.id,
            code: a.code.clone(),
            name_ur: a.name_ur.clone(),
            name_en: a.name_en.clone(),
            tag: a.tag.clone(),
        });
        let part = CapitalDifferencePart {
            link: if account.is_some() {
                CapitalLink::AccountLedger
            } else {
                CapitalLink::TrialBalance
            },
            account,
            partner,
            date: line.entry_date.clone(),
            amount_paisa: magnitude(signed)?,
            side,
            can_place,
        };
        let key = match kind {
            CapitalDifferenceKind::PartnerCapital => 0,
            CapitalDifferenceKind::AccountOpening => 1,
            CapitalDifferenceKind::OtherEntry => 2,
        };
        let entry = groups
            .entry(key)
            .or_insert_with(|| (kind, Paisa::ZERO, Vec::new()));
        add(&mut entry.1, signed)?;
        entry.2.push(part);
    }
    // The parts must add up to the account: a gap is a bug, never shown (the card is refused).
    let expected = difference_net.checked_neg().map_err(money_error)?;
    if parts_total != expected || expected != unrecorded {
        return Err(data(
            "the parts of ابتدائی فرق do not add up to the account",
        ));
    }
    let mut out = Vec::new();
    for (_, (kind, net, parts)) in groups {
        if net == Paisa::ZERO {
            continue;
        }
        out.push(CapitalDifferenceGroup {
            kind,
            amount_paisa: magnitude(net)?,
            side: side_of(net),
            parts,
        });
    }
    Ok(Some(CapitalDifference {
        amount_paisa: magnitude(unrecorded)?,
        side: side_of(unrecorded),
        groups: out,
        placeable_partners: placeable,
    }))
}
