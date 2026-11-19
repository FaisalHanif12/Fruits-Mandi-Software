//! Worked numbers of money agent Section 8 and properties of `match_bills`.

use super::*;
use proptest::prelude::*;

fn p(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn open(line_id: i64, rupees: i64) -> Open {
    Open {
        line_id,
        open: p(rupees),
    }
}

fn rows(bills: &[Open], credits: &[Open]) -> Vec<(i64, i64, i64)> {
    match_bills(bills, credits)
        .unwrap()
        .into_iter()
        .map(|a| (a.bill_line_id, a.credit_line_id, a.amount.rupees()))
        .collect()
}

#[test]
fn a_receipt_settles_the_oldest_bill_first_money_agent_8_1() {
    // PRD 4.6: اکرم owes 12,880 and pays 10,000: 10,000 to the bill, 2,880 stays open.
    assert_eq!(
        rows(&[open(1, 12_880)], &[open(9, 10_000)]),
        [(1, 9, 10_000)]
    );
    // Bills 12,880 (09-10) and 5,000 (10-10), receipt 15,000: 12,880 and 2,120.
    assert_eq!(
        rows(&[open(1, 12_880), open(2, 5_000)], &[open(9, 15_000)]),
        [(1, 9, 12_880), (2, 9, 2_120)]
    );
}

#[test]
fn a_credit_that_is_more_than_all_bills_stays_unmatched() {
    // PRD FR-15.5: only what the bills can take is matched.
    assert_eq!(rows(&[open(1, 3_000)], &[open(9, 5_000)]), [(1, 9, 3_000)]);
}

#[test]
fn an_old_unmatched_credit_settles_the_next_bill_example_b2() {
    // Money agent question B2: a prepayment of 5,000 and a new credit bill of 12,880.
    assert_eq!(rows(&[open(7, 12_880)], &[open(3, 5_000)]), [(7, 3, 5_000)]);
}

#[test]
fn advances_are_adjusted_oldest_first_by_the_seller_credit() {
    // PRD FR-15.8: رحیم's advance 15,000, credit 24,316: adjusted 15,000 (fully), nothing left.
    assert_eq!(
        rows(&[open(1, 15_000)], &[open(5, 24_316)]),
        [(1, 5, 15_000)]
    );
    // An advance of 30,000: the credit 24,316 adjusts 24,316 and 5,684 stays open.
    assert_eq!(
        rows(&[open(1, 30_000)], &[open(5, 24_316)]),
        [(1, 5, 24_316)]
    );
    // Two advances 10,000 and 8,000, a credit of 12,000: 10,000 and 2,000.
    assert_eq!(
        rows(&[open(1, 10_000), open(2, 8_000)], &[open(5, 12_000)]),
        [(1, 5, 10_000), (2, 5, 2_000)]
    );
    // A credit of 6,000 against an advance of 15,000.
    assert_eq!(rows(&[open(1, 15_000)], &[open(5, 6_000)]), [(1, 5, 6_000)]);
}

#[test]
fn several_credits_use_the_bills_in_order_and_each_bill_once() {
    // Bills 100, 200; credits 150 then 200: 100 + 50 from the first credit, 150 from the second.
    assert_eq!(
        rows(&[open(1, 100), open(2, 200)], &[open(8, 150), open(9, 200)]),
        [(1, 8, 100), (2, 8, 50), (2, 9, 150)]
    );
}

#[test]
fn lines_with_nothing_open_are_skipped_and_nothing_gives_nothing() {
    assert_eq!(
        rows(&[open(1, 0), open(2, 50)], &[open(9, 0), open(10, 30)]),
        [(2, 10, 30)]
    );
    assert!(rows(&[], &[open(9, 10)]).is_empty());
    assert!(rows(&[open(1, 10)], &[]).is_empty());
}

#[test]
fn a_negative_open_amount_is_a_bug_signal() {
    let bad = Open {
        line_id: 1,
        open: Paisa::try_from_paisa(-100).unwrap(),
    };
    assert_eq!(
        match_bills(&[bad], &[open(2, 5)]),
        Err(MoneyError::NegativeValue)
    );
    assert_eq!(
        match_bills(&[open(2, 5)], &[bad]),
        Err(MoneyError::NegativeValue)
    );
}

fn lines(amounts: &[i64], first_id: i64) -> Vec<Open> {
    amounts
        .iter()
        .zip(first_id..)
        .map(|(a, id)| open(id, *a))
        .collect()
}

proptest! {
    // Money agent 11.2 point 7: Σ allocations + unmatched = credit, no bill above its open
    // amount, no credit above its own, and the matched total is the smaller of the two sums.
    #[test]
    fn matching_never_gives_more_than_a_line_has(
        bills in proptest::collection::vec(0_i64..50_000, 0..12),
        credits in proptest::collection::vec(0_i64..50_000, 0..12),
    ) {
        let b = lines(&bills, 1);
        let c = lines(&credits, 1_000);
        let out = match_bills(&b, &c).unwrap();
        let total: i64 = out.iter().map(|a| a.amount.rupees()).sum();
        let sum_b: i64 = bills.iter().sum();
        let sum_c: i64 = credits.iter().sum();
        prop_assert_eq!(total, sum_b.min(sum_c));
        for bill in &b {
            let used: i64 = out.iter().filter(|a| a.bill_line_id == bill.line_id).map(|a| a.amount.rupees()).sum();
            prop_assert!(used <= bill.open.rupees());
        }
        for credit in &c {
            let used: i64 = out.iter().filter(|a| a.credit_line_id == credit.line_id).map(|a| a.amount.rupees()).sum();
            prop_assert!(used <= credit.open.rupees());
        }
        prop_assert!(out.iter().all(|a| a.amount > Paisa::ZERO));
    }

    // Oldest first: a bill gets something only if every older bill is already full.
    #[test]
    fn a_later_bill_is_used_only_when_the_older_ones_are_full(
        bills in proptest::collection::vec(1_i64..50_000, 1..10),
        credits in proptest::collection::vec(1_i64..50_000, 1..10),
    ) {
        let b = lines(&bills, 1);
        let c = lines(&credits, 1_000);
        let out = match_bills(&b, &c).unwrap();
        let used = |id: i64| -> i64 {
            out.iter().filter(|a| a.bill_line_id == id).map(|a| a.amount.rupees()).sum()
        };
        for (index, bill) in b.iter().enumerate() {
            if used(bill.line_id) > 0 {
                for older in b.iter().take(index) {
                    prop_assert_eq!(used(older.line_id), older.open.rupees());
                }
            }
        }
    }
}

fn ranked(line_id: i64, rupees: i64, rank: usize) -> Ranked {
    Ranked {
        line: open(line_id, rupees),
        rank,
    }
}

fn before_rows(bills: &[Ranked], credits: &[Ranked]) -> Vec<(i64, i64, i64)> {
    match_before(bills, credits)
        .unwrap()
        .into_iter()
        .map(|a| (a.bill_line_id, a.credit_line_id, a.amount.rupees()))
        .collect()
}

#[test]
fn a_credit_adjusts_only_the_advances_before_it_prd_fr_15_8() {
    // رحیم's advance 15,000 and the credit 24,316 after it: 15,000 adjusted (E2E-1 to E2E-4).
    assert_eq!(
        before_rows(&[ranked(1, 15_000, 0)], &[ranked(9, 24_316, 1)]),
        [(1, 9, 15_000)]
    );
    // The credit first and the advance after it: nothing (money agent P6).
    assert!(before_rows(&[ranked(1, 15_000, 1)], &[ranked(9, 24_316, 0)]).is_empty());
    // Two advances, the credit between them: only the first is used.
    assert_eq!(
        before_rows(
            &[ranked(1, 10_000, 0), ranked(2, 10_000, 2)],
            &[ranked(9, 15_000, 1)]
        ),
        [(1, 9, 10_000)]
    );
    // Two credits and two advances in time order A1, C1, A2, C2: C1 takes A1 only; C2 takes
    // what is left of A1 and then A2.
    assert_eq!(
        before_rows(
            &[ranked(1, 10_000, 0), ranked(2, 5_000, 2)],
            &[ranked(8, 4_000, 1), ranked(9, 20_000, 3)]
        ),
        [(1, 8, 4_000), (1, 9, 6_000), (2, 9, 5_000)]
    );
}

proptest! {
    // With every advance before every credit it is the plain oldest-first matching, and in
    // general a credit never takes an advance that comes after it.
    #[test]
    fn match_before_is_match_bills_when_every_bill_is_older_and_never_looks_forward(
        bills in proptest::collection::vec(0_i64..50_000, 0..10),
        credits in proptest::collection::vec(0_i64..50_000, 0..10),
        ranks in proptest::collection::vec(any::<bool>(), 20),
    ) {
        // All bills first, then all credits.
        let b: Vec<Ranked> = bills.iter().zip(0..).map(|(a, i)| ranked(i64::from(i) + 1, *a, i as usize)).collect();
        let c: Vec<Ranked> = credits.iter().zip(0..).map(|(a, i)| ranked(i64::from(i) + 1_000, *a, 100 + i as usize)).collect();
        let plain = match_bills(&lines(&bills, 1), &lines(&credits, 1_000)).unwrap();
        prop_assert_eq!(match_before(&b, &c).unwrap(), plain);
        // Mixed order: each item is a bill or a credit by `ranks`, in time order.
        let mut mixed_bills = Vec::new();
        let mut mixed_credits = Vec::new();
        let (mut bi, mut ci) = (0, 0);
        for (rank, is_bill) in ranks.iter().enumerate() {
            if *is_bill && bi < bills.len() {
                mixed_bills.push(ranked(i64::try_from(bi).unwrap() + 1, bills[bi], rank));
                bi += 1;
            } else if ci < credits.len() {
                mixed_credits.push(ranked(i64::try_from(ci).unwrap() + 1_000, credits[ci], rank));
                ci += 1;
            }
        }
        let out = match_before(&mixed_bills, &mixed_credits).unwrap();
        for row in &out {
            let bill_rank = mixed_bills.iter().find(|b| b.line.line_id == row.bill_line_id).unwrap().rank;
            let credit_rank = mixed_credits.iter().find(|c| c.line.line_id == row.credit_line_id).unwrap().rank;
            prop_assert!(bill_rank < credit_rank);
            prop_assert!(row.amount > Paisa::ZERO);
        }
        for bill in &mixed_bills {
            let used: i64 = out.iter().filter(|a| a.bill_line_id == bill.line.line_id).map(|a| a.amount.rupees()).sum();
            prop_assert!(used <= bill.line.open.rupees());
        }
        for credit in &mixed_credits {
            let used: i64 = out.iter().filter(|a| a.credit_line_id == credit.line.line_id).map(|a| a.amount.rupees()).sum();
            prop_assert!(used <= credit.line.open.rupees());
        }
    }
}
