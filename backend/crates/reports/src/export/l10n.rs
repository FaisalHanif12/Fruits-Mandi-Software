//! The words of the sheets (Tech Stack 4.2, NFR-URD-1): the same Fluent messages as the screens,
//! read from the `.ftl` files that `build.rs` embeds from `frontend/src/i18n/`. No sentence is
//! written in Rust. A message that is missing gives its own ID, so a gap is visible in the sheet
//! (and the tests of the catalogue fail on it).
//!
//! Example (English): `text("report-side-dr")` gives `Debit`; (Urdu) it gives `بنام`.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use mandi_core::auth::Language;
use unic_langid::LanguageIdentifier;

use crate::error::ReportsError;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/ftl_files.rs"));
}

/// The Fluent messages of one language.
pub struct L10n {
    bundle: FluentBundle<FluentResource>,
}

impl L10n {
    /// Reads every message file of the language into one bundle.
    ///
    /// # Errors
    /// `ReportsError::Data` if the language tag cannot be made (a bug).
    pub fn new(language: Language) -> Result<Self, ReportsError> {
        let (tag, files) = match language {
            Language::Ur => ("ur", embedded::UR),
            Language::En => ("en", embedded::EN),
        };
        let id: LanguageIdentifier = tag
            .parse()
            .map_err(|_| ReportsError::Data(format!("language tag {tag}")))?;
        let mut bundle = FluentBundle::new(vec![id]);
        // The isolation marks are for text on screen; a cell holds the plain text.
        bundle.set_use_isolating(false);
        for (_name, source) in files {
            // A file with a bad line still gives every good message in it.
            let resource = match FluentResource::try_new((*source).to_owned()) {
                Ok(resource) | Err((resource, _)) => resource,
            };
            bundle.add_resource_overriding(resource);
        }
        Ok(Self { bundle })
    }

    /// True if the message exists.
    pub fn has(&self, id: &str) -> bool {
        self.bundle
            .get_message(id)
            .is_some_and(|message| message.value().is_some())
    }

    /// The message with no arguments.
    pub fn text(&self, id: &str) -> String {
        self.text_with(id, &[])
    }

    /// The message with its arguments (all text, already formatted by the caller).
    pub fn text_with(&self, id: &str, args: &[(&str, &str)]) -> String {
        let Some(pattern) = self
            .bundle
            .get_message(id)
            .and_then(|message| message.value())
        else {
            return id.to_owned();
        };
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, FluentValue::from((*value).to_owned()));
        }
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_message_reads_in_both_languages() {
        let en = L10n::new(Language::En).unwrap();
        let ur = L10n::new(Language::Ur).unwrap();
        assert_eq!(en.text("report-side-dr"), "Debit");
        assert_eq!(ur.text("report-side-dr"), "بنام");
        assert!(en.has("report-r2-title") && ur.has("report-r2-title"));
    }

    #[test]
    fn a_missing_message_gives_its_own_id() {
        let en = L10n::new(Language::En).unwrap();
        assert!(!en.has("no-such-message"));
        assert_eq!(en.text("no-such-message"), "no-such-message");
    }

    #[test]
    fn arguments_are_filled_and_terms_are_resolved() {
        let en = L10n::new(Language::En).unwrap();
        let line = en.text_with("report-detail-cash-lent", &[("reason", "Medicine")]);
        assert!(line.ends_with(": Medicine"), "{line}");
        assert!(!line.contains("{"), "{line}");
    }
}
