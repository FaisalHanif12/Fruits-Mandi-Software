// The frame of a print preview (PRD FR-7.36, FR-19.3, FR-19.13): the title, the choice of print
// language and paper, the note about the print job, the three buttons (back, PDF, print) and the page
// itself. A report and an analytics view each fill it with their own page; the frame is the same, so
// the two previews look and work the same (NFR-USE-10). The print language can differ from the app
// language (an Urdu report can be previewed while the app is in English).
//
// Sending the page to a printer and saving it as a PDF need the print job for reports, which is not
// in the backend yet (the print data is). The two buttons are there, dimmed, with a plain line that
// says so, so the way the screen will work is already visible and nothing is faked.

import { Localized, useLocalization } from '@fluent/react';
import { Printer } from 'lucide-react';
import type { ReactNode } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { Button, Field, SegmentedControl, Notice, type SegmentedOption } from '../../../shared/ui';

const LANGUAGE_OPTIONS: readonly SegmentedOption<Language>[] = [
  { value: 'ur', lang: 'ur', label: <Localized id="common-language-ur" /> },
  { value: 'en', lang: 'en', label: <Localized id="common-language-en" /> },
];

/** Reports print on A4 or A5; the 80 mm roll is for slips (PRD 21). */
const PAPER_OPTIONS: readonly SegmentedOption<Paper>[] = [
  { value: 'a4', label: <Localized id="print-paper-a4" /> },
  { value: 'a5', label: <Localized id="print-paper-a5" /> },
];

/** The print job for reports is not built yet; the two buttons are dimmed and do nothing. */
function noop(): void {
  return undefined;
}

interface PrintPreviewFrameProps {
  language: Language;
  paper: Paper;
  onLanguageChange: (language: Language) => void;
  onPaperChange: (paper: Paper) => void;
  onBack: () => void;
  /** The page, or its loading, error or empty state. */
  children: ReactNode;
}

export function PrintPreviewFrame({
  language,
  paper,
  onLanguageChange,
  onPaperChange,
  onBack,
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
                  onChange={onLanguageChange}
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
                  onChange={onPaperChange}
                />
              </div>
            )}
          </Field>
        </div>
      </div>
      <Notice tone="info">
        <Localized id="report-print-job-later" />
      </Notice>
      <div className="flex flex-wrap justify-end gap-2">
        <Button onClick={onBack}>
          <Localized id="common-back" />
        </Button>
        <Button disabled onClick={noop}>
          <Localized id="report-print-pdf" />
        </Button>
        <Button variant="primary" disabled onClick={noop}>
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
