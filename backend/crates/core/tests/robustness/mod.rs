//! Shared helpers for the robustness tests. See `robustness_core.rs`.

mod dates;
mod money;
mod parse;
mod splits;
mod text;
mod words;

use std::panic::{AssertUnwindSafe, catch_unwind};

use mandi_core::money::{Bps, MAX_RUPEES, Paisa, Qty};

/// The largest whole-rupee paisa value: 92,233,720,368,547,758 rupees.
pub const MAX_PAISA: i64 = i64::MAX - i64::MAX % 100;
/// The smallest whole-rupee paisa value.
pub const MIN_PAISA: i64 = i64::MIN - i64::MIN % 100;

pub fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

pub fn paisa(p: i64) -> Paisa {
    Paisa::try_from_paisa(p).unwrap()
}

pub fn max_paisa() -> Paisa {
    paisa(MAX_PAISA)
}

pub fn min_paisa() -> Paisa {
    paisa(MIN_PAISA)
}

/// The largest single input value, 10,00,00,00,000 rupees (Code Standards 4.2).
pub fn max_input() -> Paisa {
    rs(MAX_RUPEES)
}

pub fn bps(v: i32) -> Bps {
    Bps::new(v).unwrap()
}

pub fn qty(v: i64) -> Qty {
    Qty::new(v).unwrap()
}

/// Runs `f` and fails the test with `what` if it panics.
pub fn no_panic<T>(what: &str, f: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(_) => panic!("panicked on {what}"),
    }
}

/// Hostile strings used against every text function. Each is a plain `String`, so the
/// list can hold very long values too.
pub fn hostile_strings() -> Vec<String> {
    let mut out: Vec<String> = [
        "",
        " ",
        "\u{0000}",
        "\u{FEFF}",
        "\u{200B}\u{200C}\u{200D}\u{200E}\u{200F}",
        "\u{202E}اکرم\u{202C}",
        "\u{2066}600\u{2069}",
        "' OR 1=1 --",
        "Robert'); DROP TABLE accounts;--",
        "<script>alert(1)</script>",
        "..\\..\\windows\\system32",
        "../../etc/passwd",
        "%s%n%x",
        "\u{D7FF}\u{E000}\u{FFFD}\u{10FFFF}",
        "👨‍👩‍👧 اکرم 🍌",
        "a\u{0301}\u{0302}\u{0303}\u{0304}",
        "ا\u{0651}\u{0651}\u{0651}\u{064B}\u{064E}",
        "١٢٣٤٥٦٧٨٩٠۱۲۳۴۵۶۷۸۹۰",
        "６００",
        "𝟔𝟎𝟎",
        "1e5",
        "0x10",
        "NaN",
        "-",
        "+",
        "+-+-",
        "،۔٫٬",
        "\t\r\n\u{000B}\u{000C}\u{0085}\u{2028}\u{2029}",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    out.push("9".repeat(10_000));
    out.push("0".repeat(10_000) + "1");
    out.push("1,".repeat(5_000) + "000");
    out.push("1+".repeat(5_000) + "1");
    out.push("ا".repeat(10_000));
    out.push("a".to_owned() + &"\u{0301}".repeat(10_000));
    out.push("ب".to_owned() + &"\u{0651}".repeat(10_000));
    out.push("\u{202E}".repeat(10_000));
    out.push("🍌".repeat(10_000));
    out.push("Arshad Multani ارشد ملتانی ".repeat(400));
    out
}
