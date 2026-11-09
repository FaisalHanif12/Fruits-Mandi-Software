// The lines of the جنرل entry (PRD 16.4, entry-grid skill): a grid with the account, بنام, جمع and
// a note. The party card follows the line the cursor
// is on, and the totals, the split and the cash note beside it are the backend's. A line has one
// side only; the backend says what is wrong with a line and the grid puts it on the cell.
//
// - What is typed lives in the entry store, never only in the grid, so a language switch rebuilds
//   the grid and loses nothing (Code Standards F39).
// - Keys come from shared/grid. This screen adds none of its own.
// - Four states: the grid is always there (empty lines are blank rows); the totals say what is
//   missing (waiting, failed with a retry, or the answer).

import { Localized } from '@fluent/react';
import { Plus } from 'lucide-react';
import type { ReactNode } from 'react';
import { EntryGrid, FIT_MIN, type EntryColumn, type EntryStore } from '../../../shared/grid';
import { Button, PartyCard } from '../../../shared/ui';
import { GENERAL_ACCOUNT, GENERAL_CR, GENERAL_DR, GENERAL_NARRATION } from '../general-draft';

export const GENERAL_COLUMNS: readonly EntryColumn[] = [
  {
    field: GENERAL_ACCOUNT,
    type: 'picker',
    picker: { kind: 'account', scope: 'any' },
    headerId: 'cashbook-general-col-account',
    minWidth: FIT_MIN.longName,
  },
  { field: GENERAL_DR, type: 'money', headerId: 'glossary-banaam', minWidth: FIT_MIN.heading },
  { field: GENERAL_CR, type: 'money', headerId: 'glossary-jama', minWidth: FIT_MIN.heading },
  { field: GENERAL_NARRATION, type: 'text', headerId: 'cashbook-field-narration' },
];

interface GeneralLinesSectionProps {
  store: EntryStore;
  cellErrors: Readonly<Record<string, Readonly<Record<string, string>>>>;
  /** The cells an earlier edit changed, with the words of that edit as their tip (plan 44). */
  cellMarks?: Readonly<Record<string, Readonly<Record<string, string>>>> | undefined;
  /** The id of the account the party card shows now (the focused line, or a list being browsed). */
  cardAccount: number | null;
  onCardAccount: (accountId: number | null) => void;
  /** The totals, the split and the notes (the backend's). */
  side: ReactNode;
}

export function GeneralLinesSection({
  store,
  cellErrors,
  cellMarks,
  cardAccount,
  onCardAccount,
  side,
}: GeneralLinesSectionProps) {
  const addLine = () => {
    store.getState().addRow();
    store.getState().setPendingFocus({ rowIndex: 'last', field: GENERAL_ACCOUNT });
  };

  return (
    <div className="grid grid-cols-1 gap-3 wide:grid-cols-3">
      <section
        aria-labelledby="cashbook-general-lines-heading"
        className="flex min-w-0 flex-col gap-2 wide:col-span-2"
      >
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Localized id="cashbook-general-lines-title">
            <h2 id="cashbook-general-lines-heading" className="type-section text-ink" />
          </Localized>
          <Button onClick={addLine}>
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="cashbook-general-add-line" />
          </Button>
        </div>
        <EntryGrid
          columns={GENERAL_COLUMNS}
          store={store}
          labelId="cashbook-general-grid-label"
          cellErrors={cellErrors}
          {...(cellMarks === undefined ? {} : { cellMarks })}
          onCardAccount={onCardAccount}
          fit
          heightMode="auto"
          minRows={4}
        />
      </section>
      <div className="flex min-w-0 flex-col gap-3">
        <PartyCard accountId={cardAccount} />
        {side}
      </div>
    </div>
  );
}
