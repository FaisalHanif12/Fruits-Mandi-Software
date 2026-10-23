// Repeat last month's regular expenses (PRD FR-7.62): the heads entered last month and not yet this
// month, each with last month's amount, for the user to check. "Fill the form" puts one into the
// form like a typed entry, so the preview, the cash question and the save work as they always do;
// nothing is saved by this panel. After a save the backend no longer proposes that head. Four
// states: loading, an error with a retry, nothing to repeat (a helpful message), and the rows.

import { Localized } from '@fluent/react';
import { CalendarCheck } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Button, Card, EmptyState, ErrorState, LoadingState, Num } from '../../../shared/ui';
import { useRepeatLastMonth } from '../hooks';
import type { RepeatedExpense } from '../types';

interface RepeatPanelProps {
  /** This month as `YYYY-MM`, taken from today. */
  month: string;
  /** The user chose a row: it goes into the form. */
  onFill: (row: RepeatedExpense) => void;
  onClose: () => void;
}

export function RepeatPanel({ month, onFill, onClose }: RepeatPanelProps) {
  const language = useLanguage();
  const repeat = useRepeatLastMonth(month, true);
  let body;
  if (repeat.isPending) {
    body = <LoadingState rows={2} />;
  } else if (repeat.isError) {
    body = (
      <ErrorState
        title={<Localized id="cashbook-repeat-failed" />}
        error={uiErrorOf(repeat.error)}
        onRetry={() => {
          void repeat.refetch();
        }}
      />
    );
  } else if (repeat.data.rows.length === 0) {
    body = (
      <EmptyState
        icon={CalendarCheck}
        title={<Localized id="cashbook-repeat-empty" />}
        help={<Localized id="cashbook-repeat-empty-help" />}
      />
    );
  } else {
    body = (
      <>
        <ul className="flex flex-col">
          {repeat.data.rows.map((row) => (
            <li
              key={row.expense_head_id}
              className="flex flex-wrap items-center justify-between gap-x-3 gap-y-2 border-b border-border py-2 type-body"
            >
              <span className="flex min-w-0 flex-col">
                <span>{displayName(row.head, language)}</span>
                <span className="text-ink-muted">
                  <Localized
                    id="cashbook-repeat-last-date"
                    vars={{ date: isoToDisplayDate(row.last_date) }}
                    elems={{ num: <Num /> }}
                  >
                    <span />
                  </Localized>
                </span>
              </span>
              <span className="flex items-center gap-3">
                <Num paisa={row.amount_paisa} />
                <Button
                  onClick={() => {
                    onFill(row);
                  }}
                >
                  <Localized id="cashbook-repeat-fill" />
                </Button>
              </span>
            </li>
          ))}
        </ul>
        <p className="flex items-baseline gap-2 type-body text-ink-muted">
          <Localized id="cashbook-repeat-total" />
          <Num paisa={repeat.data.total_paisa} />
        </p>
      </>
    );
  }
  return (
    <Card compact title={<Localized id="cashbook-repeat-title" />}>
      <Localized id="cashbook-repeat-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {body}
      <div>
        <Button onClick={onClose}>
          <Localized id="cashbook-repeat-close" />
        </Button>
      </div>
    </Card>
  );
}
