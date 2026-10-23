// The lines of the جنرل entry (PRD 16.4, entry-grid skill): a grid with the account, the one-time
// person (for a پختہ کھاتہ line), بنام, جمع and a note. The party card follows the line the cursor
// is on, and the totals, the split and the cash note beside it are the backend's. A line has one
// side only; the backend says what is wrong with a line and the grid puts it on the cell.
//
// - What is typed lives in the entry store, never only in the grid, so a language switch rebuilds
//   the grid and loses nothing (Code Standards F39).
// - Keys come from shared/grid. The one thing added here: Enter on the account of a line on a
//   پختہ کھاتہ account goes on to the person's field below the grid, and a person taken from the
//   list goes back to the بنام cell of that line (PRD FR-13.5).
// - Four states: the grid is always there (empty lines are blank rows); the totals say what is
//   missing (waiting, failed with a retry, or the answer).

import { Localized } from '@fluent/react';
import { Plus } from 'lucide-react';
import { useMemo, type ReactNode } from 'react';
import {
  EntryGrid,
  FIT_MIN,
  type EntryColumn,
  type EntryRow,
  type EntryStore,
  type GridInterceptors,
} from '../../../shared/grid';
import { Button, PartyCard } from '../../../shared/ui';
import { isPukhtaCode } from '../../pukhta_khata';
import {
  GENERAL_ACCOUNT,
  GENERAL_CR,
  GENERAL_DR,
  GENERAL_NARRATION,
  GENERAL_PERSON,
  type PeopleMap,
} from '../general-draft';
import { PERSON_ROW_ATTRIBUTE } from './GeneralPeople';

export const GENERAL_COLUMNS: readonly EntryColumn[] = [
  {
    field: GENERAL_ACCOUNT,
    type: 'picker',
    picker: { kind: 'account', scope: 'any' },
    headerId: 'cashbook-general-col-account',
    minWidth: FIT_MIN.longName,
  },
  // Read only: the one-time person chosen below the grid for a line on a پختہ کھاتہ account.
  {
    field: GENERAL_PERSON,
    type: 'label',
    headerId: 'cashbook-general-col-person',
  },
  { field: GENERAL_DR, type: 'money', headerId: 'glossary-banaam', minWidth: FIT_MIN.heading },
  { field: GENERAL_CR, type: 'money', headerId: 'glossary-jama', minWidth: FIT_MIN.heading },
  { field: GENERAL_NARRATION, type: 'text', headerId: 'cashbook-field-narration' },
];

/** Moves the cursor to the person field of a row, once the row is drawn. */
function focusPersonSoon(rowId: string): void {
  window.requestAnimationFrame(() => {
    document
      .querySelector<HTMLElement>(`[${PERSON_ROW_ATTRIBUTE}="${CSS.escape(rowId)}"] input`)
      ?.focus();
  });
}

interface GeneralLinesSectionProps {
  store: EntryStore;
  people: PeopleMap;
  /** What the grid's `label` cells show: the person's name per row. */
  calc: Readonly<Record<string, Readonly<Record<string, string>>>>;
  cellErrors: Readonly<Record<string, Readonly<Record<string, string>>>>;
  /** The id of the account the party card shows now (the focused line, or a list being browsed). */
  cardAccount: number | null;
  onCardAccount: (accountId: number | null) => void;
  /** The totals, the split and the notes (the backend's), and the person panel. */
  side: ReactNode;
}

export function GeneralLinesSection({
  store,
  people,
  calc,
  cellErrors,
  cardAccount,
  onCardAccount,
  side,
}: GeneralLinesSectionProps) {
  const interceptors = useMemo<GridInterceptors>(
    () => ({
      onEnter: (cell) => {
        if (cell.field !== GENERAL_ACCOUNT) return false;
        const row: EntryRow | undefined = store
          .getState()
          .rows.find((candidate) => candidate.rowId === cell.rowId);
        const code = row?.picked?.[GENERAL_ACCOUNT]?.code;
        // A line on a پختہ کھاتہ account needs a person before the amounts (PRD FR-13.5).
        if (row === undefined || !isPukhtaCode(code)) return false;
        if ((people[row.rowId]?.person ?? null) !== null) return false;
        focusPersonSoon(row.rowId);
        return true;
      },
    }),
    [store, people],
  );

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
          calc={calc}
          cellErrors={cellErrors}
          interceptors={interceptors}
          onCardAccount={onCardAccount}
          fit
          heightMode="auto"
        />
      </section>
      <div className="flex min-w-0 flex-col gap-3">
        <PartyCard accountId={cardAccount} />
        {side}
      </div>
    </div>
  );
}
