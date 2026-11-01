// What stands under an answer's figures (PRD FR-7.74): the buttons that open each source read, with
// the same filters in place, "پرنٹ" for a report or a view, and the source line with the time the
// books were read ("ماخذ: پارٹی کارڈ، R13 کھاتہ، 10-10-26 6:12 شام"). Where the books were read from is
// the backend's `sources`; the panel only names them and opens them.

import { useLocalization, type ReactLocalization } from '@fluent/react';
import { useMutation } from '@tanstack/react-query';
import { FileText, Printer } from 'lucide-react';
import type { AnswerSource } from '../../../bindings/AnswerSource';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, utcToLocalDisplay, utcToLocalTime } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { Button, ErrorNotice, showToast, stampTimeText } from '../../../shared/ui';
import { analyticsPrintView } from '../../analytics';
import { reportsPrintReport } from '../../reports';
import { analyticsFiltersOf, reportFiltersOf } from '../addresses';
import { useAssistantNavigation } from '../navigation';

/** The name of a source for the source line and for a button's accessible name. */
export function sourceLabel(source: AnswerSource, l10n: ReactLocalization): string {
  switch (source.kind) {
    case 'party_card':
      return l10n.getString('assistant-source-party-card');
    case 'report':
      return `${source.id} ${l10n.getString(source.label_id)}`;
    case 'analytics':
      return l10n.getString('assistant-source-analytics', {
        title: l10n.getString(source.label_id),
      });
    case 'tiles':
      return l10n.getString('layout-nav-dashboard');
  }
}

/** The time the books were read, as the Owner's own clock shows it: "10-10-2026 6:12 pm". */
export function readTimeText(readAt: string, l10n: ReactLocalization): string {
  const date = utcToLocalDisplay(readAt).slice(0, 10);
  const time = utcToLocalTime(readAt);
  // Anything the backend sent that is not a time is shown as it is.
  if (time === readAt) return readAt;
  return `${isoToDisplayDate(date)} ${stampTimeText(time, l10n)}`;
}

/** "ماخذ: …": every source once, then the time. */
export function SourceLine({
  sources,
  readAt,
}: {
  sources: readonly AnswerSource[];
  readAt: string;
}) {
  const { l10n } = useLocalization();
  const labels = sources
    .map((source) => sourceLabel(source, l10n))
    .filter((label, index, all) => all.indexOf(label) === index);
  if (labels.length === 0) return null;
  return (
    <p className="type-body text-ink-muted">
      {l10n.getString('assistant-sources-line', {
        sources: labels.join(l10n.getString('assistant-sources-separator')),
        time: readTimeText(readAt, l10n),
      })}
    </p>
  );
}

/** Prints a source with its own filters, in the app language, on A4, to the printer (FR-19.10). */
function usePrintSource() {
  const language = useLanguage();
  return useMutation({
    mutationFn: (source: AnswerSource) => {
      if (source.kind === 'report') {
        return reportsPrintReport({
          report: {
            id: source.id,
            filters: reportFiltersOf(source.filters),
            language,
            paper: 'a4',
            sort: null,
            group_by_month: false,
          },
          slips: false,
          to: 'printer',
          copies: null,
        });
      }
      if (source.kind === 'analytics') {
        return analyticsPrintView({
          view: {
            view: source.view,
            filters: analyticsFiltersOf(source.filters),
            language,
            paper: 'a4',
            sort: null,
          },
          to: 'printer',
          copies: null,
        });
      }
      return Promise.reject(new Error('this source has no print'));
    },
  });
}

/** The sources a button can open: a report, a view, or the dashboard. A party card has none. */
function openable(sources: readonly AnswerSource[]): AnswerSource[] {
  return sources.filter((source) => source.kind !== 'party_card');
}

function printable(source: AnswerSource): boolean {
  return source.kind === 'report' || source.kind === 'analytics';
}

export function SourceButtons({ sources }: { sources: readonly AnswerSource[] }) {
  const { l10n } = useLocalization();
  const navigation = useAssistantNavigation();
  const print = usePrintSource();
  const buttons = openable(sources);
  if (navigation === null || buttons.length === 0) return null;
  // With several sources each button says which one it opens; with one, the plain words do.
  const several = buttons.length > 1;
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        {buttons.map((source) => {
          const label = sourceLabel(source, l10n);
          return (
            <span key={`${source.kind}-${source.result_id}`} className="flex flex-wrap gap-2">
              <Button
                {...(several
                  ? {
                      accessibleName: l10n.getString('assistant-open-report-of', { source: label }),
                    }
                  : {})}
                onClick={() => {
                  navigation.openSource(source);
                }}
              >
                <FileText aria-hidden="true" className="size-icon-md" />
                {source.kind === 'tiles'
                  ? l10n.getString('assistant-open-dashboard')
                  : several
                    ? l10n.getString('assistant-open-report-of', { source: label })
                    : l10n.getString('assistant-open-report')}
              </Button>
              {printable(source) ? (
                <Button
                  {...(several
                    ? { accessibleName: l10n.getString('assistant-print-of', { source: label }) }
                    : {})}
                  busy={print.isPending}
                  onClick={() => {
                    print.mutate(source, {
                      onSuccess: () => {
                        showToast('success', l10n.getString('report-print-sent'));
                      },
                    });
                  }}
                >
                  <Printer aria-hidden="true" className="size-icon-md" />
                  {l10n.getString('assistant-print')}
                </Button>
              ) : null}
            </span>
          );
        })}
      </div>
      {print.isError ? <ErrorNotice error={uiErrorOf(print.error)} /> : null}
    </div>
  );
}
