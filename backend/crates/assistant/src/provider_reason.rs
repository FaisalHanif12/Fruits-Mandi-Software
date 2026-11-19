//! The reason a provider gives when it refuses a call (live run 2: a 403 told us nothing).
//!
//! The model client used to drop the body of a non-2xx reply, so a 403 came back as "other, 403".
//! [`reason_of`] keeps what the provider said, in one short line: the JSON `error.message` and
//! `error.code` (and, for a reply OpenRouter passes on from another provider, that provider's name
//! and its own message), else the first [`MAX_REASON_CHARS`] characters of the body.
//!
//! The reason is shown to the Owner and to us, so nothing that looks like a key is ever kept
//! (NFR-OBS-2, NFR-SEC-7, Code Standards Q8): the key of this very call is cut out wherever it
//! appears, a word that starts with `sk-` or is a long run of letters and digits is hidden, and the
//! value after `Authorization`, `Bearer` or an API key header name is hidden. The scrub runs on the
//! whole bounded body BEFORE the cut to 300 characters, so a key can not be cut into a piece that
//! no rule would see. The metadata of a reply (for example the text a moderation check flagged) is
//! never read, only the provider's name and its raw error message.
//!
//! A web address in the reason is cut to its host: `https://openrouter.ai/settings/keys/k_ab12` is
//! kept as `openrouter.ai/[hidden]`. The path and the query can hold the id of the key (not the
//! secret, but it names the key), so the id never stays. A host with no path is kept as it is.
//!
//! Example: the body `{"error":{"code":403,"message":"Key limit exceeded"}}` gives
//! `403: Key limit exceeded`; the body `Authorization: Bearer sk-or-v1-abc123` gives
//! `Authorization: Bearer [hidden]`.

use serde_json::Value;

/// The most characters of a reason.
pub const MAX_REASON_CHARS: usize = 300;
/// The most bytes read from the body of an error reply.
pub const MAX_ERROR_BODY_BYTES: u64 = 16 * 1024;

/// What a hidden value is written as.
const HIDDEN: &str = "[hidden]";
/// A run of letters, digits, `-` and `_` this long is taken for a key (hex keys are 32, 40 or 64
/// characters; model names such as `claude-3-5-sonnet-20240620` are shorter).
const KEY_LIKE_LEN: usize = 32;
/// A word that starts with `sk-` is a key from this length on.
const MIN_PREFIXED_KEY_LEN: usize = 6;
/// After a header name or `Bearer`, a word is a value from this length on, or when it has a digit.
const MIN_VALUE_LEN: usize = 10;
/// A token of this kind starts with `eyJ` (the start of a JSON header, base 64).
const TOKEN_START: &str = "eyJ";
/// The names whose next value is hidden.
const NAME_WORDS: [&str; 6] = [
    "authorization",
    "proxy-authorization",
    "x-api-key",
    "api-key",
    "api_key",
    "apikey",
];

/// The reason in the body of an error reply, scrubbed and cut, or `None` when the body has none.
/// `key` is the key used for this call.
///
/// Example: `reason_of(br#"{"error":{"message":"No access","code":"forbidden"}}"#, "k")` is
/// `Some("forbidden: No access")`.
pub fn reason_of(body: &[u8], key: &str) -> Option<String> {
    let text = String::from_utf8_lossy(body);
    let found = from_json(&text).unwrap_or_else(|| text.into_owned());
    let clean = tidy(&scrub(&hide_url_paths(&found), key));
    let cut: String = clean.chars().take(MAX_REASON_CHARS).collect();
    let cut = cut.trim_end();
    (!cut.is_empty()).then(|| cut.to_owned())
}

/// The text of a string field, if it has some.
fn text_field<'a>(value: &'a Value, name: &str) -> Option<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

/// The error of a JSON body in the shapes of the API: `{"error":{"code","message","metadata"}}` or
/// `{"error":"text"}`. `None` when the body is not that.
fn from_json(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    let error = value.get("error")?;
    if let Some(plain) = error.as_str() {
        return Some(plain.to_owned());
    }
    let message = text_field(error, "message");
    let code = match error.get("code") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };
    let head = match (code, message) {
        (Some(c), Some(m)) => Some(format!("{c}: {m}")),
        (None, Some(m)) => Some(m.to_owned()),
        (Some(c), None) => Some(c),
        (None, None) => None,
    };
    let detail = error.get("metadata").and_then(provider_detail);
    match (head, detail) {
        (Some(h), Some(d)) => Some(format!("{h} ({d})")),
        (Some(h), None) => Some(h),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    }
}

/// The provider that refused and its own message, from the metadata of a reply OpenRouter passes
/// on. Only these two fields are read.
fn provider_detail(metadata: &Value) -> Option<String> {
    let provider = text_field(metadata, "provider_name");
    let raw = match metadata.get("raw") {
        Some(Value::String(s)) => Some(raw_message(s)),
        Some(inner @ Value::Object(_)) => text_of_error(inner),
        _ => None,
    }
    .filter(|s| !s.trim().is_empty());
    match (provider, raw) {
        (Some(p), Some(r)) => Some(format!("{p}: {r}")),
        (Some(p), None) => Some(p.to_owned()),
        (None, Some(r)) => Some(r),
        (None, None) => None,
    }
}

/// The message of a provider's own error object (`error.message`, or `message`).
fn text_of_error(value: &Value) -> Option<String> {
    let inner = value.get("error").unwrap_or(value);
    if let Some(plain) = inner.as_str() {
        return Some(plain.to_owned());
    }
    text_field(inner, "message").map(str::to_owned)
}

/// The message in a raw provider reply: its JSON error message when it is JSON, else the text.
fn raw_message(raw: &str) -> String {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| text_of_error(&v))
        .unwrap_or_else(|| raw.to_owned())
}

/// One line: every run of white space or control characters is one space.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if c.is_whitespace() || c.is_control() {
            space = true;
        } else {
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push(c);
        }
    }
    out
}

/// A character that ends a run that may be a web address.
fn ends_address(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '`' | '<' | '>' | '\\')
}

/// A character that ends a sentence or a bracket and is not part of a web address.
fn is_trailing(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}')
}

/// True for a host name: dotted labels of letters, digits and `-`, the last one letters only
/// (`openrouter.ai`). A model name such as `anthropic/claude-3.5-sonnet` and a number such as
/// `1.5/2` are not.
fn looks_like_host(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        && labels
            .last()
            .is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
}

/// The part of a run after its scheme: the host, and `/[hidden]` for whatever comes after it.
/// A sign-in part before an `@` goes too, and the sentence mark at the end stays.
fn host_only(after_scheme: &str) -> String {
    let core = after_scheme.trim_end_matches(is_trailing);
    let rest = after_scheme.get(core.len()..).unwrap_or("");
    let (authority, has_path) = match core.find(['/', '?', '#']) {
        Some(at) => (core.get(..at).unwrap_or(core), true),
        None => (core, false),
    };
    let host = authority.rsplit('@').next().unwrap_or(authority);
    if has_path {
        format!("{host}/{HIDDEN}{rest}")
    } else {
        format!("{host}{rest}")
    }
}

/// One run of text with no space or quote in it, with its web address cut to the host. The
/// address may follow other text (`see:https://...`), have a scheme or be a bare `host.tld/path`.
fn address_run(run: &str) -> String {
    if let Some((before, after)) = run.split_once("://") {
        let scheme = before
            .chars()
            .rev()
            .take_while(char::is_ascii_alphabetic)
            .count();
        let keep = before.chars().count().saturating_sub(scheme);
        let prefix: String = before.chars().take(keep).collect();
        return format!("{prefix}{}", host_only(after));
    }
    let open = run
        .chars()
        .take_while(|c| matches!(c, '(' | '[' | '{'))
        .count();
    let lead: String = run.chars().take(open).collect();
    let word: String = run.chars().skip(open).collect();
    let core = word.trim_end_matches(is_trailing);
    let rest = word.get(core.len()..).unwrap_or("");
    if let Some((host, path)) = core.split_once('/')
        && !path.is_empty()
        && looks_like_host(host)
    {
        return format!("{lead}{host}/{HIDDEN}{rest}");
    }
    run.to_owned()
}

/// The text with every web address cut to its host (`https://openrouter.ai/keys/k_1` becomes
/// `openrouter.ai/[hidden]`).
fn hide_url_paths(text: &str) -> String {
    let text = text.replace("\\/", "/");
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    for c in text.chars() {
        if ends_address(c) {
            out.push_str(&address_run(&run));
            run.clear();
            out.push(c);
        } else {
            run.push(c);
        }
    }
    out.push_str(&address_run(&run));
    out
}

/// A character that ends a word.
fn is_break(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '"' | '\''
                | '`'
                | ','
                | ';'
                | ':'
                | '='
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '\\'
        )
}

/// True for a word that is a key by its look.
fn looks_like_key(word: &str) -> bool {
    let sk = word.len() >= MIN_PREFIXED_KEY_LEN && word.to_ascii_lowercase().starts_with("sk-");
    let long = word.len() >= KEY_LIKE_LEN
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    let token = word.len() >= 20 && word.starts_with(TOKEN_START);
    sk || long || token
}

/// True for a word that is a value after a header name or `Bearer`.
fn looks_like_value(word: &str) -> bool {
    word.len() >= MIN_VALUE_LEN || word.chars().any(|c| c.is_ascii_digit())
}

/// What the next word is taken for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Next {
    /// A word like any other.
    Plain,
    /// The value after `Authorization` or an API key header name: hidden when it looks like one.
    Value,
}

/// Writes one word, hidden when it is a key or the value of a header name.
fn put_word(word: &str, out: &mut String, next: &mut Next) {
    if word.is_empty() {
        return;
    }
    let lower = word.to_ascii_lowercase();
    let scheme = lower == "bearer" || lower == "basic";
    if *next == Next::Value && scheme {
        // `Authorization: Bearer <value>`: the scheme stays, the value after it goes.
        out.push_str(word);
        return;
    }
    if *next == Next::Value && looks_like_value(word) {
        out.push_str(HIDDEN);
        *next = Next::Plain;
        return;
    }
    *next = Next::Plain;
    if looks_like_key(word) {
        out.push_str(HIDDEN);
        return;
    }
    out.push_str(word);
    if NAME_WORDS.contains(&lower.as_str()) || lower == "bearer" {
        *next = Next::Value;
    }
}

/// The text with every key, `Authorization` value and `Bearer` value hidden.
fn scrub(text: &str, key: &str) -> String {
    let text = if key.is_empty() {
        text.to_owned()
    } else {
        text.replace(key, HIDDEN)
    };
    let mut out = String::with_capacity(text.len());
    let mut next = Next::Plain;
    let mut word = String::new();
    for c in text.chars() {
        if is_break(c) {
            put_word(&word, &mut out, &mut next);
            word.clear();
            out.push(c);
        } else {
            word.push(c);
        }
    }
    put_word(&word, &mut out, &mut next);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reason(body: &str) -> Option<String> {
        reason_of(body.as_bytes(), "sk-or-v1-thekeyofthiscall")
    }

    #[test]
    fn the_code_and_the_message_of_the_error_are_kept() {
        assert_eq!(
            reason(r#"{"error":{"code":403,"message":"Key limit exceeded"}}"#).as_deref(),
            Some("403: Key limit exceeded")
        );
        assert_eq!(
            reason(r#"{"error":{"code":"forbidden","message":"No access to this model"}}"#)
                .as_deref(),
            Some("forbidden: No access to this model")
        );
        // Only one of the two.
        assert_eq!(
            reason(r#"{"error":{"message":"Blocked"}}"#).as_deref(),
            Some("Blocked")
        );
        assert_eq!(reason(r#"{"error":{"code":403}}"#).as_deref(), Some("403"));
        assert_eq!(
            reason(r#"{"error":"Forbidden"}"#).as_deref(),
            Some("Forbidden")
        );
    }

    #[test]
    fn a_reply_passed_on_from_another_provider_names_it_and_keeps_its_message_only() {
        let body = r#"{"error":{"message":"Provider returned error","code":403,
            "metadata":{"raw":"{\"error\":{\"message\":\"Region not allowed\"}}",
            "provider_name":"Azure","flagged_input":"the owner's private text"}}}"#;
        let found = reason(body).unwrap();
        assert_eq!(
            found,
            "403: Provider returned error (Azure: Region not allowed)"
        );
        // The rest of the metadata is never read.
        assert!(!found.contains("private"), "{found}");
    }

    #[test]
    fn a_body_that_is_not_the_api_error_gives_its_first_300_characters_on_one_line() {
        let html = "<html>\n  <body>\n    <h1>403 Forbidden</h1>\n  </body>\n</html>";
        assert_eq!(
            reason(html).as_deref(),
            Some("<html> <body> <h1>403 Forbidden</h1> </body> </html>")
        );
        let long = "word ".repeat(200);
        let found = reason(&long).unwrap();
        assert!(found.chars().count() <= MAX_REASON_CHARS, "{}", found.len());
        assert!(found.starts_with("word word"));
        // JSON with no error object is plain text too.
        assert_eq!(
            reason(r#"{"detail":"Denied"}"#).as_deref(),
            Some(r#"{"detail":"Denied"}"#)
        );
        // An empty body, or only white space, has no reason.
        assert_eq!(reason(""), None);
        assert_eq!(reason(" \n\t "), None);
        assert_eq!(
            reason(r#"{"error":{}}"#).as_deref(),
            Some(r#"{"error":{}}"#)
        );
    }

    #[test]
    fn the_key_of_the_call_and_anything_that_looks_like_one_never_stays() {
        let key = "sk-or-v1-thekeyofthiscall";
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let body = format!(
            r#"{{"error":{{"code":403,"message":"Bad key {key}. Sent: Authorization: Bearer abc123def456 and {hex} and sk-or-v1-other1234 and x-api-key: secretvalue99"}}}}"#
        );
        let found = reason_of(body.as_bytes(), key).unwrap();
        for secret in [
            key,
            hex,
            "abc123def456",
            "sk-or-v1-other1234",
            "secretvalue99",
        ] {
            assert!(!found.contains(secret), "{secret} stayed in {found}");
        }
        assert!(found.contains("Authorization: Bearer [hidden]"), "{found}");
        assert!(found.starts_with("403: Bad key [hidden]."), "{found}");
        // A header as JSON, with escaped quotes.
        let json = r#"{"error":"sent {\"Authorization\":\"Bearer zzzzzzzzzzzz\"}"}"#;
        let found = reason(json).unwrap();
        assert!(!found.contains("zzzzzzzzzzzz"), "{found}");
        // A token that starts like a JSON header.
        let jwt = "eyJhbGciOiJIUzI1NiJ9";
        assert!(!reason(&format!("token {jwt}")).unwrap().contains(jwt));
    }

    #[test]
    fn plain_words_and_model_names_stay_readable() {
        let found = reason(
            r#"{"error":{"code":404,"message":"No endpoints found for anthropic/claude-3.5-sonnet. Authorization failed for claude-3-5-sonnet-20240620 with the bearer token."}}"#,
        )
        .unwrap();
        assert!(found.contains("anthropic/claude-3.5-sonnet"), "{found}");
        assert!(found.contains("Authorization failed"), "{found}");
        assert!(found.contains("claude-3-5-sonnet-20240620"), "{found}");
        assert!(found.contains("bearer token"), "{found}");
    }

    #[test]
    fn a_key_at_the_cut_is_hidden_before_the_cut_and_not_cut_into_a_piece() {
        // The key starts at character 280 and would be cut at 300 if the cut came first.
        let key = "sk-or-v1-zzabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef";
        let body = format!("{}{key} tail", "ab ".repeat(93));
        let found = reason(&body).unwrap();
        assert!(!found.contains("sk-or"), "{found}");
        assert!(found.chars().count() <= MAX_REASON_CHARS);
    }

    #[test]
    fn a_web_address_is_cut_to_its_host_so_the_id_of_the_key_never_stays() {
        // The reply of the live probe: the key's own page, with its id, in the message.
        let found = reason(
            r#"{"error":{"code":403,"message":"Key limit exceeded (total limit). Manage it using the openrouter.ai workspace keys page for this key: https://openrouter.ai/workspaces/default/keys/k_ab12CD34. $1.01 of $1.00 used; does not reset."}}"#,
        )
        .unwrap();
        assert!(
            found.contains("for this key: openrouter.ai/[hidden]. $1.01 of $1.00"),
            "{found}"
        );
        assert!(
            found.contains("the openrouter.ai workspace keys page"),
            "{found}"
        );
        for id in ["workspaces", "default", "k_ab12CD34", "https"] {
            assert!(!found.contains(id), "{id} stayed in {found}");
        }
        for (said, kept) in [
            // The query and the fragment go with the path.
            (
                "see https://openrouter.ai/keys?id=k_77#top now",
                "see openrouter.ai/[hidden] now",
            ),
            // A sign-in part and a port.
            (
                "at https://user:pass@host.example:8443/v1/x",
                "at host.example:8443/[hidden]",
            ),
            (
                "at http://localhost:8080/v1/x.",
                "at localhost:8080/[hidden].",
            ),
            // Brackets, a sentence mark, text stuck to the address.
            ("(see https://a.io/x/y)", "(see a.io/[hidden])"),
            ("see:https://a.io/x/y, then", "see:a.io/[hidden], then"),
            // With no scheme.
            (
                "manage openrouter.ai/settings/keys/k_5 today",
                "manage openrouter.ai/[hidden] today",
            ),
            ("(openrouter.ai/keys/k_5)", "(openrouter.ai/[hidden])"),
            // JSON text with escaped slashes.
            (
                r"https:\/\/openrouter.ai\/keys\/k_9",
                "openrouter.ai/[hidden]",
            ),
            // A host alone has no id in it.
            ("go to https://openrouter.ai.", "go to openrouter.ai."),
            ("go to openrouter.ai.", "go to openrouter.ai."),
        ] {
            assert_eq!(reason(said).as_deref(), Some(kept), "{said}");
        }
    }

    #[test]
    fn model_names_numbers_and_paths_that_are_not_addresses_stay() {
        for text in [
            "No endpoints found for anthropic/claude-3.5-sonnet.",
            "mistralai/mistral-7b-instruct-v0.1 is busy",
            "ratio 1.5/2 and and/or e.g. a/b",
            "</h1> and v1.2/x",
        ] {
            assert_eq!(reason(text).as_deref(), Some(text), "{text}");
        }
    }

    #[test]
    fn bytes_that_are_not_text_are_read_as_far_as_they_go() {
        let found = reason_of(&[0xff, 0xfe, b'o', b'k'], "secret-key-1").unwrap();
        assert!(found.ends_with("ok"), "{found}");
    }

    proptest! {
        #[test]
        fn the_key_never_stays_and_the_reason_is_never_long(
            key in "[A-Za-z0-9_-]{8,24}",
            before in "\\PC{0,200}",
            after in "\\PC{0,200}",
        ) {
            let body = format!("{before}{key}{after}");
            if let Some(found) = reason_of(body.as_bytes(), &key) {
                prop_assert!(found.chars().count() <= MAX_REASON_CHARS);
                prop_assert!(!found.contains(&key), "{found}");
            }
        }
    }
}
