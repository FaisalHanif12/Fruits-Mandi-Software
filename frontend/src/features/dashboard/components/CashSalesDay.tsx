// What one open day of the cash sales page shows (PRD FR-9.8): the buyers who paid something on the spot that day,
// each with his بکری, his bill and the نقدی he paid, and a total row from the backend. A چکتا buyer is marked
// with his name. The بکری number opens that بکری. The figures are the backend's; nothing is added up here
// (CLAUDE.md rule 1). Four states: loading, an error with a retry, nobody (a day with a row always has a
// buyer, so this is only for a bad day), and the rows.

import { Localized, useLocalization } from '@fluent/react';
import { Printer, SearchX } from 'lucide-react';
import type { CashSalesBuyerRow } from '../../../bindings/CashSalesBuyerRow';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import {
  Button,
  EmptyState,
  ErrorState,
  LinkButton,
  LoadingState,
  Num,
  RUNNING_NUMBER,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { nameWithTag } from '../../reports';
import { useOpenDestination } from '../destinations';
import { useCashSalesDay } from '../dues-hooks';

interface CashSalesDayProps {
  date: string;
  /** Opens the print preview of this day (PRD FR-19.10). */
  onPrint: (date: string) => void;
}

export function CashSalesDay({ date, onPrint }: CashSalesDayProps) {
  const language = useLanguage();
  const open = useOpenDestination();
  const { l10n } = useLocalization();
  const day = useCashSalesDay(date, true);

  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="cash-sales-day-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  if (day.data === undefined) return <LoadingState rows={2} />;
  const data = day.data;
  if (data.rows.length === 0) {
    return (
      <EmptyState
        icon={SearchX}
        title={<Localized id="cash-sales-day-empty" />}
        help={<Localized id="cash-sales-day-empty-help" />}
      />
    );
  }
  const columns: readonly TableColumn<CashSalesBuyerRow>[] = [
    {
      id: 'buyer',
      header: <Localized id="cash-sales-col-buyer" />,
      cell: (row) => (
        <span className="flex flex-wrap items-baseline gap-x-2">
          <span>{nameWithTag(row.party, row.party.tag, language)}</span>
          {row.is_chukta ? (
            <Localized id="cash-sales-chukta">
              <span className="type-chip text-ink-muted" />
            </Localized>
          ) : null}
        </span>
      ),
      // The day's total row says how many different buyers paid (the backend's `buyer_count`, the figure of the
      // month page and of the tile); a buyer in two بکری has two rows but is one buyer.
      total: (
        <Localized
          id="cash-sales-day-total"
          vars={{ count: data.buyer_count }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      ),
    },
    {
      id: 'bikri',
      header: <Localized id="cash-sales-col-bikri" />,
      kind: 'centre',
      size: 'qty',
      cell: (row) => (
        <LinkButton
          onClick={() => {
            open({ kind: 'bikri-book', id: row.bikri_id });
          }}
        >
          <Num>{String(row.bikri_number)}</Num>
        </LinkButton>
      ),
    },
    {
      id: 'bill',
      header: <Localized id="cash-sales-col-bill" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.bill_paisa} />,
      total: <Num paisa={data.bill_total_paisa} />,
    },
    {
      id: 'cash',
      header: <Localized id="cash-sales-col-cash" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.cash_paisa} />,
      total: <Num paisa={data.cash_total_paisa} />,
    },
  ];
  return (
    <div data-cash-sales-day={date} className="flex flex-col gap-2">
      <div className="flex justify-end">
        <Button
          onClick={() => {
            onPrint(date);
          }}
        >
          <Printer aria-hidden="true" className="size-icon-md" />
          <Localized id="report-preview" />
        </Button>
      </div>
      <Table
        label={l10n.getString('cash-sales-day-label')}
        columns={columns}
        rows={data.rows}
        rowKey={(row) => `${String(row.party.account_id)}-${String(row.bikri_id)}`}
      />
      {data.total_count > data.rows.length ? (
        <Localized
          id="cash-sales-day-more"
          vars={{ shown: String(data.rows.length), total: String(data.total_count) }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
}
