//! Receipts, payments and their helpers on the NFR-SCL-1 volume (plan 15).
//!
//! - `vouchers/save_receipt_busiest_buyer`, `vouchers/save_payment_busiest_seller`,
//!   `vouchers/save_cash_lent_busiest_buyer`: NFR-SPD-9, 150 ms. Each writes, so it runs on a
//!   private copy. The volume is made with plain SQL and has no matching rows for its old lines;
//!   the set-up writes the matching of the two busiest parties' history once, as every save of a
//!   real file keeps it, then the warm-up runs before the measured samples.
//! - `vouchers/preview_receipt_busiest_buyer`, `vouchers/preview_payment_busiest_seller`: the
//!   preview on each pause in typing, held to the 150 ms of the live summary (NFR-SPD-7).
//! - The same three for an ordinary buyer (`save_receipt_ordinary_buyer`,
//!   `preview_receipt_ordinary_buyer`, `open_bills_ordinary_buyer`): about 90 lines, what the
//!   Owner meets all day. The busiest parties are the stress case.
//! - `assist/open_bills_busiest_buyer`, `assist/open_payables_busiest_seller`: the bills and the
//!   payable postings of a party just picked, held to the party card's 100 ms (NFR-SPD-21).

mod support;

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::core::vouchers::{
    OpenBillsInput, OpenPayablesInput, PaymentInput, ReceiptInput, VoucherEntry, VoucherMode,
    VoucherPreviewInput,
};
use mandi_services::vouchers;
use support::{copy_of_volume, ensure_volume, fail, open};

fn flat<'a>(
    c: &'a mut Criterion,
    name: &str,
    samples: usize,
    seconds: u64,
) -> criterion::BenchmarkGroup<'a, criterion::measurement::WallTime> {
    let mut group = c.benchmark_group(name);
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(samples);
    group.measurement_time(Duration::from_secs(seconds));
    group
}

fn receipt(account: i64) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: "1,000".into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn payment(account: i64, reason: &str) -> PaymentInput {
    PaymentInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: "1,000".into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        reason: reason.into(),
        narration: String::new(),
        // The volume has little cash; the question is part of the save, not of the speed.
        confirm_cash_below_zero: true,
    }
}

fn save_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    support::describe(&volume);
    let copy = copy_of_volume(&volume);
    let (env, actor) = open(copy.path());
    let buyer = volume.busy_buyer;
    let seller = volume.busy_seller;
    let ordinary = volume.median_buyer;
    // The volume is made with plain SQL, so its old lines have no matching rows. A real file
    // does (every save keeps them canonical), so write the matching of the two busiest parties'
    // history once, before anything is measured.
    let now = env.ctx.now_text();
    if let Err(error) = env.ctx.db().write(|tx| {
        mandi_services::matching::rematch(tx, actor.user_id, &now, buyer, None)?;
        mandi_services::matching::rematch(tx, actor.user_id, &now, ordinary, None)?;
        mandi_services::matching::rematch_advances(tx, actor.user_id, &now, seller)?;
        Ok::<(), mandi_services::ServiceError>(())
    }) {
        fail(&format!(
            "the matching of the history was not written: {error}"
        ));
    }
    let receipt_input = receipt(buyer);
    let seller_payment = payment(seller, "");
    let cash_lent = payment(buyer, "x");
    // Each must really be saved, not refused: a refusal is fast and would hide a slow path.
    for (name, result) in [
        (
            "receipt",
            vouchers::save_receipt(&env.ctx, &actor, &receipt_input).map(|s| s.number),
        ),
        (
            "payment to a seller",
            vouchers::save_payment(&env.ctx, &actor, &seller_payment).map(|s| s.number),
        ),
        (
            "cash lent",
            vouchers::save_payment(&env.ctx, &actor, &cash_lent).map(|s| s.number),
        ),
    ] {
        if let Err(error) = result {
            fail(&format!("the {name} was not saved: {error}"));
        }
    }
    let mut group = flat(c, "vouchers", 50, 6);
    group.bench_function("save_receipt_busiest_buyer", |b| {
        b.iter(
            || match vouchers::save_receipt(&env.ctx, &actor, black_box(&receipt_input)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    // An ordinary buyer (about 90 lines), which is what the Owner meets all day.
    let ordinary_receipt = receipt(ordinary);
    if let Err(error) = vouchers::save_receipt(&env.ctx, &actor, &ordinary_receipt) {
        fail(&format!(
            "the receipt of an ordinary buyer was not saved: {error}"
        ));
    }
    group.bench_function("save_receipt_ordinary_buyer", |b| {
        b.iter(
            || match vouchers::save_receipt(&env.ctx, &actor, black_box(&ordinary_receipt)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    group.bench_function("save_payment_busiest_seller", |b| {
        b.iter(
            || match vouchers::save_payment(&env.ctx, &actor, black_box(&seller_payment)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    group.bench_function("save_cash_lent_busiest_buyer", |b| {
        b.iter(
            || match vouchers::save_payment(&env.ctx, &actor, black_box(&cash_lent)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    // The previews and the reads, on the same copy.
    let preview_receipt = VoucherPreviewInput {
        entry: VoucherEntry::Receipt(receipt(buyer)),
        replacing_voucher_id: None,
    };
    let preview_payment = VoucherPreviewInput {
        entry: VoucherEntry::Payment(payment(seller, "")),
        replacing_voucher_id: None,
    };
    group.bench_function("preview_receipt_busiest_buyer", |b| {
        b.iter(
            || match vouchers::preview(&env.ctx, black_box(&preview_receipt)) {
                Ok(preview) => black_box(preview.bill_effects.len()),
                Err(error) => fail(&format!("preview failed: {error}")),
            },
        );
    });
    let preview_ordinary = VoucherPreviewInput {
        entry: VoucherEntry::Receipt(receipt(ordinary)),
        replacing_voucher_id: None,
    };
    group.bench_function("preview_receipt_ordinary_buyer", |b| {
        b.iter(
            || match vouchers::preview(&env.ctx, black_box(&preview_ordinary)) {
                Ok(preview) => black_box(preview.bill_effects.len()),
                Err(error) => fail(&format!("preview failed: {error}")),
            },
        );
    });
    group.bench_function("preview_payment_busiest_seller", |b| {
        b.iter(
            || match vouchers::preview(&env.ctx, black_box(&preview_payment)) {
                Ok(preview) => black_box(preview.warnings.len()),
                Err(error) => fail(&format!("preview failed: {error}")),
            },
        );
    });
    group.finish();
    let mut group = flat(c, "assist", 50, 6);
    let bills = OpenBillsInput {
        account_id: buyer,
        person_key: None,
    };
    let payables = OpenPayablesInput {
        seller_id: seller,
        person_key: None,
        include_settled: false,
    };
    group.bench_function("open_bills_busiest_buyer", |b| {
        b.iter(|| match vouchers::open_bills(&env.ctx, black_box(&bills)) {
            Ok(found) => black_box(found.bills.len()),
            Err(error) => fail(&format!("open bills failed: {error}")),
        });
    });
    let ordinary_bills = OpenBillsInput {
        account_id: ordinary,
        person_key: None,
    };
    group.bench_function("open_bills_ordinary_buyer", |b| {
        b.iter(
            || match vouchers::open_bills(&env.ctx, black_box(&ordinary_bills)) {
                Ok(found) => black_box(found.bills.len()),
                Err(error) => fail(&format!("open bills failed: {error}")),
            },
        );
    });
    group.bench_function("open_payables_busiest_seller", |b| {
        b.iter(
            || match vouchers::open_payables(&env.ctx, black_box(&payables)) {
                Ok(found) => black_box(found.postings.len()),
                Err(error) => fail(&format!("open payables failed: {error}")),
            },
        );
    });
    group.finish();
}

criterion_group!(benches, save_speed);
criterion_main!(benches);
