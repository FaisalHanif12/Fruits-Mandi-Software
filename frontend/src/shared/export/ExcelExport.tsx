// The Excel action of a report or an analytics view (D5-Q1): the button, and under it the notices of
// the export: how far it is, where the file is saved, or why it was refused. The button sits beside
// Print preview on every report and every view, and beside PDF and Print in the print preview (where
// the file takes the print language box's choice). The notices say each state in words and with an
// icon, never by colour alone (NFR-USE-12). The bar's look is in export.css, imported by globals.css. The refusals the backend can send are each told at the
// right place: a bad path, too many rows for one sheet, a full disk; a bad job id and an overflow are
// ours or the backend's slips, so they get the plain "something went wrong".

import { Localized, useLocalization } from '@fluent/react';
import { FileSpreadsheet } from 'lucide-react';
import type { ReactNode } from 'react';
import type { Language } from '../../bindings/Language';
import type { UiError } from '../../bindings/UiError';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { FALLBACK_ERROR_MESSAGE_ID } from '../api';
import { groupDigits, useGrouping } from '../format';
import { Button, Code, ErrorMessage, Notice } from '../ui';
import type { ExcelExporter } from './useExcelExport';

/** The fields of the export commands whose refusal has words of its own. */
const PATH_FIELD = 'path';
const FILTERS_FIELD = 'filters';
const JOB_FIELD = 'job_id';

interface ExcelExportButtonProps {
  exporter: ExcelExporter;
  /** The language of the sheet: the print language box in a preview, else the app language. */
  language: Language;
  /** The report cannot run yet (a party is still to be chosen). */
  disabled?: boolean;
}

export function ExcelExportButton({
  exporter,
  language,
  disabled = false,
}: ExcelExportButtonProps) {
  return (
    <Button
      busy={exporter.busy}
      disabled={disabled}
      onClick={() => {
        exporter.start(language);
      }}
    >
      <FileSpreadsheet aria-hidden="true" className="size-icon-md" />
      <Localized id="export-excel" />
    </Button>
  );
}

/** A refusal as the words to show: the ids of our own messages, and the problems left for the common ones. */
function sortRefusal(error: UiError): { own: string[]; rest: UiErrorItem[] } {
  const own: string[] = [];
  const rest: UiErrorItem[] = [];
  for (const item of error.errors) {
    if (item.code === 'ERR_OUT_OF_RANGE' && item.field === PATH_FIELD) {
      if (!own.includes('export-path-refused')) own.push('export-path-refused');
    } else if (item.code === 'ERR_OUT_OF_RANGE' && item.field === FILTERS_FIELD) {
      if (!own.includes('export-too-many-rows')) own.push('export-too-many-rows');
    } else if (
      (item.code === 'ERR_OUT_OF_RANGE' && item.field === JOB_FIELD) ||
      item.code === 'ERR_OVERFLOW'
    ) {
      // A wrong job id is our own slip and an overflow is the backend's: plain words, no range.
      if (!rest.some((kept) => kept.code === 'ERR_INTERNAL')) {
        rest.push({ code: 'ERR_INTERNAL', field: null, values: {} });
      }
    } else {
      rest.push(item);
    }
  }
  return { own, rest };
}

function Failure({ error, close }: { error: UiError; close: ReactNode }) {
  const { own, rest } = sortRefusal(error);
  // A refusal with nothing in it is a backend slip: the plain words, as everywhere.
  const empty = own.length === 0 && rest.length === 0;
  return (
    <Notice tone="error" action={close}>
      <span className="flex flex-col gap-1">
        {own.map((id) => (
          <Localized key={id} id={id}>
            <span />
          </Localized>
        ))}
        {rest.map((item, index) => (
          <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
            <span />
          </ErrorMessage>
        ))}
        {empty ? (
          <Localized id={FALLBACK_ERROR_MESSAGE_ID}>
            <span />
          </Localized>
        ) : null}
        {rest.length > 0 && error.ref !== '' ? (
          <Localized id="err-reference" vars={{ ref: error.ref }} elems={{ code: <Code /> }}>
            <span className="text-ink-muted" />
          </Localized>
        ) : null}
      </span>
    </Notice>
  );
}

export function ExcelExportNotices({ exporter }: { exporter: ExcelExporter }) {
  const state = exporter.state;
  const grouping = useGrouping();
  const { l10n } = useLocalization();
  // The counts are the backend's, written with the app's digit grouping like every number.
  const count = (value: number) => groupDigits(String(value), grouping);
  const close = (
    <Button onClick={exporter.dismiss}>
      <Localized id="export-close" />
    </Button>
  );
  switch (state.kind) {
    case 'running': {
      // The bar is the browser's own: the counts are the backend's, nothing is worked out here. While
      // the total is not known yet (reading) it moves without a value.
      const known = state.stage !== 'reading' && state.total > 0;
      return (
        <Notice tone="info">
          <span className="flex flex-col gap-2">
            {state.stage === 'writing' ? (
              <Localized
                id="export-writing"
                vars={{ done: count(state.done), total: count(state.total) }}
              >
                <span />
              </Localized>
            ) : (
              <Localized id={state.stage === 'done' ? 'export-finishing' : 'export-reading'}>
                <span />
              </Localized>
            )}
            {known ? (
              <progress
                aria-label={l10n.getString('export-progress-label')}
                value={state.stage === 'done' ? state.total : state.done}
                max={state.total}
                className="export-progress"
              />
            ) : (
              <progress
                aria-label={l10n.getString('export-progress-label')}
                className="export-progress"
              />
            )}
          </span>
        </Notice>
      );
    }
    case 'done':
      return (
        <Notice tone="info" action={close}>
          <Localized
            id="export-done"
            vars={{ path: state.result.path, rows: count(state.result.rows) }}
          >
            <span />
          </Localized>
        </Notice>
      );
    case 'failed':
      return <Failure error={state.error} close={close} />;
    default:
      return null;
  }
}
