// The بکری of one open day of the window of earlier days (PRD FR-12.46 v1.39, plan 75 7.7). It is the same
// list as today's section: the columns of the list of بکری, the backend's totals of that day under them, page
// buttons when the day has more rows than a page, Edit and Cancel for the row that has focus, and Enter or a
// click that opens the whole بکری. It asks the backend for the day only when it opens (one indexed call,
// NFR-SPD-24). Every figure is the backend's.

import { Localized } from '@fluent/react';
import { useState } from 'react';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { BikriListInput } from '../../../bindings/BikriListInput';
import { isoToDisplayDate } from '../../../shared/format';
import type { ListColumn } from '../../../shared/grid';
import { useBikriList } from '../hooks/useBikriReads';
import type { PastReturn } from '../past-window';
import { PastList, rowOf } from './PastList';
import { PastRowActions } from './PastRowActions';

/** The filters of the lists: the Owner's own, or none while "show every بکری" is ticked. */
export type PastFilter = Omit<BikriListInput, 'offset' | 'limit' | 'sort' | 'date' | 'from' | 'to'>;

interface PastDayRowsProps {
  /** The day, `YYYY-MM-DD`. */
  day: string;
  filter: PastFilter;
  columns: readonly ListColumn<BikriListRow>[];
  onOpen: (row: BikriListRow) => void;
  /** The بکری to focus when the day opens: the one the Owner came back from after an edit. */
  focusRow: string | null;
  back: PastReturn;
}

export function PastDayRows({ day, filter, columns, onOpen, focusRow, back }: PastDayRowsProps) {
  const query = useBikriList({ ...filter, date: isoToDisplayDate(day), from: null, to: null });
  // The row that has focus is kept here, in the state of the open day, so a language switch keeps it.
  const [focus, setFocus] = useState<string | null>(focusRow);
  // The day of a way back from an edit may open before the screen knows which بکری it was.
  const [seenFocusRow, setSeenFocusRow] = useState(focusRow);
  if (focusRow !== seenFocusRow) {
    setSeenFocusRow(focusRow);
    if (focusRow !== null) setFocus(focusRow);
  }
  const first = query.data?.pages[0];
  return (
    <div className="flex flex-col gap-2">
      <div className="flex justify-end">
        <PastRowActions row={rowOf(first, focus)} back={back} />
      </div>
      <PastList
        query={query}
        columns={columns}
        labelId="bikri-past-label"
        failedId="bikri-past-failed"
        empty={
          <p className="type-body text-ink-muted">
            <Localized id="bikri-past-day-empty" />
          </p>
        }
        onOpen={onOpen}
        focusRowId={focus}
        onFocusRow={setFocus}
        autoHeight
      />
    </div>
  );
}
