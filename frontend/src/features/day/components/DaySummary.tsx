// What the days to be closed came to (PRD FR-21.1, 4.9): the number of بکری, خام بکری, the commission
// (seller + buyer + لاگا), the shared income, each partner's share, the lots still in باقی مال, and
// above all the trial balance difference and the cash in hand, which the card shows beside it.
// Every figure is the backend's `DayCloseSummary`, worked out from stored records (CLAUDE.md
// rule 1); this file only shows them. A partner's share line has his income share, his expense
// share and his profit share (C12), so the three add up for the eye: 5,488 - 3,000 = 2,488.

import { Localized, useLocalization } from '@fluent/react';
import type { DayCloseSummary } from '../../../bindings/DayCloseSummary';
import type { DayClosePartner } from '../../../bindings/DayClosePartner';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Fact, Notice, Num, Table, type TableColumn } from '../../../shared/ui';

export function DaySummary({ summary }: { summary: DayCloseSummary }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const columns: readonly TableColumn<DayClosePartner>[] = [
    {
      id: 'partner',
      header: <Localized id="day-summary-col-partner" />,
      cell: (row) => <span lang={language}>{displayName(row.partner, language)}</span>,
    },
    {
      id: 'income',
      header: <Localized id="day-summary-col-income" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.income_paisa} />,
    },
    {
      id: 'expense',
      header: <Localized id="day-summary-col-expense" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.expense_paisa} />,
    },
    {
      id: 'profit',
      header: <Localized id="day-summary-col-profit" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.profit_paisa} />,
    },
  ];
  return (
    <section aria-labelledby="day-summary-heading" className="flex flex-col gap-3">
      <Localized id="day-summary-title">
        <h2 id="day-summary-heading" className="type-section text-ink" />
      </Localized>
      <div className="@container min-w-0">
        <dl className="grid grid-cols-2 gap-4 @form3:grid-cols-3 @form4:grid-cols-6">
          <Fact label={<Localized id="day-summary-bikri-count" />}>
            <Num>{String(summary.bikri_count)}</Num>
          </Fact>
          <Fact label={<Localized id="day-summary-gross" />}>
            <Num paisa={summary.gross_paisa} />
          </Fact>
          <Fact label={<Localized id="day-summary-commission" />}>
            <Num paisa={summary.commission_paisa} />
          </Fact>
          <Fact label={<Localized id="day-summary-shared-income" />}>
            <Num paisa={summary.shared_income_paisa} />
          </Fact>
          <Fact label={<Localized id="day-summary-open-lots" />}>
            <Num>{String(summary.open_lots)}</Num>
          </Fact>
          <Fact label={<Localized id="day-summary-open-qty" />}>
            <Num>{String(summary.open_qty)}</Num>
          </Fact>
        </dl>
      </div>
      {summary.open_lots > 0 ? (
        <Notice tone="warn">
          <Localized
            id="day-summary-stock-left-note"
            vars={{ lots: summary.open_lots, qty: summary.open_qty }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        </Notice>
      ) : null}
      {summary.partners.length === 0 ? (
        <Localized id="day-summary-no-partners">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <Table
          label={l10n.getString('day-summary-partners-label')}
          columns={columns}
          rows={summary.partners}
          rowKey={(row) => String(row.partner.id)}
        />
      )}
    </section>
  );
}
