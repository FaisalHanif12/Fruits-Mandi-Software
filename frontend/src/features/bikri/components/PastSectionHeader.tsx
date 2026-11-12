// The heading of one part of پچھلی بکری, above its table (PRD 12.7, plan 72; Umar 12-11-2026): the title
// ("آج کی بکری" or "پچھلے 7 دن کی بکری") and, beside it, small chips with what the part holds: the day or
// the days it covers, how many بکری, and their خام بکری. Every figure is the backend's, as the list sent
// it (`total_count` and `totals.gross` over the whole filter); nothing is added up here (CLAUDE.md
// rule 1). The count and the خام بکری are left out while they are not known, and the خام بکری when
// there is no بکری.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { isoToDisplayDate, type Paisa } from '../../../shared/format';
import { Num, RUNNING_NUMBER } from '../../../shared/ui';
import { WINDOW_DAYS } from '../past-window';

type PastSectionHeaderProps = {
  /** The part the heading is for: today's section, or the window of earlier days. */
  part: 'today' | 'window';
  /** How many بکری the filter has in the part, or null while it is not known. */
  count: number | null;
  /** Their خام بکری, or null while it is not known. */
  gross: Paisa | null;
} & (
  | {
      /** Today, `YYYY-MM-DD`. */
      date: string;
      from?: undefined;
      to?: undefined;
    }
  | {
      date?: undefined;
      /** The first and the last day of the window, `YYYY-MM-DD`. */
      from: string;
      to: string;
    }
);

/** One small chip: a label and its value, or only the words. */
function Chip({ children }: { children: ReactNode }) {
  return (
    <li className="inline-flex items-baseline gap-1 rounded-full bg-surface-sunken px-3 py-1 type-chip text-ink-muted">
      {children}
    </li>
  );
}

export function PastSectionHeader({ part, count, gross, date, from, to }: PastSectionHeaderProps) {
  const known = count !== null;
  return (
    <header className="flex flex-wrap items-center gap-x-4 gap-y-2">
      <h2 className="type-section text-ink">
        {part === 'today' ? (
          <Localized id="bikri-past-today-title" />
        ) : (
          <Localized
            id="bikri-past-window-title"
            vars={{ days: String(WINDOW_DAYS) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
        )}
      </h2>
      <ul className="flex flex-wrap items-center gap-2">
        <Chip>
          {date === undefined ? (
            <Localized
              id="bikri-past-range"
              vars={{ from: isoToDisplayDate(from), to: isoToDisplayDate(to) }}
              elems={RUNNING_NUMBER}
            >
              <span />
            </Localized>
          ) : (
            <Num inherit>{isoToDisplayDate(date)}</Num>
          )}
        </Chip>
        {known ? (
          <Chip>
            <Localized id="bikri-past-day-sales" />
            <Num inherit>{String(count)}</Num>
          </Chip>
        ) : null}
        {known && count > 0 && gross !== null ? (
          <Chip>
            <Localized id="glossary-kham-bikri" />
            <Num inherit paisa={gross} />
          </Chip>
        ) : null}
      </ul>
    </header>
  );
}
