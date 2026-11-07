// What stands under an answer's figures (PRD FR-7.74): the buttons that open each source read, with
// the same filters in place, "پرنٹ" for a report or a view, and the source line with the time the
// books were read ("ماخذ: پارٹی کارڈ، R13 کھاتہ، 10-10-26 6:12 شام"). Where the books were read from is
// the backend's `sources`; the panel only names them and opens them.

import { useLocalization, type ReactLocalization } from '@fluent/react';
import { useMutation } from '@tanstack/react-query';
import { FileText, Printer } from 'lucide-react';
import type { AnswerSource } from '../../../bindings/AnswerSource';
import type { ScreenRoute } from '../../../bindings/ScreenRoute';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, utcToLocalDisplay, utcToLocalTime } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { Button, ErrorNotice, showToast, stampTimeText } from '../../../shared/ui';
import { analyticsPrintView } from '../../analytics';
import { reportsPrintReport } from '../../reports';
import { analyticsFiltersOf, reportFiltersOf } from '../addresses';
import { useAssistantNavigation } from '../navigation';

/** The message of the name of each screen a source can send the Owner to (the screen's own title). */
const SCREEN_LABEL_IDS: Readonly<Record<ScreenRoute, string>> = {
  dashboard: 'layout-nav-dashboard',
  commission_rules: 'masters-rules-title',
  day_close: 'day-title',
  audit: 'settings-tab-audit',
  // The overview of the partners (PRD 17.3): the screen's own title, like the other routes.
  partner_overview: 'partners-title',
};

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
    case 'screen':
      return l10n.getString(SCREEN_LABEL_IDS[source.route]);
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

/**
 * A source with no `result_id` was not read for this answer: it is only a link (the answer to "how do I
 * print this?" opens a report with its filters set, plan 54 S2). It gets its button but is not named
 * in the source line, which says where the books were read from.
 */
function wasRead(source: AnswerSource): boolean {
  return source.result_id !== '';
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
    .filter(wasRead)
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

/** The sources a button can open: a report, a view, the dashboard or a screen. A party card has none. */
function openable(sources: readonly AnswerSource[]): AnswerSource[] {
  return sources.filter((source) => source.kind !== 'party_card');
}

function printable(source: AnswerSource): boolean {
  return source.kind === 'report' || source.kind === 'analytics';
}

/** The words of the button that opens a source: a screen is opened, a report is the full report. */
function buttonText(
  source: AnswerSource,
  label: string,
  several: boolean,
  l10n: ReactLocalization,
): string {
  if (source.kind === 'tiles') return l10n.getString('assistant-open-dashboard');
  if (source.kind === 'screen') {
    return several
      ? l10n.getString('assistant-open-screen-of', { source: label })
      : l10n.getString('assistant-open-screen');
  }
  return several
    ? l10n.getString('assistant-open-report-of', { source: label })
    : l10n.getString('assistant-open-report');
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
        {buttons.map((source, index) => {
          const label = sourceLabel(source, l10n);
          return (
            // Two link-only sources of one kind share an empty `result_id`, so the place is in the key.
            <span
              key={`${source.kind}-${source.result_id}-${String(index)}`}
              className="flex flex-wrap gap-2"
            >
              <Button
                {...(several
                  ? {
                      // The name says what the text says (the label is in the name), whatever the source is.
                      accessibleName: buttonText(source, label, several, l10n),
                    }
                  : {})}
                onClick={() => {
                  navigation.openSource(source);
                }}
              >
                <FileText aria-hidden="true" className="size-icon-md" />
                {buttonText(source, label, several, l10n)}
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
