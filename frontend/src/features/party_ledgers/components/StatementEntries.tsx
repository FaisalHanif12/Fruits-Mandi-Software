// A party's statement with one row for each entry (PRD FR-12.47, FR-19.16, plan 78 A7, B1 to B3): date, entry
// ("بکری 1", "ادائیگی 1", with the mark of an edit), one line of items, بنام, جمع, the part of the credit paid on
// the spot (نقد, buyers only) and the running balance. The first row is the balance the account was made with
// (ابتدائی بقایا, its own entry) or, in a window that starts later, the balance brought forward. One بکری is one
// row, with the money paid on the spot in it, and an entry that an edit or a cancel replaced is not on the list
// (it is behind "منسوخ بھی دکھائیں"). A row opens under itself with its items in full (the same lines as the
// party ledger R13), its narration and its edit, and two buttons: that day's slip and the record itself. The
// figures are the backend's; this adds up nothing (CLAUDE.md rule 1). The table is the shared `DayTable`, with
// a key for each row, because one date can hold many rows.

import { Localized, useLocalization } from '@fluent/react';
import { Pencil, Printer } from 'lucide-react';
import { Fragment, useMemo, type ReactNode } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { StatementRows } from '../../../bindings/StatementRows';
import type { StatementRowsRow } from '../../../bindings/StatementRowsRow';
import { isoToDisplayDate } from '../../../shared/format';
import { DayTable, useListContext, type DayColumn } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, Button, Num, RUNNING_NUMBER } from '../../../shared/ui';
import { PartyDaySlip, detailLines, useOpenRowSource } from '../../reports';

/** The key of a row: one date can hold many entries. */
export function entryKey(row: Pick<StatementRowsRow, 'entry_id'>): string {
  return String(row.entry_id);
}

interface StatementEntriesProps {
  /** The party: an opened entry can show the R35 slip of its day for him. */
  accountId: number;
  /** Buyer or seller: a buyer has the column for the money he paid on the spot. */
  kind: LedgerSide;
  rows: readonly StatementRowsRow[];
  /** The backend's figures of the period: the balance brought forward, its sums and its closing balance. */
  totals: Pick<
    StatementRows,
    'brought_forward' | 'dr_total_paisa' | 'cr_total_paisa' | 'on_spot_total_paisa' | 'closing'
  >;
  open: ReadonlySet<string>;
  onToggle: (key: string, open: boolean) => void;
  active: string | null;
  onActive: (key: string) => void;
  /** The keys of the entries whose day slip shows in place; it is the screen's, so it outlives a print preview. */
  slips: ReadonlySet<string>;
  onSlip: (key: string, shown: boolean) => void;
  /** Prints the slip of one day (the R35 slip of this party). */
  onDaySlip: (date: string) => void;
  empty: ReactNode;
  busy: boolean;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
}

/** The mark of an edited entry: the word is always written (NFR-USE-12); pointing at it shows the reason. */
function EditedBadge({ edited }: { edited: NonNullable<StatementRowsRow['edited']> }) {
  const { l10n } = useLocalization();
  const reason = edited.reason ?? '';
  return (
    <span
      className="inline-flex items-center gap-1 rounded-1 bg-warn-soft px-2 type-body text-warn"
      title={reason === '' ? undefined : l10n.getString('party-ledgers-edited-reason', { reason })}
      data-edited-mark=""
    >
      <Pencil aria-hidden="true" className="size-icon-sm" />
      <Localized id="edit-mark" />
    </span>
  );
}

/** One line of items: the item (a lot's name on a seller) and its quantity, each item once. */
function ShortItems({ row }: { row: StatementRowsRow }) {
  const language = useLanguage();
  return (
    <span className="type-body">
      {row.short.map((item, index) => (
        <Fragment key={index}>
          {index === 0 ? null : <Localized id="party-ledgers-list-separator" />}
          <bdi dir="auto">
            {displayName({ name_ur: item.item.ur, name_en: item.item.en }, language)}
          </bdi>{' '}
          <span className="ms-1">
            <Num>{String(item.qty)}</Num>
          </span>
        </Fragment>
      ))}
    </span>
  );
}

/** The entry's own words: its kind and number, and the mark of an edit or of a replaced entry. */
function EntryLabel({ row }: { row: StatementRowsRow }) {
  return (
    <span className="inline-flex flex-wrap items-center justify-center gap-x-2 gap-y-1">
      <Localized id={row.type_id}>
        <span />
      </Localized>
      {row.number === null ? null : <Num>{String(row.number)}</Num>}
      {row.edited === null ? null : <EditedBadge edited={row.edited} />}
      {row.is_replaced ? (
        <Localized id="party-ledgers-replaced">
          <span className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </span>
  );
}

/** The R35 slip of the day of an entry, in place, with its own print preview (PRD FR-12.47, FR-19.12). */
function DaySlipInPlace({
  accountId,
  date,
  onPrint,
}: {
  accountId: number;
  date: string;
  onPrint: (date: string) => void;
}) {
  const { l10n } = useLocalization();
  const shown = isoToDisplayDate(date);
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="party-ledgers-day-heading" vars={{ date: shown }} elems={RUNNING_NUMBER}>
          <h3 className="type-section" />
        </Localized>
        <Button
          accessibleName={l10n.getString('party-ledgers-day-print-name', { date: shown })}
          onClick={() => {
            onPrint(date);
          }}
        >
          <Printer aria-hidden="true" className="size-icon-md" />
          <Localized id="report-preview" />
        </Button>
      </div>
      <PartyDaySlip accountId={accountId} date={date} inPlace />
    </div>
  );
}

/**
 * What an open row shows: the items in full, the narration and the edit, then two buttons: the slip of that
 * day, shown in place, and the record itself.
 */
function EntryDetail({
  accountId,
  row,
  slip,
  onSlip,
  onDaySlip,
}: {
  accountId: number;
  row: StatementRowsRow;
  slip: boolean;
  onSlip: (shown: boolean) => void;
  onDaySlip: (date: string) => void;
}) {
  const { l10n } = useLocalization();
  const context = useListContext();
  const openSource = useOpenRowSource();
  const lines = detailLines(row.parts, context);
  const editReason = row.edited?.reason ?? '';
  return (
    <div className="flex flex-col gap-2">
      {lines.length === 0 ? null : (
        <ul className="flex list-none flex-col gap-1 type-body">
          {lines.map((line, index) => (
            <li key={index}>
              <bdi dir="auto">{line}</bdi>
            </li>
          ))}
        </ul>
      )}
      {row.narration === null || row.narration === '' ? null : (
        <p className="type-body text-ink-muted">
          <bdi dir="auto">{row.narration}</bdi>
        </p>
      )}
      {editReason === '' ? null : (
        <p className="type-body text-ink-muted">
          <bdi dir="auto">
            {l10n.getString('party-ledgers-edited-reason', { reason: editReason })}
          </bdi>
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button
          expanded={slip}
          onClick={() => {
            onSlip(!slip);
          }}
        >
          <Localized id="party-ledgers-day-slip-button" />
        </Button>
        <Button
          onClick={() => {
            openSource(row.source);
          }}
        >
          <Localized id="party-ledgers-open-record" />
        </Button>
      </div>
      {slip ? <DaySlipInPlace accountId={accountId} date={row.date} onPrint={onDaySlip} /> : null}
    </div>
  );
}

export function StatementEntries({
  accountId,
  kind,
  totals,
  slips,
  onSlip,
  onDaySlip,
  ...table
}: StatementEntriesProps) {
  const buyer = kind === 'buyer';
  // The columns that wait for the normal step keep the compact step (1024 px with both menus open) from
  // scrolling sideways (NFR-USE-15, F22): the one line of items and the money paid on the spot.
  const columns = useMemo<readonly DayColumn<StatementRowsRow>[]>(
    () => [
      {
        key: 'date',
        headerId: 'party-ledgers-col-date',
        cell: (row) => <Num>{isoToDisplayDate(row.date)}</Num>,
        footer: (
          <Localized id="glossary-meezan">
            <span />
          </Localized>
        ),
      },
      {
        key: 'entry',
        headerId: 'party-ledgers-col-entry',
        cell: (row) => <EntryLabel row={row} />,
        lead: <Localized id="party-ledgers-brought-forward" />,
      },
      {
        key: 'detail',
        headerId: 'party-ledgers-col-detail',
        fromStep: 'normal',
        cell: (row) => <ShortItems row={row} />,
      },
      {
        key: 'debit',
        headerId: 'party-ledgers-col-debit',
        cell: (row) => <Num paisa={row.dr_paisa} />,
        footer: <Num paisa={totals.dr_total_paisa} />,
      },
      {
        key: 'credit',
        headerId: 'party-ledgers-col-credit',
        cell: (row) => <Num paisa={row.cr_paisa} />,
        footer: <Num paisa={totals.cr_total_paisa} />,
      },
      ...(buyer
        ? [
            {
              key: 'cash',
              headerId: 'party-ledgers-col-cash',
              fromStep: 'normal' as const,
              cell: (row: StatementRowsRow) => <Num paisa={row.on_spot_paisa} />,
              footer: <Num paisa={totals.on_spot_total_paisa} />,
            },
          ]
        : []),
      {
        key: 'balance',
        headerId: 'party-ledgers-col-running',
        cell: (row) => <Balance balance={row.balance} />,
        footer: <Balance balance={totals.closing} />,
        lead: <Balance balance={totals.brought_forward} />,
      },
    ],
    [buyer, totals],
  );
  // The balance brought forward is a line of its own only when there is one (a window that starts after the
  // first entry, or an account made with a balance before the period).
  const withLead = totals.brought_forward.side !== 'zero';
  return (
    <DayTable
      columns={columns}
      rowKey={entryKey}
      withLead={withLead}
      renderDay={(key) => {
        const row = table.rows.find((candidate) => entryKey(candidate) === key);
        return row === undefined ? null : (
          <EntryDetail
            accountId={accountId}
            row={row}
            slip={slips.has(key)}
            onSlip={(shown) => {
              onSlip(key, shown);
            }}
            onDaySlip={onDaySlip}
          />
        );
      }}
      {...table}
    />
  );
}
