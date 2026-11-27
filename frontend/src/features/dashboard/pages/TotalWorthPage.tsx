// The page of کل مالیت (route /total-worth; PRD FR-9.10, FR-9.1, FR-19.11, plan 100 B): the tile کل مالیت
// opens it. At the top the net, shown big, with the money to have and the money to give. Then the two
// sides, each with the total of every source (each part equals its own tile or report) and its rows, one
// for each party and one for روکڑ, page by page. Every figure is the backend's, read in one moment
// (`dashboard_total_worth`): the screen never adds the parts or takes one side from the other (CLAUDE.md
// rule 1). A row opens that party's history. Print, PDF and Excel hold both sides with every row.
//
// - Four states for each side: loading, an error with a retry, nothing on this side, and the rows. The
//   figures at the top come with the first page of the money to have; while they are on their way their
//   places stay empty, and a refusal shows its own error with a retry.
// - ↑ ↓ choose a row of the list that has focus; Enter opens the party's history.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, Scale } from 'lucide-react';
import { useCallback, type ReactNode } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { TotalWorthPart } from '../../../bindings/TotalWorthPart';
import type { TotalWorthRow } from '../../../bindings/TotalWorthRow';
import type { TotalWorthSide } from '../../../bindings/TotalWorthSide';
import type { TotalWorthTile } from '../../../bindings/TotalWorthTile';
import { uiErrorOf } from '../../../shared/api';
import { ExcelExportNotices } from '../../../shared/export';
import { PagedList } from '../../../shared/grid';
import {
  Card,
  EmptyState,
  ErrorState,
  Fact,
  ListPage,
  Num,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { dashboardTotalWorthPrint, dashboardTotalWorthPrintData } from '../api';
import { useTotalWorth } from '../dues-hooks';
import { PrintButtons, usePrintView } from '../print-view';
import { TOTAL_WORTH_COLUMNS, WORTH_SOURCE_IDS } from '../total-worth-columns';
import { worthLinkOf } from '../total-worth-links';

/** The id of a row: its source and its account (one account is in one source only, and روکڑ is one row). */
function rowId(row: TotalWorthRow): string {
  const { party } = row as { party?: TotalWorthRow['party'] };
  return party === undefined ? 'totals' : `${row.source}-${String(party.account_id)}`;
}

/** The net, big, with the two sides it comes from. All three are the backend's. */
function Totals({ totals }: { totals: TotalWorthTile | undefined }) {
  const figure = (paisa: TotalWorthTile['net_paisa'] | undefined) =>
    paisa === undefined ? null : <Num inherit paisa={paisa} />;
  return (
    <dl className="grid grid-cols-1 gap-3 normal:grid-cols-3" data-total-worth>
      <Fact stat label={<Localized id="total-worth-net" />}>
        {figure(totals?.net_paisa)}
      </Fact>
      <Fact stat label={<Localized id="total-worth-have" />}>
        {figure(totals?.have_paisa)}
      </Fact>
      <Fact stat label={<Localized id="total-worth-give" />}>
        {figure(totals?.give_paisa)}
      </Fact>
    </dl>
  );
}

/** Each source of one side with how many rows it has and its total, which is its tile's or report's. */
function Parts({ side, parts }: { side: TotalWorthSide; parts: readonly TotalWorthPart[] }) {
  const { l10n } = useLocalization();
  const shown = parts.filter((part) => part.side === side);
  if (shown.length === 0) return null;
  const columns: TableColumn<TotalWorthPart>[] = [
    {
      id: 'source',
      header: <Localized id="total-worth-col-source" />,
      cell: (part) => <Localized id={WORTH_SOURCE_IDS[part.source]} />,
    },
    {
      id: 'count',
      header: <Localized id="total-worth-col-count" />,
      kind: 'number',
      size: 'qty',
      cell: (part) => <Num>{String(part.count)}</Num>,
    },
    {
      id: 'total',
      header: <Localized id="total-worth-col-amount" />,
      kind: 'number',
      size: 'money',
      cell: (part) => <Num paisa={part.total_paisa} />,
    },
  ];
  return (
    <Table
      label={l10n.getString(`total-worth-parts-${side}`)}
      columns={columns}
      rows={shown}
      rowKey={(part) => part.source}
    />
  );
}

interface SideProps {
  side: TotalWorthSide;
  list: ReturnType<typeof useTotalWorth>;
  total: ReactNode;
}

/** One side: its total, the total of each source, then its rows page by page. */
function Side({ side, list, total }: SideProps) {
  const navigate = useNavigate();
  const first = list.data?.pages[0];
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const open = useCallback(
    (row: TotalWorthRow) => {
      void navigate(worthLinkOf(row));
    },
    [navigate],
  );
  return (
    <Card compact icon={Scale} title={<Localized id={`total-worth-${side}`} />}>
      {total}
      {first === undefined ? null : <Parts side={side} parts={first.parts} />}
      <PagedList<TotalWorthRow>
        columns={TOTAL_WORTH_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId={`total-worth-${side}-label`}
        failedId="total-worth-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        autoHeight
        empty={
          <EmptyState
            icon={Scale}
            title={<Localized id={`total-worth-${side}-empty`} />}
            help={<Localized id="total-worth-empty-help" />}
          />
        }
        onOpen={open}
      />
    </Card>
  );
}

export function TotalWorthPage() {
  const have = useTotalWorth('have');
  const give = useTotalWorth('give');
  const totals = have.data?.pages[0]?.totals;
  const print = usePrintView();
  const printInput = (language: Language, paper: Paper | null) => ({ language, paper });
  const exporter = useTableExcel('total-worth', (sheet) => ({
    kind: 'total_worth',
    ...printInput(sheet, null),
  }));
  if (print.preview) {
    return (
      <ListPage icon={Landmark} titleId="total-worth-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['total_worth']}
          read={(language, paper) => dashboardTotalWorthPrintData(printInput(language, paper))}
          send={(to, language, paper) =>
            dashboardTotalWorthPrint({ sheet: printInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }
  const sideTotal = (side: TotalWorthSide) =>
    totals === undefined ? null : (
      <p className="type-body flex justify-end text-ink">
        <Localized
          id="total-worth-side-total"
          elems={{
            money: <Num inherit paisa={side === 'have' ? totals.have_paisa : totals.give_paisa} />,
          }}
        >
          <span className="type-field" />
        </Localized>
      </p>
    );
  return (
    <ListPage
      icon={Landmark}
      titleId="total-worth-title"
      actions={<PrintButtons onPreview={print.show} exporter={exporter} />}
      help={
        <Localized id="total-worth-help">
          <p />
        </Localized>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      {have.isError ? (
        <ErrorState
          title={<Localized id="total-worth-failed" />}
          error={uiErrorOf(have.error)}
          onRetry={() => {
            void have.refetch();
          }}
        />
      ) : null}
      <Totals totals={totals} />
      <div className="no-scrollbar flex min-h-0 flex-1 flex-col gap-3 overflow-auto pb-corner">
        <Side side="have" list={have} total={sideTotal('have')} />
        <Side side="give" list={give} total={sideTotal('give')} />
      </div>
    </ListPage>
  );
}
