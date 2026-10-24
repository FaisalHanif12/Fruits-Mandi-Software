//! The all-parties table of R35: sections of buyers and sellers with their totals.

use super::activity::*;
use super::*;

/// The columns of the table of all parties.
pub(super) fn table_columns() -> Vec<crate::reports::model::Column> {
    vec![
        col("code", ids::COL_CODE, ColumnKind::Text),
        col("party", ids::COL_ACCOUNT, ColumnKind::Party),
        col("previous", ids::COL_PREVIOUS, ColumnKind::Balance),
        col("debit", ids::COL_DEBIT, ColumnKind::Money),
        col("credit", ids::COL_CREDIT, ColumnKind::Money),
        col("remaining", ids::COL_BALANCE, ColumnKind::Balance),
        col("collector", ids::COL_COLLECTOR, ColumnKind::Text),
    ]
}

/// The sums of a section (or of the whole table).
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Sums {
    pub(super) prev_net: Paisa,
    pub(super) day_dr: Paisa,
    pub(super) day_cr: Paisa,
    pub(super) end_net: Paisa,
}

impl Sums {
    pub(super) fn add(&mut self, item: &Activity) -> Result<(), ReportsError> {
        self.prev_net = self.prev_net.checked_add(item.prev_net()?)?;
        add(&mut self.day_dr, item.day_dr)?;
        add(&mut self.day_cr, item.day_cr)?;
        self.end_net = self.end_net.checked_add(item.end_net()?)?;
        Ok(())
    }

    pub(super) fn cells(&self, lead: Cell) -> Result<Vec<Cell>, ReportsError> {
        Ok(vec![
            lead,
            Cell::Empty,
            net_cell(self.prev_net)?,
            Cell::Money { paisa: self.day_dr },
            Cell::Money { paisa: self.day_cr },
            net_cell(self.end_net)?,
            Cell::Empty,
        ])
    }
}

/// One party row of the table.
pub(super) fn party_row(item: &Activity, party: &PartyRef) -> Result<Row, ReportsError> {
    let code = party
        .short_code
        .clone()
        .or_else(|| Some(party.code.clone()));
    Ok(Row {
        cells: vec![
            text_cell(code),
            Cell::Party {
                party: party.clone(),
            },
            net_cell(item.prev_net()?)?,
            Cell::Money { paisa: item.day_dr },
            Cell::Money { paisa: item.day_cr },
            net_cell(item.end_net()?)?,
            Cell::Empty,
        ],
        source: Some(RowSource {
            kind: SourceKind::Account,
            id: item.account_id,
        }),
        kind: RowKind::Normal,
    })
}

/// The all-parties layout.
pub(super) fn table(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    day: &str,
) -> Result<ReportPage, ReportsError> {
    let mut items = Vec::new();
    for item in activities(conn, day, None, None)? {
        if item.worth_a_row()? {
            items.push(item);
        }
    }
    let parties = parties_of(conn, &items)?;
    // Urdu alphabetical order of the names; on a tie the account and then the person.
    let collator = UrduCollator::new().ok();
    let sort_key = |name: &str| -> Vec<u8> {
        collator
            .as_ref()
            .map_or_else(|| name.as_bytes().to_vec(), |c| c.sort_key(name))
    };
    let mut keyed: Vec<(Vec<u8>, Activity)> = items
        .into_iter()
        .map(|item| {
            let name = parties
                .get(&(item.account_id, item.person_key.clone()))
                .map(|p| p.name_ur.clone())
                .unwrap_or_default();
            (sort_key(&name), item)
        })
        .collect();
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.account_id.cmp(&b.1.account_id))
            .then(a.1.person_key.cmp(&b.1.person_key))
    });

    let mut rows = Vec::new();
    let mut grand = Sums::default();
    // The shop's total اگراہی (PRD FR-19.9): what the buyers owe, a buyer who is in credit not
    // taken off. R20 lists the same amounts, so this line always equals its میزان.
    let mut collection = Paisa::ZERO;
    // The sellers' table has its own total (what the shop owes them), not added to the buyers'.
    let mut sellers_total: Option<Sums> = None;
    for (buyers, label) in [(true, ids::R35_BUYERS), (false, ids::R35_SELLERS)] {
        let section: Vec<&Activity> = keyed
            .iter()
            .map(|(_, a)| a)
            .filter(|a| a.buyer == buyers)
            .collect();
        if section.is_empty() {
            continue;
        }
        let mut sums = Sums::default();
        let mut section_rows = Vec::with_capacity(section.len());
        for item in &section {
            sums.add(item)?;
            if buyers && item.end_net()? > Paisa::ZERO {
                add(&mut collection, item.end_net()?)?;
            }
            let party = parties
                .get(&(item.account_id, item.person_key.clone()))
                .ok_or_else(|| ReportsError::Data("a party of the sheet is missing".to_owned()))?;
            section_rows.push(party_row(item, party)?);
        }
        // The totals row is the buyers' table: what is owed to the shop. The sellers' total is
        // what the shop owes; the two are not added together (PRD E2E-43, money agent S1).
        if buyers {
            grand = sums;
        } else {
            sellers_total = Some(sums);
        }
        rows.push(Row {
            cells: sums.cells(Cell::Label {
                id: label.to_owned(),
            })?,
            source: None,
            kind: RowKind::Group,
        });
        rows.extend(section_rows);
    }

    let mut summary = vec![line(ids::R35_COLLECTION, Cell::Money { paisa: collection })];
    if let Some(sellers) = sellers_total {
        summary.push(line(
            ids::R35_SELLERS_CREDIT,
            Cell::Money {
                paisa: sellers.day_cr,
            },
        ));
        summary.push(line(
            ids::R35_SELLERS_DEBIT,
            Cell::Money {
                paisa: sellers.day_dr,
            },
        ));
        summary.push(line(ids::R35_SELLERS_REMAINING, net_cell(sellers.end_net)?));
    }

    let mut resolved = typed.resolved.clone();
    resolved.as_of = Some(day.to_owned());
    Ok(ReportPage {
        id: ReportId::R35,
        title_id: ids::title_id("R35"),
        filters: resolved,
        columns: table_columns(),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: grand.cells(Cell::Empty)?,
        summary,
    })
}
