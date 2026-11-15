//! The link of a separate khata or of a person who is lent money, and the type of that person
//! (plan 76 points 5 and 10, PRD FR-16.4, FR-16.6 v1.41).
//!
//! A link says who bears what goes to a khata or to a person: the shop (every movement is split
//! between the active partners by their shares of its own day), one partner (the whole of it) or
//! nobody (the khata or loan is the firm's, exactly as before). The link belongs to the account and
//! changes only while its balance is 0.
//!
//! Example: خاندانی کھاتہ linked to the shop (40 / 30 / 30) is paid out 10,000: الف -4,000, ب -3,000,
//! ج -3,000 in their کل رقم.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What an account is linked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// Not linked: the account is the firm's, as before.
    None,
    /// The shop: each movement is split between the active partners by their shares of that day.
    Shop,
    /// One partner: the whole movement is his.
    Partner,
}

impl LinkKind {
    /// The word stored in the database and sent to the screens.
    pub const fn as_str(self) -> &'static str {
        match self {
            LinkKind::None => "none",
            LinkKind::Shop => "shop",
            LinkKind::Partner => "partner",
        }
    }

    /// Reads the stored word.
    pub fn parse(text: &str) -> Option<LinkKind> {
        match text {
            "none" => Some(LinkKind::None),
            "shop" => Some(LinkKind::Shop),
            "partner" => Some(LinkKind::Partner),
            _ => None,
        }
    }
}

/// A link: what and, for a partner link, which partner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct Link {
    /// None, the shop or one partner.
    pub kind: LinkKind,
    /// The partner of a partner link (`ERR_LINK_PARTNER_REQUIRED` on `link.partner_id` when it is
    /// missing); null for every other kind.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
}

impl Link {
    /// No link.
    pub const NONE: Link = Link {
        kind: LinkKind::None,
        partner_id: None,
    };
}

/// The type of a person who is lent money (PRD FR-16.4 v1.41). A label: it changes no amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PersonType {
    /// A staff member.
    Staff,
    /// The owner of a garden.
    GardenOwner,
    /// The person sent on a market trip.
    TripPerson,
    /// Anyone else.
    Other,
}

impl PersonType {
    /// The word stored in the database and sent to the screens.
    pub const fn as_str(self) -> &'static str {
        match self {
            PersonType::Staff => "staff",
            PersonType::GardenOwner => "garden_owner",
            PersonType::TripPerson => "trip_person",
            PersonType::Other => "other",
        }
    }

    /// Reads the stored word.
    pub fn parse(text: &str) -> Option<PersonType> {
        match text {
            "staff" => Some(PersonType::Staff),
            "garden_owner" => Some(PersonType::GardenOwner),
            "trip_person" => Some(PersonType::TripPerson),
            "other" => Some(PersonType::Other),
            _ => None,
        }
    }
}
