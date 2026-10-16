// One partner on the overview (PRD 17.3): his name and eight numbers, all worked out by the
// backend and only shown here (CLAUDE.md rule 1). The columns of plan 17 (returned, خالص, کل رقم)
// are not here yet. Money goes through `<Num paisa>`, which writes it with the app's grouping and
// keeps it left to right on the physical right edge; a negative amount shows its minus sign.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { formatBps } from '../../../shared/format';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Card, Num } from '../../../shared/ui';
import type { PartnerOverviewRow } from '../types';

interface MetricProps {
  labelId: string;
  children: ReactNode;
}

function Metric({ labelId, children }: MetricProps) {
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <Localized id={labelId}>
        <dt className="type-body text-ink-muted" />
      </Localized>
      <dd dir="ltr" className="type-title text-end">
        {children}
      </dd>
    </div>
  );
}

export function OverviewCard({ row }: { row: PartnerOverviewRow }) {
  const language = useLanguage();
  return (
    <Card
      title={
        <span className="flex flex-wrap items-baseline gap-x-3">
          <span lang={language}>{displayName(row, language)}</span>
          <span className="type-body text-ink-muted" lang={otherScriptLang(language)}>
            {otherScriptName(row, language)}
          </span>
        </span>
      }
    >
      <dl className="grid grid-cols-4 gap-4 wide:grid-cols-8">
        <Metric labelId="partners-share-now">
          <Num>{`${formatBps(row.share_bps)}%`}</Num>
        </Metric>
        <Metric labelId="partners-metric-today">
          <Num paisa={row.today_income_paisa} />
        </Metric>
        <Metric labelId="partners-metric-month">
          <Num paisa={row.month_income_paisa} />
        </Metric>
        <Metric labelId="partners-metric-year">
          <Num paisa={row.year_income_paisa} />
        </Metric>
        <Metric labelId="partners-metric-expense">
          <Num paisa={row.month_expense_paisa} />
        </Metric>
        <Metric labelId="partners-metric-profit">
          <Num paisa={row.month_profit_paisa} />
        </Metric>
        <Metric labelId="partners-metric-taken">
          <Num paisa={row.month_taken_paisa} />
        </Metric>
        <Metric labelId="partners-metric-capital">
          <Num paisa={row.capital_balance_paisa} />
        </Metric>
      </dl>
    </Card>
  );
}
