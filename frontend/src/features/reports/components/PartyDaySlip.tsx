// One party's day slip on the party ledgers screen (PRD FR-12.43 level 3, FR-19.12, plan 66): the
// R35 slip for one account and one date, drawn by the same pieces as the report screen (the goods
// list and the card of lines), and its print preview, which is the report's own preview. Nothing
// here works out a figure: the slip, the print and the Excel sheet are the backend's R35 for these
// filters (CLAUDE.md rule 1).

import { useMemo } from 'react';
import { ExcelExportNotices, exportFileName, useExcelExport } from '../../../shared/export';
import { reportsExportExcel } from '../api';
import { useReportPages } from '../hooks';
import { PARTY_DAY_TAKES, partyDayFilters } from '../party-day';
import type { ReportRowView } from '../columns';
import { useOpenRowSource } from '../open-source';
import { ReportPreview } from './ReportPreview';
import { ReportResults } from './ReportResults';

interface PartyDayProps {
  accountId: number;
  /** The date of the slip, `YYYY-MM-DD`. */
  date: string;
}

const NO_FOCUS = null;

/** The slip as the report screen draws it, with its four states (loading, error, nothing, rows). */
export function PartyDaySlip({ accountId, date }: PartyDayProps) {
  const filters = useMemo(() => partyDayFilters(accountId, date), [accountId, date]);
  const query = useReportPages('R35', filters, false);
  const openSource = useOpenRowSource();
  // Enter on a line of goods opens the بکری it came from, as it does on the report screen (FR-19.4).
  const open = (view: ReportRowView) => {
    openSource(view.row.source);
  };
  return (
    <ReportResults
      takes={PARTY_DAY_TAKES}
      query={query}
      kept={null}
      filtering={false}
      balanceOnly={false}
      onClear={() => undefined}
      onOpen={open}
      focusRowId={NO_FOCUS}
      onFocusRow={() => undefined}
      // The screen has its side menu: the goods and the lines sit side by side only when there is room.
      slipBesideFrom="wide"
    />
  );
}

interface PartyDayPreviewProps extends PartyDayProps {
  onBack: () => void;
}

/** The print preview of the slip: print, save as PDF, paper and language, and the Excel sheet. */
export function PartyDayPreview({ accountId, date, onBack }: PartyDayPreviewProps) {
  const filters = useMemo(() => partyDayFilters(accountId, date), [accountId, date]);
  const exporter = useExcelExport({
    defaultName: () => exportFileName('R35', new Date()),
    run: ({ path, jobId, language }) =>
      reportsExportExcel({
        job_id: jobId,
        id: 'R35',
        filters,
        language,
        path,
        sort: null,
        group_by_month: false,
      }),
  });
  return (
    <div className="flex flex-col gap-3">
      <ExcelExportNotices exporter={exporter} />
      <ReportPreview
        id="R35"
        filters={filters}
        sort={null}
        groupByMonth={false}
        onBack={onBack}
        exporter={exporter}
      />
    </div>
  );
}
