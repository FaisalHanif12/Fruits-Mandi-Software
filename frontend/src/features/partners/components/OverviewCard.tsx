// One partner on the overview (PRD 17.3): his name and his numbers, all worked out by the backend
// and only shown here (CLAUDE.md rule 1). The first group is the income side (share, today, month,
// year, expense, profit, money taken, capital balance); the second group is plan 17's: money
// returned, خالص (C20), capital put in, the باغ، سفر اور پھل column and his کل رقم (C22, FR-17.9).
// Two buttons open his statement and his capital. Money goes through `<Num paisa>`, which writes it
// with the app's grouping and keeps it left to right on the physical right edge; a negative amount
// shows its minus sign and the due colour, so colour is never the only signal (NFR-USE-12).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, ScrollText } from 'lucide-react';
import type { ReactNode } from 'react';
import { formatBps } from '../../../shared/format';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Button, Card, Num } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import type { PartnerOverviewRow } from '../types';
import { OpenVentures } from './OpenVentures';

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
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  const name = displayName(row, language);
  return (
    <Card
      title={
        <span className="flex flex-wrap items-baseline gap-x-3">
          <span lang={language}>{name}</span>
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
      <dl className="grid grid-cols-4 gap-4 border-t border-border pt-3 wide:grid-cols-5">
        <Metric labelId="partners-metric-returned">
          <Num paisa={row.month_returned_paisa} />
        </Metric>
        <Metric labelId="partners-metric-net">
          <SignedNum paisa={row.month_net_paisa} />
        </Metric>
        <Metric labelId="partners-metric-capital-in">
          <SignedNum paisa={row.month_capital_in_paisa} />
        </Metric>
        <Metric labelId="partners-metric-ventures">
          <SignedNum paisa={row.month_ventures_paisa} />
        </Metric>
        <Metric labelId="partners-metric-total">
          <SignedNum paisa={row.total_paisa} />
        </Metric>
      </dl>
      <OpenVentures ventures={row.open_ventures} partnerName={name} />
      <div className="flex flex-wrap gap-2">
        <Button
          accessibleName={l10n.getString('partners-open-statement-of', { name })}
          onClick={() => {
            void navigate({ to: '/partners/statement', search: { partner: row.partner_id } });
          }}
        >
          <ScrollText aria-hidden="true" className="size-icon-md" />
          <Localized id="partners-open-statement" />
        </Button>
        <Button
          accessibleName={l10n.getString('partners-open-capital-of', { name })}
          onClick={() => {
            void navigate({ to: '/partners/capital', search: { partner: row.partner_id } });
          }}
        >
          <Landmark aria-hidden="true" className="size-icon-md" />
          <Localized id="partners-open-capital" />
        </Button>
      </div>
    </Card>
  );
}
