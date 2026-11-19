// One day of a partner's statement, opened in place (PRD FR-17.12, FR-17.10, plan 76 point 11): every
// movement of that day with where it came from (the group, the sentence with its number, item or reason),
// its amount and a link to the entry, and the کل رقم before the day and after it ("after today, کل رقم = …").
// A note line (a trip's sale, cost and result, a month close) has no amount. The words of each line are the
// shared statement words (statement-text.ts), so a sentence reads the same here as in the print; every
// figure is the backend's and only shown (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { useMemo } from 'react';
import type { StatementLine } from '../../../bindings/StatementLine';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { NumberRuns } from '../../../shared/grid/NumberRuns';
import { useLanguage } from '../../../shared/i18n';
import { ErrorState, LoadingState } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import { usePartnerStatementDay } from '../money-hooks';
import { detailText, headingRepeated, headingText, type StatementSay } from '../statement-text';

interface PartnerDayCardProps {
  partnerId: number;
  /** The day, `YYYY-MM-DD`. */
  date: string;
}

export function PartnerDayCard({ partnerId, date }: PartnerDayCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const day = usePartnerStatementDay(partnerId, date, true);
  const say = useMemo<StatementSay>(
    () => ({
      language,
      grouping,
      text: (id, vars) => l10n.getString(id, vars),
      exists: (id) => l10n.getBundle(id) !== null,
    }),
    [l10n, language, grouping],
  );
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="partners-day-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  if (day.data === undefined) return <LoadingState rows={2} />;
  const lines: readonly StatementLine[] = day.data.lines;
  return (
    <div data-partner-day={date} className="flex flex-col gap-2">
      <p className="flex items-baseline gap-2 type-body text-ink-muted">
        <Localized id="partners-day-before" />
        <SignedNum paisa={day.data.opening_total_paisa} />
      </p>
      <ul className="flex flex-col gap-1">
        {lines.map((line, index) => {
          const heading = headingText(line, say);
          const detail = detailText(line, say);
          return (
            <li
              key={`${String(line.entry_id)}-${String(index)}`}
              className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 border-b border-border py-1 last:border-b-0"
            >
              <span className="flex min-w-0 flex-wrap items-baseline gap-x-2">
                {/* The group is shown only when the sentence does not already say it ("Sale 1: ..."). */}
                {headingRepeated(heading, detail) ? null : (
                  <span className="type-chip text-ink-muted">{heading}</span>
                )}
                <span className="type-body">
                  <NumberRuns text={detail} />
                </span>
              </span>
              <span className="flex items-baseline gap-3">
                {line.amount_paisa === null ? null : <SignedNum paisa={line.amount_paisa} />}
              </span>
            </li>
          );
        })}
      </ul>
      <p className="flex items-baseline gap-2 type-body">
        <Localized id="partners-day-after" />
        <SignedNum paisa={day.data.after_total_paisa} />
      </p>
    </div>
  );
}
