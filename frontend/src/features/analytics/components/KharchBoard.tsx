// The two parts of the expenses view M and of the kharch page (PRD FR-9.5, 18.2, plan 77): the shop's kharch (one
// row for each دکان خرچہ head with the whole shop's total) and the partners' kharch (one row for each partner with
// what he spent himself, ذاتی خرچہ, and a total). Both come from the ONE read of view M (`analytics_run`, view M, with
// `expense_part` shop or partner), so the page and the view can never differ. Each part has its own heading, chart and
// table (the table of every view: page by page, the backend's totals over the whole filter, a window of the period
// columns), and its own print preview and Excel. Enter on a figure opens the entries behind it (date, entry, name,
// reason, amount) and Esc returns. The partner part is behind the partner code (PRD FR-17.11): the command refuses it
// with ERR_PARTNER_LOCKED and the table says "enter the code" in its place, while the shop part shows (ErrorState).
//
// The board holds its own state (the open entries, the open print preview, the windows of the columns), so a language
// switch loses nothing (Code Standards F39). It adds nothing up: every figure is the backend's (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { Printer } from 'lucide-react';
import { useId, useMemo, useState } from 'react';
import type { AnalyticsKey } from '../../../bindings/AnalyticsKey';
import {
  ExcelExportButton,
  ExcelExportNotices,
  exportFileName,
  useExcelExport,
  type ExcelExporter,
} from '../../../shared/export';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import { Button } from '../../../shared/ui';
import { analyticsExportExcel } from '../api';
import { useAnalyticsPages, type LinesRequest } from '../hooks';
import type { AnalyticsFilters } from '../types';
import { AnalyticsLines } from './AnalyticsLines';
import { AnalyticsPreview } from './AnalyticsPreview';
import { AnalyticsResults } from './AnalyticsResults';

/** The two parts of the view: the shop's kharch and the partners' own. */
export type KharchPart = 'shop' | 'partner';

const PARTS: readonly KharchPart[] = ['shop', 'partner'];

/**
 * The filters of one part. The partner filter belongs to the partner part only (the shop part refuses it), and
 * each part says which it is (`expense_part`).
 */
export function kharchPartFilters(filters: AnalyticsFilters, part: KharchPart): AnalyticsFilters {
  return part === 'shop'
    ? { ...filters, partner_id: null, expense_part: 'shop' }
    : { ...filters, expense_part: 'partner' };
}

/** The Excel of one part: every row of the filters as they are now, in the sheet's language. */
function useKharchExcel(filters: AnalyticsFilters, part: KharchPart): ExcelExporter {
  return useExcelExport({
    defaultName: () => exportFileName(`kharch-${part}`, new Date()),
    run: ({ path, jobId, language }) =>
      analyticsExportExcel({
        job_id: jobId,
        view: 'M',
        filters: kharchPartFilters(filters, part),
        language,
        path,
        sort: null,
      }),
  });
}

interface OpenEntries {
  request: LinesRequest;
  about: string;
}

interface KharchSectionProps {
  part: KharchPart;
  filters: AnalyticsFilters;
  exporter: ExcelExporter;
  onPreview: () => void;
  onDrill: (
    part: KharchPart,
    row: AnalyticsKey,
    column: AnalyticsKey | null,
    about: string,
  ) => void;
}

function KharchSection({ part, filters, exporter, onPreview, onDrill }: KharchSectionProps) {
  const headingId = useId();
  const language = useLanguage();
  const query = useAnalyticsPages(
    'M',
    useMemo(() => kharchPartFilters(filters, part), [filters, part]),
  );
  const [windowStart, setWindowStart] = useState(0);
  const [otherColumns, setOtherColumns] = useState(false);
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  return (
    <section aria-labelledby={headingId} data-kharch-part={part} className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <Localized id={`analytics-filter-expense-${part}`}>
          <h2 id={headingId} className="type-section text-ink" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <Button onClick={onPreview}>
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <ExcelExportButton exporter={exporter} language={language} />
        </div>
      </div>
      <ExcelExportNotices exporter={exporter} />
      <AnalyticsResults
        view="M"
        query={query}
        kept={null}
        filtering={false}
        onClear={() => undefined}
        windowStart={windowStart}
        onWindow={setWindowStart}
        otherColumns={otherColumns}
        onOtherColumns={setOtherColumns}
        onDrill={(row, column, about) => {
          onDrill(part, row, column, about);
        }}
        focusRowId={focusRowId}
        onFocusRow={setFocusRowId}
      />
    </section>
  );
}

interface KharchBoardProps {
  /**
   * The period, the grouping and the partner as the backend reads them. `expense_part` is the board's own; a value
   * given here is replaced for each part.
   */
  filters: AnalyticsFilters;
  /** Esc on the board itself (not in its entries or its print preview): the screen it sits in goes back. */
  onLeave?: (() => void) | undefined;
}

export function KharchBoard({ filters, onLeave }: KharchBoardProps) {
  const [entries, setEntries] = useState<OpenEntries | null>(null);
  const [preview, setPreview] = useState<KharchPart | null>(null);
  const shopExcel = useKharchExcel(filters, 'shop');
  const partnerExcel = useKharchExcel(filters, 'partner');
  const exporters: Record<KharchPart, ExcelExporter> = { shop: shopExcel, partner: partnerExcel };

  // Esc: the print preview and the entries go back to the two parts; on the two parts it leaves the screen.
  useEscape(
    'leave',
    () => {
      if (preview !== null) setPreview(null);
      else if (entries !== null) setEntries(null);
      else onLeave?.();
    },
    preview !== null || entries !== null || onLeave !== undefined,
  );
  // F9 shows the print preview of the shop's part; the partner's part has its own button.
  useShortcut(SHORTCUTS.print, () => {
    if (preview === null && entries === null) setPreview('shop');
  });

  if (entries !== null) {
    return (
      <AnalyticsLines
        request={entries.request}
        about={entries.about}
        onBack={() => {
          setEntries(null);
        }}
      />
    );
  }
  if (preview !== null) {
    return (
      <AnalyticsPreview
        view="M"
        filters={kharchPartFilters(filters, preview)}
        sort={null}
        onBack={() => {
          setPreview(null);
        }}
        exporter={exporters[preview]}
      />
    );
  }
  return (
    <div className="flex flex-col gap-6">
      {PARTS.map((part) => (
        <KharchSection
          key={part}
          part={part}
          filters={filters}
          exporter={exporters[part]}
          onPreview={() => {
            setPreview(part);
          }}
          onDrill={(from, row, column, about) => {
            setEntries({
              request: {
                view: 'M',
                filters: kharchPartFilters(filters, from),
                rowKey: row,
                columnKey: column,
              },
              about,
            });
          }}
        />
      ))}
    </div>
  );
}
