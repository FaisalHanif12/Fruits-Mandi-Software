// The print preview of a report (PRD FR-7.36, FR-19.3, FR-19.13): the same page that is printed,
// drawn in the print language, which can differ from the app language (an Urdu report can be
// previewed while the app is in English). It holds every row of the filter, not one page of it.
// The language and the paper are chosen here; the app's language and the filters stay as they are.
//
// Sending the page to a printer and saving it as a PDF need the print job for reports, which is
// not in the backend yet (the report print data is). The two buttons are there, dimmed, with a
// plain line that says so, so the way the screen will work is already visible and nothing is faked.

import { Localized, useLocalization } from '@fluent/react';
import { FileX, Printer } from 'lucide-react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { ReportFilters } from '../../../bindings/ReportFilters';
import type { ReportId } from '../../../bindings/ReportId';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import {
  Button,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Notice,
  SegmentedControl,
  type SegmentedOption,
} from '../../../shared/ui';
import { useReportPrintData } from '../hooks';
import { ReportPrintDocument } from '../print/ReportPrintDocument';

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

interface ReportPreviewProps {
  id: ReportId;
  filters: ReportFilters;
  onBack: () => void;
}

export function ReportPreview({ id, filters, onBack }: ReportPreviewProps) {
  const { l10n } = useLocalization();
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const query = useReportPrintData({ id, filters, language, paper });

  let body;
  if (query.isPending) {
    body = <LoadingState rows={8} />;
  } else if (query.isError) {
    body = (
      <ErrorState
        title={<Localized id="report-print-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  } else if (query.data.rows.length === 0 && query.data.summary.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <ReportPrintDocument data={query.data} />;
  }

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
                  onChange={setLanguage}
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
                  onChange={setPaper}
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
        {body}
      </div>
    </div>
  );
}
