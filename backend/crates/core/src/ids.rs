//! Typed ids, so an account id cannot be passed where an entry id is expected (Code Standards B6).

use serde::{Deserialize, Serialize};

macro_rules! typed_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(i64);

        impl $name {
            /// An id from a stored row id.
            pub const fn new(id: i64) -> Self {
                Self(id)
            }

            /// The row id inside, for storage.
            pub const fn get(self) -> i64 {
                self.0
            }
        }
    };
}

typed_id!(
    /// The id of an account (`accounts.id`).
    AccountId
);
typed_id!(
    /// The id of a journal entry (`journal_entries.id`).
    EntryId
);
typed_id!(
    /// The id of a partner (`partners.id`). A lower id means the partner was created first,
    /// which is the tie rule of the split leftover (PRD C10).
    PartnerId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_keep_their_number_and_stay_apart() {
        assert_eq!(AccountId::new(7).get(), 7);
        assert_eq!(EntryId::new(7).get(), 7);
        assert_eq!(PartnerId::new(7).get(), 7);
        assert!(AccountId::new(1) < AccountId::new(2));
    }
}
