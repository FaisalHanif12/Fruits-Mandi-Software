//! The instant sentences of the figures that a report or a view holds as a total (D7, plan 59 F3): the
//! quantity of a sale, an item's average rate, the shop's expenses, receipts, the cash book's closing, the
//! sales of one payment type, a seller's net sale, the value of fruit taken and capital in or out.
//!
//! Each sentence's figure is a REFERENCE to one cell of a result read for this question; nothing is added,
//! subtracted or rounded here (Code Standards Q5). A result that holds partner figures while the partner
//! section is locked gives no sentence.

use super::{Cx, Sentence, What, Who, count_of, input_str, reference, row_or_total, whole};
use crate::results::{Node, ToolResult, Val};

impl Cx<'_> {
    /// The result of a read of a report, when it exists and holds no hidden partner part.
    fn open_report(&self, id: &str) -> Option<&ToolResult> {
        let r = self.report(id)?;
        (!r.partner_parts_locked).then_some(r)
    }

    /// How many { نگ } were sold or bought: the `nag` column of view A (the shop, an item, a seller's or a
    /// buyer's item), B (a seller) or C (a buyer), read for the period the question named.
    pub(super) fn qty(&self, who: &Who) -> Option<Sentence> {
        // A record's figure needs the period the question named (see `flow`).
        if self.u.periods.is_empty() {
            return None;
        }
        let mut args: Vec<(&'static str, String)> = Vec::new();
        let (role, figure) = match who {
            Who::Shop => {
                let view = self.analytics("A", "", None)?;
                ("shop", reference(view, &["totals", "nag"])?)
            }
            Who::Item(key) => {
                args.push(("name", self.name_of(key)?));
                (
                    "item",
                    row_or_total(self.analytics("A", "item", Some(key))?, key, "nag")?,
                )
            }
            Who::Seller(key) => {
                args.push(("name", self.name_of(key)?));
                (
                    "seller",
                    row_or_total(self.analytics("B", "seller", Some(key))?, key, "nag")?,
                )
            }
            Who::Buyer(key) => {
                args.push(("name", self.name_of(key)?));
                (
                    "buyer",
                    row_or_total(self.analytics("C", "buyer", Some(key))?, key, "nag")?,
                )
            }
            Who::SellerItem(party, item) | Who::BuyerItem(party, item) => {
                let field = if matches!(who, Who::SellerItem(..)) {
                    "seller"
                } else {
                    "buyer"
                };
                let view = self.result("analytics", |r| {
                    input_str(r, "view") == Some("A")
                        && input_str(r, field) == Some(party)
                        && input_str(r, "item") == Some(item)
                })?;
                args.push(("name", self.name_of(party)?));
                args.push(("item", self.name_of(item)?));
                (
                    if field == "seller" {
                        "seller-item"
                    } else {
                        "buyer-item"
                    },
                    row_or_total(view, item, "nag")?,
                )
            }
            _ => return None,
        };
        let mut all = vec![("role", role.to_owned())];
        all.extend(args);
        all.push(("figure", figure));
        Some(Sentence {
            id: "assistant-instant-qty",
            args: all,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// A figure of a whole group of items over the period: the totals of view A read for the group
    /// (`gross`, `total_comm` with Laga, `laga`, `nag`).
    pub(super) fn group(&self, group: &str, what: What) -> Option<Sentence> {
        if self.u.periods.is_empty() {
            return None;
        }
        let (kind, column) = match what {
            What::Sale => ("sale", "gross"),
            What::Commission => ("commission", "total_comm"),
            What::Laga => ("laga", "laga"),
            What::Qty => ("qty", "nag"),
            _ => return None,
        };
        let view = self.analytics("A", "item_group", Some(group))?;
        Some(Sentence {
            id: "assistant-instant-group",
            args: vec![
                ("kind", kind.to_owned()),
                ("group", group.to_owned()),
                ("figure", reference(view, &["totals", column])?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// An item's average rate (the `avg_rate` of its row of view A), or a seller's average rate of one
    /// item, named as the average. It is the seller's (auction) rate: a question about the buyer's rate is
    /// the model's. An item with no sale in the period has no row and no sentence.
    pub(super) fn avg_rate(&self, who: &Who) -> Option<Sentence> {
        if self.u.periods.is_empty() || self.u.buyer_rate {
            return None;
        }
        let (view, role, item, seller) = match who {
            Who::Item(item) => (
                self.analytics("A", "item", Some(item))?,
                "item",
                item.as_str(),
                None,
            ),
            Who::SellerItem(seller, item) => (
                self.result("analytics", |r| {
                    input_str(r, "view") == Some("A")
                        && input_str(r, "seller") == Some(seller)
                        && input_str(r, "item") == Some(item)
                })?,
                "seller-item",
                item.as_str(),
                Some(seller.as_str()),
            ),
            _ => return None,
        };
        // Only a real row holds an average: the totals of an empty view have none.
        if count_of(view, &["count"])? != 1 {
            return None;
        }
        let mut args = vec![("role", role.to_owned())];
        if let Some(seller) = seller {
            args.push(("name", self.name_of(seller)?));
        }
        args.push(("item", self.name_of(item)?));
        args.push(("figure", reference(view, &["rows", item, "avg_rate"])?));
        Some(Sentence {
            id: "assistant-instant-rate-average",
            args,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The shop's expenses over the period (`totals.expense` of R29), or those of one head: its row of R29
    /// (a shop head row has an `expense`, a name and no section). A head with no row has no expense in the
    /// period, which the model says.
    pub(super) fn expense(&self, who: &Who) -> Option<Sentence> {
        // R29 hides a partner's own rows while the partner section is locked; the shop's `expense` column and
        // the heads are not partner parts, so a locked section does not stop this answer.
        let r29 = self.report("R29")?;
        let mut args: Vec<(&'static str, String)> = Vec::new();
        let (kind, figure) = match who {
            Who::Shop => ("shop", reference(r29, &["totals", "expense"])?),
            Who::Head(name) => {
                let Some(Node::Keyed(rows)) = r29.node.get(&["rows"]) else {
                    return None;
                };
                let mut found = None;
                for (key, row) in rows {
                    let named = matches!(
                        row.get(&["name"]),
                        Some(Node::Val(Val::Name(n))) if n.name_en == *name
                    );
                    let shop_row = matches!(row.get(&["expense"]), Some(Node::Val(Val::Money(_))))
                        && row
                            .get(&["section"])
                            .is_none_or(|s| matches!(s, Node::Null));
                    if named && shop_row {
                        // Two rows of one name are not told apart.
                        if found.is_some() {
                            return None;
                        }
                        found = Some(key.as_str());
                    }
                }
                let key = found?;
                args.push(("name", reference(r29, &["rows", key, "name"])?));
                ("head", reference(r29, &["rows", key, "expense"])?)
            }
            _ => return None,
        };
        let mut all = vec![("kind", kind.to_owned())];
        all.extend(args);
        all.push(("figure", figure));
        Some(Sentence {
            id: "assistant-instant-expense",
            args: all,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// Money received over the period: the total of the receipts register (R23), the whole shop's or one
    /// collector's (the read was made for him).
    pub(super) fn receipts(&self, who: &Who) -> Option<Sentence> {
        let r23 = self.open_report("R23")?;
        let mut args = Vec::new();
        let role = match who {
            Who::Shop => {
                // A read for a collector is not the shop's.
                if input_str(r23, "collector").is_some() {
                    return None;
                }
                "shop"
            }
            Who::Collector(key) => {
                if input_str(r23, "collector") != Some(key.as_str()) {
                    return None;
                }
                args.push(("name", self.name_of(key)?));
                "collector"
            }
            _ => return None,
        };
        let mut all = vec![("role", role.to_owned())];
        all.extend(args);
        all.push(("figure", reference(r23, &["totals", "amount"])?));
        Some(Sentence {
            id: "assistant-instant-receipts",
            args: all,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The cash book's closing balance at the end of the period (`summary.closing` of R12).
    pub(super) fn cash_closing(&self) -> Option<Sentence> {
        let r12 = self.open_report("R12")?;
        Some(Sentence {
            id: "assistant-instant-cash-book",
            args: vec![
                ("kind", "closing".to_owned()),
                ("figure", reference(r12, &["summary", "closing"])?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The cash received or paid out over the period: the `cash_in` or `cash_out` total of the cash book
    /// (R12), worded as the cash book's own sentence.
    pub(super) fn cash_flow(&self, what: What) -> Option<Sentence> {
        let (kind, column) = match what {
            What::CashIn => ("in", "cash_in"),
            What::CashOut => ("out", "cash_out"),
            _ => return None,
        };
        let r12 = self.open_report("R12")?;
        // The totals hold a cancelled or edited entry twice (its posting and its reversal): with one in the
        // period, or with rows the app did not read, the figure is not what came in or went out.
        if cash_book_has_reversal_or_unread_rows(r12) {
            return None;
        }
        Some(Sentence {
            id: "assistant-instant-cash-book",
            args: vec![
                ("kind", kind.to_owned()),
                ("figure", reference(r12, &["totals", column])?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// What the buyers of one city owe: the `balance` of the city's own row of R21, the row that names no buyer.
    pub(super) fn city_dues(&self, city: &str) -> Option<Sentence> {
        let r21 = self.result("report", |r| {
            input_str(r, "id") == Some("R21") && input_str(r, "city") == Some(city)
        })?;
        let Some(Node::Keyed(rows)) = r21.node.get(&["rows"]) else {
            return None;
        };
        // The city's row is the one with no buyer; there is exactly one.
        let mut group = rows.iter().filter(|(key, _)| {
            matches!(
                r21.node.get(&["rows", key, "buyer"]),
                None | Some(Node::Null)
            )
        });
        let (key, _) = group.next()?;
        if group.next().is_some() {
            return None;
        }
        Some(Sentence {
            id: "assistant-instant-city-dues",
            args: vec![
                ("name", self.name_of(city)?),
                ("figure", reference(r21, &["rows", key, "balance"])?),
            ],
            period: None,
        })
    }

    /// The total debit or credit of the trial balance (R18), as of now.
    pub(super) fn trial_total(&self, what: What) -> Option<Sentence> {
        let (kind, column) = match what {
            What::TrialDebit => ("debit", "debit"),
            What::TrialCredit => ("credit", "credit"),
            _ => return None,
        };
        let r18 = self.open_report("R18")?;
        Some(Sentence {
            id: "assistant-instant-trial-total",
            args: vec![
                ("kind", kind.to_owned()),
                ("figure", reference(r18, &["totals", column])?),
            ],
            period: None,
        })
    }

    /// The sales of one payment type over the period: the total `value` of the cash sales (R3) or of the
    /// credit sales (R4).
    pub(super) fn sale_kind(&self, what: What) -> Option<Sentence> {
        let (id, kind) = match what {
            What::CashSales => ("R3", "cash"),
            What::CreditSales => ("R4", "credit"),
            _ => return None,
        };
        let register = self.open_report(id)?;
        // A line sold at a buyer rate that is not the seller's makes the register carry the seller rate: the
        // `value` total is the buyer's goods value then, which is not what "sales" are in the tile (خام بکری),
        // so the model words it (money review of F3a).
        if matches!(
            register.node.get(&["columns"]),
            Some(Node::List(columns))
                if columns.iter().any(|c| matches!(c, Node::Str(s) if s == "seller_rate"))
        ) {
            return None;
        }
        Some(Sentence {
            id: "assistant-instant-sale-kind",
            args: vec![
                ("kind", kind.to_owned()),
                ("figure", reference(register, &["totals", "value"])?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// A seller's net sale: the `net` column of his row of view B.
    pub(super) fn sale_net(&self, key: &str) -> Option<Sentence> {
        if self.u.periods.is_empty() {
            return None;
        }
        let view = self.analytics("B", "seller", Some(key))?;
        // A negative net is the seller owing the shop (PRD FR-12.7): the model words it.
        if whole(view, &["rows", key, "net"]).is_some_and(|n| n < 0) {
            return None;
        }
        Some(Sentence {
            id: "assistant-instant-sale-net",
            args: vec![
                ("name", self.name_of(key)?),
                ("figure", row_or_total(view, key, "net")?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The value of the fruit taken: the total `value` of R37 for everyone, for the guests alone, or for
    /// one partner.
    pub(super) fn fruit(&self, who: &Who) -> Option<Sentence> {
        let r37 = self.open_report("R37")?;
        let mut args = Vec::new();
        let role = match who {
            Who::Shop => {
                if input_str(r37, "partner").is_some() {
                    return None;
                }
                "shop"
            }
            Who::Guests => {
                if input_str(r37, "partner") != Some("guests") {
                    return None;
                }
                "guests"
            }
            Who::Partner(Some(key)) => {
                if input_str(r37, "partner") != Some(key.as_str()) {
                    return None;
                }
                args.push(("name", self.name_of(key)?));
                "partner"
            }
            _ => return None,
        };
        let mut all = vec![("role", role.to_owned())];
        all.extend(args);
        all.push(("figure", reference(r37, &["totals", "value"])?));
        Some(Sentence {
            id: "assistant-instant-fruit",
            args: all,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// Capital put into the shop or taken back over the period: the total of the capital vouchers (R15
    /// with that voucher type).
    pub(super) fn capital_flow(&self, what: What) -> Option<Sentence> {
        let (voucher_type, kind) = match what {
            What::CapitalIn => ("capital_in", "in"),
            What::CapitalOut => ("capital_out", "out"),
            _ => return None,
        };
        let r15 = self.open_report("R15")?;
        if input_str(r15, "voucher_type") != Some(voucher_type) {
            return None;
        }
        Some(Sentence {
            id: "assistant-instant-capital-flow",
            args: vec![
                ("kind", kind.to_owned()),
                ("figure", reference(r15, &["totals", "amount"])?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }
}

/// True when the cash book read has a reversal entry among its rows, or has more entries than rows read, or
/// its rows or count are missing: the cash in and out totals then hold an entry twice or cannot be vouched
/// for.
///
/// Example: after بکری 2 is cancelled the same day, its reversal is a row of the day, so this is true.
fn cash_book_has_reversal_or_unread_rows(r12: &ToolResult) -> bool {
    let Some(count) = count_of(r12, &["count"]) else {
        return true;
    };
    if count == 0 {
        return false;
    }
    let Some(Node::Keyed(rows)) = r12.node.get(&["rows"]) else {
        return true;
    };
    if !i64::try_from(rows.len()).is_ok_and(|n| n == count) {
        return true;
    }
    rows.iter().any(|(key, _)| {
        matches!(
            r12.node.get(&["rows", key, "entry"]).and_then(Node::as_val),
            Some(Val::Doc { type_id, .. }) if type_id.ends_with("reversal")
        )
    })
}
