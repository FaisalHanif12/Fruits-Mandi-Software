// The frame of a print preview (PRD FR-7.36, FR-19.3, FR-19.13): the title, the choice of print
// language and paper, the buttons (back, PDF, print, and Excel where the screen has it), what the last
// job did, and the page itself. A report, its slips and an analytics view each fill it with their own
// page; the frame is the same, so the previews look and work the same (NFR-USE-10). The print language
// can differ from the app language (an Urdu report can be previewed while the app is in English).
//
// "Print" sends the page to the printer and "Save as PDF" writes a file, both through the backend's
// print job (`reports_print_report`, `analytics_print_view`) with the language and paper chosen here.
// Off Windows the backend says printing works on Windows only, and the preview stays the way to see
// the page. The Excel button (D5-Q1) saves every row of the report as a file.

import { Localized, useLocalization } from '@fluent/react';
import { FileDown, Printer } from 'lucide-react';
import type { ReactNode } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { Code } from '../../../shared/ui/Code';
import {
  Button,
  ErrorNotice,
  Field,
  SegmentedControl,
  type SegmentedOption,
} from '../../../shared/ui';
import type { PrintJob } from './usePrintJob';

const LANGUAGE_OPTIONS: readonly SegmentedOption<Language>[] = [
  { value: 'ur', lang: 'ur', label: <Localized id="common-language-ur" /> },
  { value: 'en', lang: 'en', label: <Localized id="common-language-en" /> },
];

/** Reports print on A4 or A5; the 80 mm roll is for slips (PRD 21). */
const PAPER_OPTIONS: readonly SegmentedOption<Paper>[] = [
  { value: 'a4', label: <Localized id="print-paper-a4" /> },
  { value: 'a5', label: <Localized id="print-paper-a5" /> },
];

interface PrintPreviewFrameProps {
  language: Language;
  paper: Paper;
  onLanguageChange: (language: Language) => void;
  onPaperChange: (paper: Paper) => void;
  onBack: () => void;
  /**
   * The Excel button, when the screen has one: it sits with the other actions and takes the print
   * language chosen in this frame for the sheet (D5-Q1).
   */
  excel?: ReactNode;
  /** The print job: the buttons start it and the frame shows what it did or what was refused. */
  job: PrintJob;
  /** False while the page is loading or empty: there is nothing to send yet. */
  canPrint: boolean;
  /** The page, or its loading, error or empty state. */
  children: ReactNode;
}

export function PrintPreviewFrame({
  language,
  paper,
  onLanguageChange,
  onPaperChange,
  onBack,
  excel,
  job,
  canPrint,
  children,
}: PrintPreviewFrameProps) {
  const { l10n } = useLocalization();
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <Localized id="report-print-title">
          <h2 className="type-section" />
        </Localized>
        <div className="flex flex-wrap items-end gap-4">
          <Field label={<Localized id="print-box-language" />}>
            {() => (
              <div>
                <SegmentedControl
                  options={LANGUAGE_OPTIONS}
                  value={language}
                  groupLabel={l10n.getString('print-box-language')}
                  onChange={(next) => {
                    job.reset();
                    onLanguageChange(next);
                  }}
                />
              </div>
            )}
          </Field>
          <Field label={<Localized id="report-print-paper" />}>
            {() => (
              <div>
                <SegmentedControl
                  options={PAPER_OPTIONS}
                  value={paper}
                  groupLabel={l10n.getString('report-print-paper')}
                  onChange={(next) => {
                    job.reset();
                    onPaperChange(next);
                  }}
                />
              </div>
            )}
          </Field>
        </div>
      </div>
      {job.error === null ? null : <ErrorNotice error={job.error} />}
      {job.outcome === null ? null : (
        <div role="status" className="type-body flex flex-col gap-1 text-money-in">
          {job.outcome.pdf_paths === null ? (
            <Localized id="report-print-sent" />
          ) : (
            <>
              <Localized id="print-box-done-pdf" />
              {job.outcome.pdf_paths.map((path) => (
                <Code key={path}>{path}</Code>
              ))}
            </>
          )}
        </div>
      )}
      <div className="flex flex-wrap justify-end gap-2">
        <Button onClick={onBack}>
          <Localized id="common-back" />
        </Button>
        {excel}
        <Button
          disabled={!canPrint || job.busy}
          onClick={() => {
            job.send('pdf');
          }}
        >
          <FileDown aria-hidden="true" className="size-icon-md" />
          <Localized id="report-print-pdf" />
        </Button>
        <Button
          variant="primary"
          disabled={!canPrint}
          busy={job.busy}
          onClick={() => {
            job.send('printer');
          }}
        >
          <Printer aria-hidden="true" className="size-icon-md" />
          <Localized id="report-print-print" />
        </Button>
      </div>
      <div className="min-w-0 overflow-auto rounded-3 border border-border" data-report-preview>
        {children}
      </div>
    </div>
  );
}
