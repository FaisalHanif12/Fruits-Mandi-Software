// What every page with a Print button shares (PRD FR-19.10, FR-19.3, plan 79): the view that shows the print preview in
// place of the table, and the two buttons that open it (Print, `F9`, and Excel). `Esc` leaves the preview first
// (PRD FR-7.18). Nothing here works out a figure: the preview reads every row of the page's filter from the
// backend in the print language (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { Printer } from 'lucide-react';
import { useState } from 'react';
import { ExcelExportButton, type ExcelExporter } from '../../shared/export';
import { useLanguage } from '../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../shared/keyboard';
import { Button } from '../../shared/ui';

/** Whether the page shows its table or the preview of its print, and how to go between them. */
export function usePrintView() {
  const [preview, setPreview] = useState(false);
  useEscape(
    'leave',
    () => {
      setPreview(false);
    },
    preview,
  );
  useShortcut(SHORTCUTS.print, () => {
    if (!preview) setPreview(true);
  });
  return {
    preview,
    show: () => {
      setPreview(true);
    },
    back: () => {
      setPreview(false);
    },
  };
}

interface PrintButtonsProps {
  onPreview: () => void;
  exporter: ExcelExporter;
}

/** The Print button (with its key) and the Excel button, in the buttons of a page's name row. */
export function PrintButtons({ onPreview, exporter }: PrintButtonsProps) {
  const language = useLanguage();
  return (
    <>
      <Button shortcut={SHORTCUTS.print} onClick={onPreview}>
        <Printer aria-hidden="true" className="size-icon-md" />
        <Localized id="report-preview" />
      </Button>
      <ExcelExportButton exporter={exporter} language={language} />
    </>
  );
}
