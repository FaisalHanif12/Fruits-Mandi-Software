// What the screen پچھلی بکری needs for its print and its Excel (plan 72 P8, PRD FR-12.46, FR-19.3): the two
// buttons with their keys (Print `F9`, Excel), the notices of the Excel export, and the preview that takes
// the place of the page while it is open. The screen builds the filter it asks the list with (today, the
// window, the filters and the switch "show every بکری"); Print and Excel follow it exactly, so what is on
// paper and in the sheet is what the screen shows. Nothing here works out a figure.
//
//   const print = usePastPrintScreen(filter);
//   if (print.preview !== null) return print.preview;
//   ... actions={<>{print.buttons} ...</>}  and  {print.notices} under the filters

import { Localized, useLocalization } from '@fluent/react';
import { Printer } from 'lucide-react';
import { useMemo, useState, type ReactNode } from 'react';
import {
  ExcelExportButton,
  ExcelExportNotices,
  exportFileName,
  useExcelExport,
} from '../../../shared/export';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import { Button } from '../../../shared/ui';
import { PastPreview } from '../components/PastPreview';
import { bikriListExportExcel } from '../past-print-api';
import type { PastPrintFilter } from '../past-print-filter';

export interface PastPrintScreen {
  /** The Print and Excel buttons, for the actions of the screen. */
  buttons: ReactNode;
  /** The notices of the Excel export, for the area under the actions. */
  notices: ReactNode;
  /** The preview, or null while the list itself is on screen. It takes the place of the page. */
  preview: ReactNode;
}

/**
 * What the screen builds: the window and the filters it asks the list with. Today's section is always
 * on the print of this screen, so `with_today` is added here and the screen need not send it.
 */
export type PastScreenFilter = Omit<PastPrintFilter, 'with_today'>;

/**
 * `screenFilter` is null while the screen does not know its days yet: the buttons are then dimmed. With
 * `compact` (a narrow window, plan 72) Print is its icon and its key, and Excel its icon: the words are
 * the names of the buttons, so a screen reader still hears them.
 */
export function usePastPrintScreen(
  screenFilter: PastScreenFilter | null,
  { compact = false }: { compact?: boolean } = {},
): PastPrintScreen {
  const { l10n } = useLocalization();
  const filter = useMemo<PastPrintFilter | null>(
    () => (screenFilter === null ? null : { with_today: true, ...screenFilter }),
    [screenFilter],
  );
  const language = useLanguage();
  const [previewing, setPreviewing] = useState(false);
  const exporter = useExcelExport({
    defaultName: () => exportFileName('past-sales', new Date()),
    run: ({ path, jobId, language: sheet }) =>
      filter === null
        ? Promise.reject(new Error('the list cannot be exported before its days are known'))
        : bikriListExportExcel({ ...filter, job_id: jobId, language: sheet, path }),
  });
  const open = () => {
    if (filter !== null) setPreviewing(true);
  };
  useShortcut(SHORTCUTS.print, open, { enabled: filter !== null && !previewing });
  // Esc leaves the preview and brings the list back (PRD FR-7.18: the screen is left last).
  useEscape(
    'leave',
    () => {
      setPreviewing(false);
    },
    previewing,
  );
  const buttons = (
    <>
      <Button
        shortcut={SHORTCUTS.print}
        disabled={filter === null}
        {...(compact ? { accessibleName: l10n.getString('print-box-print') } : {})}
        onClick={open}
      >
        <Printer aria-hidden="true" className="size-icon-md" />
        {compact ? null : <Localized id="print-box-print" />}
      </Button>
      <ExcelExportButton
        exporter={exporter}
        language={language}
        disabled={filter === null}
        compact={compact}
      />
    </>
  );
  return {
    buttons,
    notices: <ExcelExportNotices exporter={exporter} />,
    preview:
      previewing && filter !== null ? (
        <PastPreview
          filter={filter}
          exporter={exporter}
          onBack={() => {
            setPreviewing(false);
          }}
        />
      ) : null,
  };
}
