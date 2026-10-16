//! The password hasher: real Argon2id with the recommended settings (NFR-SEC-1, S2).
#![cfg(test)]

use std::time::Instant;

use mandi_services::Secret;
use mandi_services::ports::{Argon2Hasher, OsRandom, PasswordHasher, RandomSource};

#[test]
fn recommended_settings_are_argon2id_v19_with_19_mib() {
    let hasher = Argon2Hasher::recommended(OsRandom);
    let started = Instant::now();
    let hash = hasher.hash(&Secret::new("secret1")).unwrap();
    let took = started.elapsed();
    assert!(
        hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
        "{hash}"
    );
    assert!(!hash.contains("secret1"));
    assert!(hasher.verify(&Secret::new("secret1"), &hash));
    assert!(!hasher.verify(&Secret::new("secret2"), &hash));
    assert!(!hasher.verify(&Secret::new("Secret1"), &hash));
    assert!(!hasher.verify(&Secret::new(""), &hash));
    // About 60 ms on a laptop in a release build; anything near a second would hurt the
    // sign-in target (NFR-SPD-2). The dev profile builds argon2 optimised, so this holds in tests.
    assert!(took.as_millis() < 1500, "hashing took {took:?}");
}

#[test]
fn every_hash_has_its_own_salt() {
    let hasher = Argon2Hasher::with_cost(OsRandom, 8, 1).unwrap();
    let password = Secret::new("same-password");
    let a = hasher.hash(&password).unwrap();
    let b = hasher.hash(&password).unwrap();
    assert_ne!(a, b);
    assert!(hasher.verify(&password, &a) && hasher.verify(&password, &b));
}

#[test]
fn a_hash_that_cannot_be_read_is_a_mismatch_not_a_crash() {
    let hasher = Argon2Hasher::with_cost(OsRandom, 8, 1).unwrap();
    let password = Secret::new("secret1");
    for bad in [
        "",
        "plain text",
        "$argon2id$",
        "$argon2id$v=19$m=8,t=1,p=1$$",
        "$2b$12$abcdefghijklmnopqrstuv",
        "\u{0}",
    ] {
        assert!(!hasher.verify(&password, bad), "{bad:?}");
    }
}

#[test]
fn urdu_and_long_passwords_hash_and_verify() {
    let hasher = Argon2Hasher::with_cost(OsRandom, 8, 1).unwrap();
    for text in ["پاسورڈ12", &"x".repeat(128), "with spaces  and tabs\t"] {
        let secret = Secret::new(text);
        let hash = hasher.hash(&secret).unwrap();
        assert!(hasher.verify(&secret, &hash), "{text:?}");
    }
}

#[test]
fn bad_costs_are_an_error_not_a_panic() {
    assert!(
        Argon2Hasher::with_cost(OsRandom, 1, 1).is_err(),
        "memory below 8 KiB"
    );
    assert!(
        Argon2Hasher::with_cost(OsRandom, 8, 0).is_err(),
        "no passes"
    );
}

#[test]
fn the_os_random_source_fills_and_differs() {
    let mut a = [0_u8; 32];
    let mut b = [0_u8; 32];
    OsRandom.fill(&mut a).unwrap();
    OsRandom.fill(&mut b).unwrap();
    assert_ne!(a, b);
    assert_ne!(a, [0_u8; 32]);
}
