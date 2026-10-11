//! The Owner's recovery code (NFR-SEC-12, PRD FR-8.5, plan 04 Section 4).
//!
//! 16 characters from an alphabet without look-alikes, shown as `XXXX-XXXX-XXXX-XXXX`. The
//! code is printed once. It is stored only as an Argon2id hash and is spent when it is used.
//! 30 letters and digits for 16 places is about 78 bits, far more than can be guessed.

/// Letters and digits without `0 O 1 I L U`, so a printed code is easy to read and type.
pub const ALPHABET: &[u8; 30] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";
/// Characters in a code, without dashes.
pub const CODE_CHARS: usize = 16;
/// Characters in each group of the shown code.
pub const GROUP_CHARS: usize = 4;
/// Random bytes under this limit are used; the others are skipped, so every letter is equally
/// likely (240 = 8 x 30).
const ACCEPT_BELOW: u8 = 240;

/// Builds a code from random bytes, skipping bytes that would favour some letters.
/// Returns the 16 characters without dashes, or `None` if the bytes ran out (the caller asks
/// for more bytes; 96 are enough for any normal run).
///
/// ```
/// use mandi_domain::auth::recovery::generate;
/// // Bytes 0, 1, 2 ... map to the letters A, B, C ...
/// let bytes: Vec<u8> = (0..16).collect();
/// assert_eq!(generate(&bytes).as_deref(), Some("ABCDEFGHJKMNPQRS"));
/// assert_eq!(generate(&[0, 1, 2]), None);
/// ```
pub fn generate(random_bytes: &[u8]) -> Option<String> {
    let mut code = String::with_capacity(CODE_CHARS);
    for &byte in random_bytes {
        if byte >= ACCEPT_BELOW {
            continue;
        }
        let index = usize::from(byte).checked_rem(ALPHABET.len())?;
        code.push(char::from(*ALPHABET.get(index)?));
        if code.len() == CODE_CHARS {
            return Some(code);
        }
    }
    None
}

/// Puts dashes in: `ABCDEFGHJKMNPQRS` becomes `ABCD-EFGH-JKMN-PQRS`.
pub fn format_code(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    chars
        .chunks(GROUP_CHARS)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Reads a typed code: dashes, spaces and lower case are fine. `None` if it is not 16 letters
/// and digits of the alphabet. This is the text that is hashed and compared.
///
/// ```
/// use mandi_domain::auth::recovery::normalize;
/// assert_eq!(normalize("abcd-efgh jkmn-pqrs").as_deref(), Some("ABCDEFGHJKMNPQRS"));
/// assert_eq!(normalize("ABCD-EFGH-JKMN"), None);
/// assert_eq!(normalize("ABCD-EFGH-JKMN-PQR0"), None); // 0 is not in the alphabet
/// ```
pub fn normalize(input: &str) -> Option<String> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(char::to_uppercase)
        .collect();
    let valid =
        cleaned.chars().count() == CODE_CHARS && cleaned.bytes().all(|b| ALPHABET.contains(&b));
    valid.then_some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabet_has_no_look_alikes() {
        assert_eq!(ALPHABET.len(), 30);
        for banned in b"01OILU" {
            assert!(!ALPHABET.contains(banned), "{}", char::from(*banned));
        }
        let mut sorted = ALPHABET.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 30, "no letter twice");
    }

    #[test]
    fn biased_bytes_are_skipped() {
        // 240 and above are skipped. 239 is the last one used: 239 % 30 = 29, the last letter.
        let mut bytes = vec![255, 240, 250, 239];
        bytes.extend(0..15);
        let code = generate(&bytes).unwrap();
        assert_eq!(code.chars().next(), Some('9'));
        assert_eq!(code.len(), 16);
    }

    #[test]
    fn every_letter_is_equally_likely() {
        // All 240 accepted byte values, 8 times each letter.
        let bytes: Vec<u8> = (0..240).map(|b| u8::try_from(b).unwrap()).collect();
        let mut counts = [0_u32; 30];
        for &b in &bytes {
            counts[usize::from(b) % 30] += 1;
        }
        assert!(counts.iter().all(|&c| c == 8));
    }

    #[test]
    fn show_and_read_round_trip() {
        let raw = "ABCDEFGHJKMNPQRS";
        let shown = format_code(raw);
        assert_eq!(shown, "ABCD-EFGH-JKMN-PQRS");
        assert_eq!(normalize(&shown).as_deref(), Some(raw));
        assert_eq!(normalize(&shown.to_lowercase()).as_deref(), Some(raw));
        assert_eq!(normalize(&format!("  {shown}  ")).as_deref(), Some(raw));
    }

    #[test]
    fn wrong_codes_are_refused() {
        for bad in [
            "",
            "ABCD",
            "ABCD-EFGH-JKMN-PQRSA",
            "ABCD-EFGH-JKMN-PQR1",
            "ABCD-EFGH-JKMN-PQRé",
            "ÄBCD-EFGH-JKMN-PQRS",
        ] {
            assert_eq!(normalize(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_short_run_of_bytes_gives_none_not_a_panic() {
        assert_eq!(generate(&[]), None);
        assert_eq!(generate(&[255; 100]), None);
    }
}
