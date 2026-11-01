//! Property tests (Code Standards 9.3, money agent case 18): random drafts keep the books
//! balanced and every total equals the sum of its parts.

use mandi_core::bikri::{LotInput, PaymentType, PukhtaInput, SaleInput};
use mandi_core::ids::AccountId;
use mandi_core::money::Paisa;
use proptest::prelude::*;

use super::*;
use crate::ledger::Side;
use crate::partners::{Nature, share_bases};

fn arb_sale() -> impl Strategy<Value = SaleInput> {
    (
        prop_oneof![
            Just(AKRAM),
            Just(ARSHAD),
            Just(NAQDI),
            Just(PUKHTA_BUYER),
            Just(CASH_ONLY_BUYER),
            Just(PARTY_BUYER),
            Just(PARTY_BUYER_CREDIT)
        ],
        1u32..=100,
        1u32..=5_000,
        any::<bool>(),
        1u32..=5_000,
        prop_oneof![
            Just(None),
            Just(Some(PaymentType::Credit)),
            Just(Some(PaymentType::Cash))
        ],
        prop::option::of(0u32..=100),
        prop::option::of(0u32..=50),
    )
        .prop_map(
            |(buyer, qty, rate, differs, buyer_rate, payment, percent, laga)| {
                let mut s = sale(buyer, &qty.to_string(), &rate.to_string());
                s.rate_differs = differs;
                s.buyer_rate = differs.then(|| buyer_rate.to_string());
                s.payment_type = payment;
                if buyer == PUKHTA_BUYER {
                    s.pukhta = Some(PukhtaInput {
                        name_ur: "شاہد".into(),
                        name_en: "Shahid".into(),
                        phone: "0300-0000000".into(),
                    });
                }
                s.seller_comm = percent.map(|v| mandi_core::bikri::LineChargeInput {
                    method: mandi_core::bikri::LineChargeMethod::Percent,
                    value: v.to_string(),
                });
                s.laga = laga.map(|v| mandi_core::bikri::LineChargeInput {
                    method: mandi_core::bikri::LineChargeMethod::PerUnit,
                    value: v.to_string(),
                });
                s
            },
        )
}

fn arb_lot() -> impl Strategy<Value = LotInput> {
    (
        prop_oneof![
            Just(RAHIM),
            Just(KARIM),
            Just(PUKHTA_SELLER),
            Just(PARTY_SELLER)
        ],
        prop_oneof![Just(BANANA), Just(APPLE), Just(MOOLI)],
        1u32..=300,
        prop::option::of(0u32..=5_000),
        any::<bool>(),
        prop::option::of(0u32..=20_000),
        prop::option::of(0u32..=20),
        prop::collection::vec(arb_sale(), 1..=6),
    )
        .prop_map(
            |(seller, item, qty, freight, seller_paid, cash, labour, sales)| {
                let mut l = lot(seller, item, &qty.to_string());
                l.freight = freight.map(|f| f.to_string());
                if seller_paid {
                    l.freight_paid_by = mandi_core::bikri::FreightPaidBy::Seller;
                }
                l.cash_paid = cash.map(|c| c.to_string());
                l.labour = labour.and_then(|v| per_unit(&v.to_string()));
                if seller == PUKHTA_SELLER {
                    l.pukhta = Some(PukhtaInput {
                        name_ur: "ناصر".into(),
                        name_en: "Nasir".into(),
                        phone: "0300-0000000".into(),
                    });
                }
                l.sales = sales;
                l
            },
        )
}

/// Σ of one side over the lines of an account in a set.
fn side_total(entry: &crate::ledger::BalancedEntry, accounts: &[i64], side: Side) -> Paisa {
    entry
        .lines()
        .iter()
        .filter(|l| l.side == side && accounts.iter().any(|a| AccountId::new(*a) == l.account_id))
        .fold(Paisa::ZERO, |sum, l| sum.checked_add(l.amount).unwrap())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn the_books_balance_and_totals_equal_their_parts(lots in prop::collection::vec(arb_lot(), 1..=3)) {
        let draft = draft(lots);
        let calc = run(&draft);
        // The same draft gives the same answer.
        prop_assert_eq!(&calc, &run(&draft));

        let mut all_lines = Vec::new();
        for lot in &calc.lots {
            all_lines.extend(lot.lines.iter().filter_map(|l| l.calc.as_ref()));
        }
        // Every amount is a whole rupee.
        for l in &all_lines {
            for amount in [l.seller_amount, l.buyer_value, l.seller_comm.amount, l.buyer_comm.amount, l.laga.amount, l.buyer_total, l.rate_diff] {
                prop_assert_eq!(amount.as_paisa() % 100, 0);
            }
            // C5 and C19 hold line by line.
            prop_assert_eq!(l.buyer_total, l.buyer_value.checked_add(l.buyer_comm.amount).unwrap().checked_add(l.laga.amount).unwrap());
            prop_assert_eq!(l.rate_diff, l.buyer_value.checked_sub(l.seller_amount).unwrap());
        }
        // Per lot: C7 and the relation of C8 to نقد; the lot totals are sums of its lines.
        for lot in &calc.lots {
            if let Some(t) = &lot.totals {
                prop_assert_eq!(t.net, t.seller_credit.checked_sub(t.cash_paid).unwrap());
                prop_assert_eq!(t.net, t.gross.checked_sub(t.deductions).unwrap());
                let lines: Vec<_> = lot.lines.iter().filter_map(|l| l.calc.as_ref()).collect();
                let comm = lines.iter().fold(Paisa::ZERO, |s, l| s.checked_add(l.seller_comm.amount).unwrap());
                prop_assert_eq!(t.seller_comm, comm);
            }
        }
        // The whole is the sum of its lots.
        let gross = calc.lots.iter().filter_map(|l| l.totals.as_ref()).fold(Paisa::ZERO, |s, t| s.checked_add(t.gross).unwrap());
        prop_assert_eq!(calc.summary.gross, gross);
        prop_assert_eq!(calc.summary.net, calc.summary.seller_credit.checked_sub(calc.summary.cash_paid).unwrap());

        if calc.journal.lots.is_empty() {
            prop_assert_eq!(calc.summary.gross, Paisa::ZERO);
        } else {
            // The entry always balances (build_bikri_entry refuses otherwise).
            let entry = entry_of(&calc);
            let sys = system_accounts();
            let buyers = [
                AKRAM,
                ARSHAD,
                NAQDI,
                PUKHTA_BUYER,
                CASH_ONLY_BUYER,
                PARTY_BUYER,
                PARTY_BUYER_CREDIT,
            ];
            // Σ buyer totals = Σ debits on buyer accounts (the cash part only credits them).
            prop_assert_eq!(side_total(&entry, &buyers, Side::Dr), calc.summary.buyer_total);
            // Σ line seller commission = the credit on 04010001.
            prop_assert_eq!(side_total(&entry, &[102], Side::Cr), calc.summary.seller_comm);
            // The shared income of the summary is the income base of the entry.
            let bases = share_bases(&entry, |id| {
                if (sys.seller_commission.get()..=sys.rate_diff.get()).contains(&id.get()) { Nature::Income } else { Nature::Other }
            }).unwrap();
            prop_assert_eq!(bases.income, calc.summary.shared_income);
            // The cash change of the entry equals the cash lines minus نقد.
            let change = crate::cash::net_change(&entry, sys.cash).unwrap();
            prop_assert_eq!(change, calc.summary.buyer_cash.checked_sub(calc.summary.cash_paid).unwrap());
        }
    }

    // Plan 47 (PRD FR-13.10, FR-13.11): a one-time seller and a پختہ بیوپاری with his own account are
    // both paid the seller credit on the spot when نقد is left empty. The one with his own account
    // can have an open advance that his credit adjusts, and then the cash is the credit LESS that
    // advance (A79), so credit = cash + advance used, and the money he is owed after the save is 0
    // whenever the credit is above 0. A typed نقد is kept as typed.
    #[test]
    fn a_pukhta_seller_with_no_cash_typed_is_paid_his_credit_less_the_advance_it_adjusts(
        own_account in any::<bool>(),
        qty in 1u32..=100,
        rate in 1u32..=5_000,
        freight in prop::option::of(0u32..=5_000),
        typed in prop::option::of(0u32..=20_000),
        advance in 0u32..=30_000,
        ticked in any::<bool>(),
    ) {
        let seller = if own_account { PARTY_SELLER } else { PUKHTA_SELLER };
        let mut l = lot(seller, APPLE, &qty.to_string());
        l.freight = freight.map(|f| f.to_string());
        l.cash_paid = typed.map(|c| c.to_string());
        l.adjust_advance = ticked;
        if !own_account {
            l.pukhta = Some(PukhtaInput {
                name_ur: "ناصر".into(),
                name_en: "Nasir".into(),
                phone: "0300-0000000".into(),
            });
        }
        l.sales = vec![sale(AKRAM, &qty.to_string(), &rate.to_string())];
        let calc = run_with(
            &draft(vec![l]),
            &{
                let mut facts = lookups();
                if let Some(a) = facts.accounts.get_mut(&PARTY_SELLER) {
                    a.advance_available = p(i64::from(advance));
                }
                facts
            },
            default_rules(),
            switches(),
        );
        if let Some(t) = calc.lots.first().and_then(|lot| lot.totals.as_ref()) {
            // What his credit adjusts: only a party seller with the tick on and a credit above 0.
            let used = if own_account && ticked && t.seller_credit > Paisa::ZERO {
                t.seller_credit.min(p(i64::from(advance)))
            } else {
                Paisa::ZERO
            };
            match typed {
                Some(cash) => prop_assert_eq!(t.cash_paid.as_paisa(), i64::from(cash) * 100),
                None if t.seller_credit > Paisa::ZERO => {
                    prop_assert_eq!(t.cash_paid.checked_add(used).unwrap(), t.seller_credit);
                    prop_assert_eq!(t.net, used);
                }
                None => prop_assert_eq!(t.cash_paid, Paisa::ZERO),
            }
            prop_assert_eq!(t.net, t.seller_credit.checked_sub(t.cash_paid).unwrap());
        }
    }
}
