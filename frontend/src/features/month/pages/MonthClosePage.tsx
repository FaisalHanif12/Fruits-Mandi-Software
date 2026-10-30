// مہینہ بند, the month close (route /month; PRD FR-17.6, FR-21.2). The Owner closes a finished month
// here: its income and shop expenses move into the partners' capital accounts, each partner by his
// share, and no partner's total (کل رقم) changes. The page is a list of the months with where each
// stands, and below it the month that is open: a month that can be closed shows what closing it
// would post and the Close button; a month that is closed shows when and the Reverse button.
//
// - Opening the page puts the newest month that can be closed below the list, so the usual case, the
//   month that just ended, is one press away.
// - Enter or a click on a row opens that month. The month is kept here, above the grid, so a
//   language switch (which builds the grid again) loses nothing (Code Standards F39). A month that
//   was just closed or reversed stays open, so the Owner sees what his press did.
// - Four states: loading, an error with a retry, nothing yet (no entry, so no month), and the list.

import { Localized } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { CalendarCheck } from 'lucide-react';
import { useState } from 'react';
import type { MonthRow } from '../../../bindings/MonthRow';
import { uiErrorOf } from '../../../shared/api';
import { useEscape } from '../../../shared/keyboard';
import { EmptyState, ErrorState, InfoHint, LoadingState } from '../../../shared/ui';
import { MonthClosedCard } from '../components/MonthClosedCard';
import { MonthListGrid } from '../components/MonthListGrid';
import { MonthPreviewCard } from '../components/MonthPreviewCard';
import { useMonthList } from '../hooks';
import { parseMonthSearch } from '../search';

/** The month to open first: the address's, else the newest that can be closed, else the newest. */
function firstMonth(rows: readonly MonthRow[], wanted: string | undefined): string | null {
  if (wanted !== undefined && rows.some((row) => row.month === wanted)) return wanted;
  return (rows.find((row) => row.state === 'ready') ?? rows[0])?.month ?? null;
}

export function MonthClosePage() {
  const list = useMonthList();
  const navigate = useNavigate();
  const search = parseMonthSearch(useSearch({ strict: false }));
  const [picked, setPicked] = useState<string | null>(null);
  // Esc leaves the screen (PRD FR-7.18).
  useEscape('leave', () => {
    void navigate({ to: '/partners' });
  });

  if (list.isPending) return <LoadingState />;
  if (list.isError) {
    return (
      <ErrorState
        title={<Localized id="month-load-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  }
  const rows = list.data.rows;
  const header = (
    <div className="flex items-center gap-1">
      <Localized id="month-title">
        <h1 className="type-title" />
      </Localized>
      <InfoHint>
        <Localized id="month-help">
          <p />
        </Localized>
      </InfoHint>
    </div>
  );
  if (rows.length === 0) {
    return (
      <div className="flex flex-col gap-4">
        {header}
        <EmptyState
          icon={CalendarCheck}
          title={<Localized id="month-empty" />}
          help={<Localized id="month-empty-help" />}
        />
      </div>
    );
  }
  // A month that is gone from the list (it cannot be) falls back to the first one.
  const opened = firstMonth(rows, picked ?? search.month);
  const row = rows.find((candidate) => candidate.month === opened);
  return (
    <div className="flex flex-col gap-3 short:gap-2">
      {header}
      <MonthListGrid rows={rows} selected={opened} onOpen={setPicked} />
      {row === undefined ? null : row.state === 'closed' ? (
        <MonthClosedCard key={`${row.month}-closed`} row={row} onReversed={setPicked} />
      ) : (
        <MonthPreviewCard key={`${row.month}-${row.state}`} row={row} onClosed={setPicked} />
      )}
    </div>
  );
}
