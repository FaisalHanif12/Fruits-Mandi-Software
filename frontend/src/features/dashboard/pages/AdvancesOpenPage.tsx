// The page behind the tile پیشگی باقی (route /dues/advances; PRD FR-9.1, FR-15.8, FR-19.11, plan 79 item 3,
// v1.61 plan 100 A): every seller with an advance his open payable does not cover: what was given, adjusted by
// his sales, returned, what is open, and the part not covered, with the day of the last advance. The total of
// the not covered column is the tile's figure; with none the page says so. The
// rows, the counts and the totals are the backend's (`advances_report` with `only_open`); the screen adds
// nothing up (CLAUDE.md rule 1).
//
// - ↑ ↓ choose a seller; Enter opens the پیشگی screen with his name chosen, and his advances show under
//   the form (PRD FR-7.60). Nothing is saved by opening it, and every amount stays empty.
// - Four states: loading, an error with a retry, no open advance, and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { PiggyBank } from 'lucide-react';
import { useCallback, useState } from 'react';
import type { AdvancesReportRow } from '../../../bindings/AdvancesReportRow';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { ExcelExportNotices } from '../../../shared/export';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { PrintButtons, usePrintView } from '../print-view';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { EmptyState, Fact, ListPage, Num } from '../../../shared/ui';
import { ADVANCES_OPEN_COLUMNS } from '../other-dues-columns';
import { dashboardAdvancesOpenPrint, dashboardAdvancesOpenPrintData } from '../api';
import { useAdvancesOpen } from '../dues-hooks';

/** The id of a row; the pinned totals row has no seller and gets a name of its own. */
function rowId(row: AdvancesReportRow): string {
  const { account } = row as { account?: AdvancesReportRow['account'] };
  return account === undefined ? 'totals' : String(account.id);
}

export function AdvancesOpenPage() {
  const navigate = useNavigate();
  const [focusId, setFocusId] = useState<string | null>(null);
  const list = useAdvancesOpen();
  // Print, PDF and Excel hold every seller with an open advance, as of today (PRD FR-19.10).
  const print = usePrintView();
  const printInput = (language: Language, paper: Paper | null) => ({
    seller_id: null,
    as_of: '',
    search: null,
    only_open: true,
    language,
    paper,
  });
  const exporter = useTableExcel('advances-open', (sheet) => ({
    kind: 'advances_open',
    ...printInput(sheet, null),
  }));
  const first = list.data?.pages[0];
  const uncovered = first?.uncovered_paisa ?? null;
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);

  const open = useCallback(
    (row: AdvancesReportRow) => {
      void navigate({
        to: '/bikri/voucher',
        search: {
          type: 'advance',
          party: {
            id: row.account.id,
            code: row.account.code,
            name_ur: row.account.name_ur,
            name_en: row.account.name_en,
            ...(row.account.tag === undefined || row.account.tag === ''
              ? {}
              : { tag: row.account.tag }),
          },
        },
      });
    },
    [navigate],
  );

  // The totals row is the backend's, over every seller and not over the rows loaded.
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'seller',
      values:
        first === undefined
          ? {}
          : {
              seller: context.text('glossary-meezan'),
              given: context.money(first.given_paisa),
              adjusted: context.money(first.adjusted_paisa),
              returned: context.money(first.returned_paisa),
              open: context.money(first.open_paisa),
              ...(first.advance_covered === null
                ? {}
                : { covered: context.money(first.advance_covered) }),
              ...(first.uncovered_paisa === null
                ? {}
                : { uncovered: context.money(first.uncovered_paisa) }),
            },
    }),
    [first],
  );

  if (print.preview) {
    return (
      <ListPage icon={PiggyBank} titleId="advances-open-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['advances_open']}
          read={(language, paper) => dashboardAdvancesOpenPrintData(printInput(language, paper))}
          send={(to, language, paper) =>
            dashboardAdvancesOpenPrint({ sheet: printInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={PiggyBank}
      titleId="advances-open-title"
      actions={<PrintButtons onPreview={print.show} exporter={exporter} />}
      help={
        <Localized id="advances-open-help">
          <p />
        </Localized>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <dl className="grid grid-cols-2 gap-3">
        <Fact stat label={<Localized id="dashboard-tile-advances" />}>
          {uncovered === null ? null : <Num inherit paisa={uncovered} />}
        </Fact>
        <Fact stat label={<Localized id="advances-open-sellers" />}>
          {first === undefined ? null : <Num inherit>{String(first.total_count)}</Num>}
        </Fact>
      </dl>
      <PagedList<AdvancesReportRow>
        columns={ADVANCES_OPEN_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="advances-open-label"
        failedId="advances-open-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        empty={
          <EmptyState
            icon={PiggyBank}
            title={<Localized id="advances-open-empty" />}
            help={<Localized id="advances-open-empty-help" />}
          />
        }
        onOpen={open}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
    </ListPage>
  );
}
