// The total dues page (route /dues/total; PRD FR-9.7, FR-9.1, FR-19.11, plan 79 item 2b): the tile کل اگراہی opens it. Four
// sections, each its own list with its own total: the buyers who owe, the sellers who owe the shop, the dead
// amount and the people who borrowed (ادھار رقم). The last line is the grand total of everyone who owes
// the shop, which is the tile's figure. Each row opens that person's history. The counts and the totals are
// the backend's, read in one moment (`dashboard_total_dues`); the grand total is the backend's too and is
// never added up from the sections (CLAUDE.md rule 1).
//
// - Four states: while the totals are on their way each section already loads its own list; a refused
//   read of the totals shows an error with a retry above the sections (they keep working); a section
//   with nobody says so in words; and the rows.
// - ↑ ↓ choose a row of the list that has focus; Enter opens the person's history.

import { Localized } from '@fluent/react';
import { Scale } from 'lucide-react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { ExcelExportNotices } from '../../../shared/export';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { PrintButtons, usePrintView } from '../print-view';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, Fact, ListPage, Num } from '../../../shared/ui';
import {
  BuyersSection,
  DeadSection,
  OthersSection,
  SellersSection,
} from '../components/TotalDuesSections';
import { dashboardTotalDuesPrint, dashboardTotalDuesPrintData } from '../api';
import { useTotalDues } from '../dues-hooks';

export function TotalDuesPage() {
  const totals = useTotalDues();
  // Print, PDF and Excel hold the four parts and the grand total (PRD FR-19.10).
  const print = usePrintView();
  const printInput = (language: Language, paper: Paper | null) => ({ language, paper });
  const exporter = useTableExcel('total-dues', (sheet) => ({
    kind: 'total_dues',
    ...printInput(sheet, null),
  }));
  const parts = totals.data?.parts;
  const part = (section: 'buyers' | 'sellers' | 'dead' | 'others') =>
    parts?.find((candidate) => candidate.section === section);
  if (print.preview) {
    return (
      <ListPage icon={Scale} titleId="total-dues-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['total_dues']}
          read={(language, paper) => dashboardTotalDuesPrintData(printInput(language, paper))}
          send={(to, language, paper) =>
            dashboardTotalDuesPrint({ sheet: printInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={Scale}
      titleId="total-dues-title"
      actions={<PrintButtons onPreview={print.show} exporter={exporter} />}
      help={
        <Localized id="total-dues-help">
          <p />
        </Localized>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      {totals.isError ? (
        <ErrorState
          title={<Localized id="total-dues-failed" />}
          error={uiErrorOf(totals.error)}
          onRetry={() => {
            void totals.refetch();
          }}
        />
      ) : null}
      <BuyersSection part={part('buyers')} />
      <SellersSection part={part('sellers')} />
      <DeadSection part={part('dead')} />
      <OthersSection part={part('others')} />
      <dl>
        <Fact stat label={<Localized id="dashboard-tile-dues-total" />}>
          {totals.data === undefined ? null : <Num inherit paisa={totals.data.grand_total_paisa} />}
        </Fact>
      </dl>
    </ListPage>
  );
}
