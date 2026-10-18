//! Random sales, later sales, edits, cancels, write-offs and receipts through the real services
//! (Code Standards 9.3, plan 16). After EVERY step the books still hold:
//!
//! - the trial balance difference is 0 and the chains verify at the end;
//! - an entry and its reversal are zero on every account;
//! - the stored matching rows are the canonical ones: working them out again changes nothing;
//! - no buyer has an open bill next to an unmatched credit;
//! - the partner share rows add up to the income accounts;
//! - the lot columns equal the sums of their lines in force, and the stock moves equal the lot
//!   balance (zero for a cancelled بکری);
//! - the analytics facts equal the lines in force;
//! - every stored amount is a whole rupee.
//!
//! The steps come from a seeded random source, so a failure can be repeated from its seed.

use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDraft, LaterSaleSaveInput, LineChargeMethod, PaymentType, StockListInput, WriteOffInput,
};
use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput, RuleSaved};
use mandi_services::core::vouchers::OpenPayablesInput;
use mandi_services::{ledger, matching, rules, vouchers};

use crate::common::{assert_chain_ok, raw, set_today};
use crate::helpers::*;
use crate::support::*;
use crate::tools;

/// A small deterministic random source.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn index(&mut self, n: usize) -> usize {
        usize::try_from(self.below(u64::try_from(n).unwrap_or(1).max(1))).unwrap_or(0)
    }
}

const SHAHID: &str = "شاہد|03000000000";
const NASIR: &str = "ناصر|03001111111";

fn sql_i64(w: &World, sql: &str) -> i64 {
    scalar(w, sql)
}

fn day_text(day: u64) -> String {
    format!("{day:02}-10-2026")
}

/// A new بکری of one lot with one to three lines.
fn random_save(w: &World, rng: &mut Rng, step: usize) -> BikriDraft {
    let seller = [w.rahim, w.karim][rng.index(2)];
    let item = [w.banana, w.apple, w.mooli][rng.index(3)];
    let total = 6 + rng.below(30);
    let lines = 1 + rng.below(3);
    let mut left = i64::try_from(total).unwrap_or(6);
    let mut sales = Vec::new();
    for j in 0..lines {
        if left == 0 {
            break;
        }
        let qty = (1 + i64::try_from(rng.below(6)).unwrap_or(1)).min(left);
        left -= qty;
        let which = rng.index(4);
        let buyer = [w.akram, w.arshad, w.naqdi, w.pukhta_buyer][which];
        let mut line = sale(
            &format!("s-{step}-{j}"),
            buyer,
            &qty.to_string(),
            &(100 + rng.below(900)).to_string(),
        );
        if which == 3 {
            let (ur, en, phone) = if rng.below(2) == 0 {
                ("شاہد", "Shahid", "0300-0000000")
            } else {
                ("ناصر", "Nasir", "0300-1111111")
            };
            line.pukhta = person(ur, en, phone);
        }
        if which < 2 && rng.below(3) == 0 {
            line.payment_type = Some(PaymentType::Cash);
        }
        sales.push(line);
    }
    let mut l = lot(w, "lot-1", seller, item, &total.to_string());
    l.freight = Some(rng.below(2_000).to_string());
    l.cash_paid = (rng.below(2) == 0).then(|| rng.below(2_000).to_string());
    l.adjust_advance = rng.below(4) != 0;
    l.sales = sales;
    let mut d = draft(vec![l]);
    d.header.entry_date = day_text(10 + rng.below(3));
    d
}

fn random_later(w: &World, rng: &mut Rng, step: usize) -> bool {
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    if stock.rows.is_empty() {
        return false;
    }
    let row = &stock.rows[rng.index(stock.rows.len())];
    let received: u64 = row.received_date[8..10].parse().unwrap_or(10);
    let date = day_text(received + rng.below(13 - received));
    let qty = 1 + rng.below(u64::try_from(row.balance).unwrap_or(1));
    let buyer = [w.akram, w.arshad][rng.index(2)];
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: row.lot_id,
            date,
            sales: vec![sale(
                &format!("l-{step}"),
                buyer,
                &qty.to_string(),
                &(100 + rng.below(900)).to_string(),
            )],
            confirm_cash_below_zero: true,
        },
    )
    .is_ok()
}

fn random_edit(w: &World, rng: &mut Rng) -> bool {
    let ids: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare("SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id")
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    if ids.is_empty() {
        return false;
    }
    let bikri_id = ids[rng.index(ids.len())];
    let detail = bikri::get(&w.env.ctx, bikri_id).unwrap();
    let live: Vec<_> = detail.postings.iter().filter(|p| p.editable).collect();
    if live.is_empty() {
        return false;
    }
    let posting = live[rng.index(live.len())];
    let opened = tools::open(w, bikri_id, posting.entry_id);
    let mut d = opened.draft.clone();
    let li = rng.index(d.lots.len());
    let lot = &mut d.lots[li];
    if lot.sales.is_empty() {
        return false;
    }
    let si = rng.index(lot.sales.len());
    match rng.below(9) {
        0 => lot.sales[si].seller_rate = (100 + rng.below(900)).to_string(),
        5 => {
            // A new line (a lot with a per-lot rule may then charge it on another line).
            let buyer = [w.akram, w.arshad][rng.index(2)];
            let key = format!("n-{}", rng.next());
            lot.sales
                .push(sale(&key, buyer, "1", &(100 + rng.below(900)).to_string()));
        }
        6 => {
            // The quantity received of a lot of the first posting.
            if posting.posting_type == "bikri_first" {
                let qty: i64 = lot.qty_received.parse().unwrap_or(1);
                lot.qty_received = (qty + 1 + i64::try_from(rng.below(3)).unwrap_or(0)).to_string();
            }
        }
        7 => {
            // A typed seller commission on one line: a carrier of a per-lot charge may be typed over.
            lot.sales[si].seller_comm = typed(LineChargeMethod::Percent, "4");
        }
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
            if posting.posting_type == "bikri_first" {
                lot.cash_paid = Some(rng.below(3_000).to_string());
                lot.freight = Some(rng.below(3_000).to_string());
            }
        }
        _ => {
            lot.sales[si].payment_type = Some(if rng.below(2) == 0 {
                PaymentType::Cash
            } else {
                PaymentType::Credit
            });
        }
    }
    let preview = tools::preview(w, &opened, &d);
    if !preview.errors.is_empty() {
        return false;
    }
    let done = tools::try_edit(w, &opened, &d, true).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(done.preview, preview, "the save is not the preview");
    true
}

fn random_cancel(w: &World, rng: &mut Rng) -> bool {
    let ids: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare("SELECT id FROM bikri_entries WHERE status = 'saved' ORDER BY id")
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    if ids.is_empty() {
        return false;
    }
    tools::cancel(w, ids[rng.index(ids.len())], true).is_ok()
}

fn random_write_off(w: &World, rng: &mut Rng) -> bool {
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    if stock.rows.is_empty() {
        return false;
    }
    let row = &stock.rows[rng.index(stock.rows.len())];
    bikri::write_off(
        &w.env.ctx,
        &w.owner,
        &WriteOffInput {
            lot_id: row.lot_id,
            qty: "1".into(),
            reason: "خراب".into(),
        },
    )
    .is_ok()
}

fn keys(w: &World) -> [(i64, Option<&'static str>); 4] {
    [
        (w.akram, None),
        (w.arshad, None),
        (w.pukhta_buyer, Some(SHAHID)),
        (w.pukhta_buyer, Some(NASIR)),
    ]
}

/// What must hold after any step.
fn check_books(w: &World, seed: u64, step: usize) {
    let at = format!("seed {seed:x} step {step}");
    assert_eq!(tools::trial_difference(w), 0, "{at}: the trial balance");
    // An entry and its reversal are zero on every account.
    assert_eq!(
        sql_i64(
            w,
            "SELECT COUNT(*) FROM (
               SELECT l.account_id FROM journal_entries r
               JOIN journal_lines l ON l.entry_id IN (r.id, r.reverses_entry_id)
               WHERE r.reverses_entry_id IS NOT NULL
               GROUP BY r.id, l.account_id, COALESCE(l.pukhta_person_key, '')
               HAVING SUM(l.dr_paisa) - SUM(l.cr_paisa) <> 0)"
        ),
        0,
        "{at}: a reversal is not the mirror of its entry"
    );
    // The stored matching is the canonical one.
    let now = w.env.ctx.now_text();
    for (account, key) in keys(w) {
        let again = w
            .env
            .ctx
            .db()
            .write(|tx| matching::rematch(tx, w.owner.user_id, &now, account, key))
            .unwrap();
        assert_eq!(
            (again.switched_off, again.added),
            (0, 0),
            "{at}: matching of {account} {key:?} is not canonical"
        );
        let (open, unmatched, _) = key_state(w, account, key);
        assert!(open >= 0 && unmatched >= 0, "{at}: negative open or credit");
        assert!(
            open == 0 || unmatched == 0,
            "{at}: an open bill waits next to an unmatched credit for {account} {key:?}"
        );
    }
    for seller in [w.rahim, w.karim] {
        let again = w
            .env
            .ctx
            .db()
            .write(|tx| matching::rematch_advances(tx, w.owner.user_id, &now, seller))
            .unwrap();
        assert_eq!(
            (again.switched_off, again.added),
            (0, 0),
            "{at}: advance matching of {seller} is not canonical"
        );
        let advances = mandi_services::assist::advances(&w.env.ctx, seller).unwrap();
        assert!(advances.open_paisa.as_paisa() >= 0);
    }
    // No credit or bill line is matched for more than it holds.
    assert_eq!(
        sql_i64(
            w,
            "SELECT COUNT(*) FROM (SELECT l.id FROM journal_lines l
               JOIN bill_allocations a ON a.is_active = 1 AND (a.bill_line_id = l.id OR a.payment_line_id = l.id)
               GROUP BY l.id HAVING SUM(a.amount_paisa) > MAX(l.dr_paisa, l.cr_paisa))"
        ),
        0,
        "{at}: a line is matched for more than it holds"
    );
    // Partner shares add up to the income accounts.
    assert_eq!(
        sql_i64(
            w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows"
        ),
        sql_i64(
            w,
            "SELECT COALESCE(SUM(l.cr_paisa) - SUM(l.dr_paisa), 0) FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id WHERE a.code LIKE '04%'"
        ),
        "{at}: the shares are not the income"
    );
    check_each_split(w, &at);
    check_seller_balances(w, &at);
    // Lot columns and stock moves.
    assert_eq!(
        sql_i64(
            w,
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
             WHERE b.status = 'saved' AND (
               l.qty_sold <> (SELECT COALESCE(SUM(qty), 0) FROM bikri_sales WHERE lot_id = l.id AND is_active = 1)
               OR l.gross_paisa <> (SELECT COALESCE(SUM(seller_amount_paisa), 0) FROM bikri_sales WHERE lot_id = l.id AND is_active = 1)
               OR l.seller_comm_paisa <> (SELECT COALESCE(SUM(seller_comm_paisa), 0) FROM bikri_sales WHERE lot_id = l.id AND is_active = 1)
               OR (SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE lot_id = l.id)
                    <> l.qty_received - l.qty_sold - l.qty_written_off)"
        ),
        0,
        "{at}: a lot differs from its lines or its stock"
    );
    assert_eq!(
        sql_i64(
            w,
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
             WHERE b.status = 'cancelled' AND (
               (SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE lot_id = l.id) <> 0
               OR (SELECT COUNT(*) FROM bikri_sales WHERE lot_id = l.id AND is_active = 1) <> 0)"
        ),
        0,
        "{at}: a cancelled lot still holds stock or lines"
    );
    // Analytics facts equal the lines in force.
    assert_eq!(
        sql_i64(
            w,
            "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0) FROM analytics_facts"
        ),
        sql_i64(
            w,
            "SELECT COALESCE(SUM(seller_amount_paisa), 0) FROM bikri_sales WHERE is_active = 1"
        ),
        "{at}: the facts are not the lines"
    );
    assert_eq!(
        sql_i64(
            w,
            "SELECT COALESCE(SUM(sign * qty), 0) FROM analytics_facts"
        ),
        sql_i64(
            w,
            "SELECT COALESCE(SUM(qty), 0) FROM bikri_sales WHERE is_active = 1"
        ),
        "{at}: the fact quantities are not the lines"
    );
    // Every amount is a whole rupee.
    assert_eq!(
        sql_i64(
            w,
            "SELECT (SELECT COUNT(*) FROM journal_lines WHERE dr_paisa % 100 <> 0 OR cr_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM bikri_sales WHERE buyer_total_paisa % 100 <> 0 OR seller_comm_paisa % 100 <> 0)
                  + (SELECT COUNT(*) FROM partner_share_rows WHERE amount_paisa % 100 <> 0)"
        ),
        0,
        "{at}: an amount that is not a whole rupee"
    );
}

/// Every posting's partner rows add up to the income and expense lines of THAT posting, and follow
/// the leftover rule C10: every partner gets his share rounded down in whole rupees, and the
/// leftover goes to the partner with the biggest share (a tie: the one created first).
fn check_each_split(w: &World, at: &str) {
    assert_eq!(
        sql_i64(
            w,
            "SELECT COUNT(*) FROM journal_entries e WHERE e.status = 'posted' AND
               COALESCE((SELECT SUM(amount_paisa) FROM partner_share_rows WHERE journal_entry_id = e.id), 0)
               <> COALESCE((SELECT SUM(l.cr_paisa) - SUM(l.dr_paisa) FROM journal_lines l
                            JOIN accounts a ON a.id = l.account_id
                            WHERE l.entry_id = e.id AND (a.code LIKE '04%' OR a.code LIKE '05%')), 0)"
        ),
        0,
        "{at}: a posting's shares are not its income and expense"
    );
    let rows: Vec<(i64, i64, i64, i64, i64)> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT journal_entry_id, partner_id, share_bps, base_paisa, amount_paisa
                 FROM partner_share_rows ORDER BY journal_entry_id, partner_id",
            )
            .unwrap();
        statement
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let mut start = 0;
    while start < rows.len() {
        let entry = rows[start].0;
        let end = start + rows[start..].iter().take_while(|r| r.0 == entry).count();
        let group = &rows[start..end];
        let base = group[0].3.abs() / 100;
        let floors: Vec<i64> = group.iter().map(|r| base * r.2 / 10_000).collect();
        let leftover = base - floors.iter().sum::<i64>();
        // The partner with the biggest share; a tie goes to the one created first (lowest id).
        let top = group
            .iter()
            .enumerate()
            .max_by_key(|(_, r)| (r.2, -r.1))
            .map(|(i, _)| i)
            .unwrap();
        for (i, row) in group.iter().enumerate() {
            let want = floors[i] + if i == top { leftover } else { 0 };
            assert_eq!(
                row.4.abs() / 100,
                want,
                "{at}: entry {entry}, partner {}: the share is not the C10 share",
                row.1
            );
        }
        start = end;
    }
}

/// A seller's balance is his open payable, less his open advances, less what he owes beyond them
/// (PRD FR-7.59, FR-15.9, R24): the working agrees with the ledger after any step.
fn check_seller_balances(w: &World, at: &str) {
    for seller in [w.rahim, w.karim] {
        let (_, _, dr_less_cr) = key_state(w, seller, None);
        let payables = vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: seller,
                person_key: None,
                include_settled: false,
            },
        )
        .unwrap();
        let credit_balance = payables.open_paisa.as_paisa()
            - payables.advances_open_paisa.as_paisa()
            - payables.owes_firm_paisa.as_paisa();
        assert_eq!(
            credit_balance,
            -dr_less_cr,
            "{at}: seller {seller}: open {} advances {} owes {} but the ledger says {}",
            payables.open_paisa.as_paisa(),
            payables.advances_open_paisa.as_paisa(),
            payables.owes_firm_paisa.as_paisa(),
            -dr_less_cr
        );
    }
}

/// Saves the per-lot rule for کیلا (or changes it, or switches it off), so the random run meets
/// lots that carry a per-lot commission and rules that change between edits (PRD FR-11.9).
fn banana_per_lot_rule(
    w: &World,
    rule: Option<&RuleSaved>,
    value: &str,
    active: bool,
) -> RuleSaved {
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: rule.map(|r| r.row.id),
            version: rule.map(|r| r.row.version),
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: None,
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::PerLot,
            value: value.into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: active,
            note: String::new(),
        },
    )
    .unwrap()
}

/// What the rows and the lines of a key say: the open bills, the unmatched credits, and the
/// balance, in paisa (debit positive).
fn key_state(w: &World, account: i64, key: Option<&str>) -> (i64, i64, i64) {
    let conn = raw(&w.dir);
    let sums = |sql: &str| -> i64 {
        conn.query_row(sql, (account, key.map(str::to_owned)), |r| r.get(0))
            .unwrap()
    };
    // Lines in force: an entry that was taken back and its reversal are both left out.
    let live = "FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                WHERE l.account_id = ?1 AND l.pukhta_person_key IS ?2
                  AND e.reverses_entry_id IS NULL
                  AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)";
    let dr = sums(&format!("SELECT COALESCE(SUM(l.dr_paisa), 0) {live}"));
    let cr = sums(&format!("SELECT COALESCE(SUM(l.cr_paisa), 0) {live}"));
    let matched = sums(
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations WHERE account_id = ?1 AND pukhta_person_key IS ?2 AND is_active = 1",
    );
    (dr - matched, cr - matched, dr - cr)
}

#[test]
fn after_any_sequence_of_actions_the_books_hold() {
    let w = world_with(100_000_000, true);
    set_today(&w.env, 2026, 10, 12);
    give_advance_on(&w, w.rahim, "30,000", "10-10-2026");
    give_advance_on(&w, w.karim, "10,000", "10-10-2026");
    let mut done = [0_u32; 6];
    let mut rule = banana_per_lot_rule(&w, None, "300", true);
    for seed in [
        0x9E37_79B9_7F4A_7C15_u64,
        0xD1B5_4A32_D192_ED03,
        0x2545_F491_4F6C_DD1D,
        0x1234_5678_9ABC_DEF1,
        0xA5A5_A5A5_5A5A_5A5B,
        0x0F0F_1357_9BDF_2461,
    ] {
        let mut rng = Rng(seed);
        for step in 0..30 {
            // The rule changes between edits: off, then on again with another value.
            if step == 10 {
                rule = banana_per_lot_rule(&w, Some(&rule), "300", false);
            } else if step == 20 {
                rule = banana_per_lot_rule(&w, Some(&rule), "450", true);
            }
            let kind = rng.below(10);
            let (slot, ok) = match kind {
                0..=2 => {
                    let d = random_save(&w, &mut rng, step);
                    let ok = bikri::preview(&w.env.ctx, &d).unwrap().errors.is_empty()
                        && save(&w, &d).is_ok();
                    (0, ok)
                }
                3 | 4 => (1, random_later(&w, &mut rng, step)),
                5..=7 => (2, random_edit(&w, &mut rng)),
                8 => {
                    // Cancels are rarer than the rest so the run keeps something to work on.
                    if rng.below(3) == 0 {
                        (3, random_cancel(&w, &mut rng))
                    } else {
                        (4, random_write_off(&w, &mut rng))
                    }
                }
                _ => {
                    let (account, key) = keys(&w)[rng.index(2)];
                    let _ = key;
                    tools::receipt_on(
                        &w,
                        account,
                        &(100 + rng.below(8_000)).to_string(),
                        &day_text(12),
                    );
                    (5, true)
                }
            };
            if ok {
                done[slot] += 1;
            }
            check_books(&w, seed, step);
        }
    }
    // The run did real work of each kind.
    eprintln!("saves, later sales, edits, cancels, write-offs, receipts: {done:?}");
    assert!(done[0] >= 8, "saves: {done:?}");
    assert!(done[1] >= 3, "later sales: {done:?}");
    assert!(done[2] >= 8, "edits: {done:?}");
    assert!(done[3] >= 1, "cancels: {done:?}");
    assert!(done[5] >= 3, "receipts: {done:?}");
    // The per-lot rule was really met: some line carried a per-lot commission.
    assert!(
        sql_i64(
            &w,
            "SELECT COUNT(*) FROM bikri_sales WHERE seller_comm_method = 'per_lot' AND seller_comm_paisa > 0"
        ) >= 1,
        "no line carried a per-lot commission"
    );
    assert_chain_ok(&w.dir);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}
