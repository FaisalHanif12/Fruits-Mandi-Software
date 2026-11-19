// The page behind the tile ادھار رقم باقی (route /dues/lent; PRD FR-9.1, FR-16.5, FR-19.11, plan 79 item 3): every person
// who borrowed and still owes, with his type, what is open, the day of his oldest open loan and how much is
// past its return-by day. The total under the columns is the tile's figure. The rows, the counts and the
// totals are the backend's, over everyone; the screen adds nothing up (CLAUDE.md rule 1).
//
// - ↑ ↓ choose a person; Enter opens the ادھار رقم screen with his name chosen, and his loans (given, returned
//   and still open) show under the form. Page Up and Page Down turn the pages.
// - Four states: loading, an error with a retry, nobody owes anything, and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Handshake } from 'lucide-react';
import { useCallback, useState } from 'react';
import type { LentOpenRow } from '../../../bindings/LentOpenRow';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { ExcelExportNotices } from '../../../shared/export';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { PrintButtons, usePrintView } from '../print-view';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { EmptyState, Fact, ListPage, Num } from '../../../shared/ui';
import { LENT_OPEN_COLUMNS } from '../other-dues-columns';
import { dashboardLentOpenPrint, dashboardLentOpenPrintData } from '../api';
import { useLentOpen } from '../dues-hooks';
import { lentSearchOf } from '../dues-links';

/** The id of a row; the pinned totals row has no person and gets a name of its own. */
function rowId(row: LentOpenRow): string {
  const { party } = row as { party?: LentOpenRow['party'] };
  return party === undefined ? 'totals' : String(party.account_id);
}

export function LentOpenPage() {
  const navigate = useNavigate();
  const [focusId, setFocusId] = useState<string | null>(null);
  const list = useLentOpen();
  // Print, PDF and Excel hold every person, in the order on screen (PRD FR-19.10).
  const print = usePrintView();
  const printInput = (language: Language, paper: Paper | null) => ({
    search: null,
    sort: list.sort,
    language,
    paper,
  });
  const exporter = useTableExcel('lent-open', (sheet) => ({
    kind: 'lent_open',
    ...printInput(sheet, null),
  }));
  const first = list.data?.pages[0];
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);

  const open = useCallback(
    (row: LentOpenRow) => {
      void navigate({ to: '/rokar/lent', search: lentSearchOf(row.party) });
    },
    [navigate],
  );

  // The totals row is the backend's, over every person and not over the rows loaded.
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'person',
      values:
        first === undefined
          ? {}
          : {
              person: context.text('glossary-meezan'),
              open: context.money(first.total_paisa),
              overdue: context.money(first.overdue_paisa),
            },
    }),
    [first],
  );

  if (print.preview) {
    return (
      <ListPage icon={Handshake} titleId="lent-open-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['lent_open', list.sort]}
          read={(language, paper) => dashboardLentOpenPrintData(printInput(language, paper))}
          send={(to, language, paper) =>
            dashboardLentOpenPrint({ sheet: printInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={Handshake}
      titleId="lent-open-title"
      actions={<PrintButtons onPreview={print.show} exporter={exporter} />}
      help={
        <Localized id="lent-open-help">
          <p />
        </Localized>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <dl className="grid grid-cols-3 gap-3">
        <Fact stat label={<Localized id="dashboard-tile-lent" />}>
          {first === undefined ? null : <Num inherit paisa={first.total_paisa} />}
        </Fact>
        <Fact stat label={<Localized id="lent-open-col-overdue" />}>
          {first === undefined ? null : <Num inherit paisa={first.overdue_paisa} />}
        </Fact>
        <Fact stat label={<Localized id="lent-open-people" />}>
          {first === undefined ? null : <Num inherit>{String(first.total_count)}</Num>}
        </Fact>
      </dl>
      <PagedList<LentOpenRow>
        columns={LENT_OPEN_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="lent-open-label"
        failedId="lent-open-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        sort={{ value: list.sort, onChange: list.setSort }}
        empty={
          <EmptyState
            icon={Handshake}
            title={<Localized id="lent-open-empty" />}
            help={<Localized id="lent-open-empty-help" />}
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
