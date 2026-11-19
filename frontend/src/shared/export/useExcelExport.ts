// The Excel export of a report or an analytics view (D5-Q1, backend round 10). One press asks where to
// save in the system's save box, then runs the command, which writes EVERY row of the filter (never
// only the page on screen), and tells how far it is by the event `export-progress` (reading, writing
// with done and total, done). The window stays free while it runs. The app never opens or reads the
// file back.
//
// The states: idle, choosing (the save box is open), running (with the stage and the rows so far),
// done (with the path and the row count) and failed (with the backend's refusal). Esc answers at the
// overlay level while the export runs or has finished: while it runs it does nothing (the export
// cannot be stopped, and leaving the screen would lose its result); after it, Esc closes the notice.
//
// Nothing here works out any figure: the numbers shown are the counts the backend sends.

import { useLocalization } from '@fluent/react';
import { useCallback, useEffect, useRef, useState } from 'react';
import type { ExcelExported } from '../../bindings/ExcelExported';
import type { ExportStage } from '../../bindings/ExportStage';
import type { Language } from '../../bindings/Language';
import type { UiError } from '../../bindings/UiError';
import { chooseSavePath, listenEvent, uiErrorOf } from '../api';
import { useLatest } from '../hooks/useLatest';
import { useEscape } from '../keyboard';

/** The event the backend sends while it writes the file. */
export const EXPORT_PROGRESS_EVENT = 'export-progress';

/** The extension of an Excel file, without its dot. */
const EXTENSION = 'xlsx';

export type ExportState =
  | { kind: 'idle' }
  | { kind: 'choosing' }
  | { kind: 'running'; stage: ExportStage; done: number; total: number }
  | { kind: 'done'; result: ExcelExported }
  | { kind: 'failed'; error: UiError };

/** What the screen hands the hook: a name for the box, and the command that writes the file. */
export interface ExcelExportOptions {
  /** The file name the save box starts with, for example `R18-20261024.xlsx`. */
  defaultName: () => string;
  /** Runs the export command with the screen's CURRENT filters. */
  run: (job: { path: string; jobId: string; language: Language }) => Promise<ExcelExported>;
}

export interface ExcelExporter {
  state: ExportState;
  /** The save box is open or the file is being written: the button cannot be pressed again. */
  busy: boolean;
  /** Asks where to save, then writes the file with the sheet in `language`. */
  start: (language: Language) => void;
  /** Closes the notice of a finished or failed export. */
  dismiss: () => void;
}

/** The file name for a day: `stem-20261024.xlsx`. The date is the computer's own. */
export function exportFileName(stem: string, today: Date): string {
  const two = (part: number) => String(part).padStart(2, '0');
  const stamp = `${String(today.getFullYear())}${two(today.getMonth() + 1)}${two(today.getDate())}`;
  return `${stem}-${stamp}.${EXTENSION}`;
}

/** The path with `.xlsx` at its end: the backend refuses any other ending. */
export function withExtension(path: string): string {
  return path.toLowerCase().endsWith(`.${EXTENSION}`) ? path : `${path}.${EXTENSION}`;
}

let jobCounter = 0;

/** A job id the backend accepts: letters and digits, at most 64. A new one for each export. */
function newJobId(): string {
  jobCounter += 1;
  return `x${Date.now().toString(36)}${jobCounter.toString(36)}`;
}

const STAGES: readonly string[] = ['reading', 'writing', 'done'];

/** The progress of this job, or null for any other payload or another job's. */
function progressOf(
  payload: unknown,
  jobId: string,
): { stage: ExportStage; done: number; total: number } | null {
  if (typeof payload !== 'object' || payload === null) return null;
  if (!('job_id' in payload) || payload.job_id !== jobId) return null;
  if (
    !('stage' in payload) ||
    typeof payload.stage !== 'string' ||
    !STAGES.includes(payload.stage)
  ) {
    return null;
  }
  if (!('done' in payload) || typeof payload.done !== 'number') return null;
  if (!('total' in payload) || typeof payload.total !== 'number') return null;
  const stage = payload.stage as ExportStage;
  return { stage, done: payload.done, total: payload.total };
}

export function useExcelExport(options: ExcelExportOptions): ExcelExporter {
  const { l10n } = useLocalization();
  const latest = useLatest(options);
  const [state, setState] = useState<ExportState>({ kind: 'idle' });
  const current = useRef<ExportState>(state);
  const alive = useRef(true);

  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);

  const change = useCallback((next: ExportState | ((now: ExportState) => ExportState)) => {
    const value = typeof next === 'function' ? next(current.current) : next;
    current.current = value;
    // The export goes on in the backend if the screen is left; only the notice is gone.
    if (alive.current) setState(value);
  }, []);

  const start = useCallback(
    (language: Language) => {
      const now = current.current.kind;
      if (now === 'choosing' || now === 'running') return;
      void (async () => {
        change({ kind: 'choosing' });
        let chosen: string | null;
        try {
          chosen = await chooseSavePath({
            defaultName: latest().defaultName(),
            extension: EXTENSION,
            filterName: l10n.getString('export-file-type'),
          });
        } catch (error) {
          // The box could not open: nothing was chosen and nothing was written.
          change({ kind: 'failed', error: uiErrorOf(error) });
          return;
        }
        if (chosen === null) {
          change({ kind: 'idle' });
          return;
        }
        const jobId = newJobId();
        change({ kind: 'running', stage: 'reading', done: 0, total: 0 });
        // Listening starts before the command, so the first event is not missed.
        const unlisten = await listenEvent(EXPORT_PROGRESS_EVENT, (payload) => {
          const progress = progressOf(payload, jobId);
          if (progress === null) return;
          change((was) => (was.kind === 'running' ? { kind: 'running', ...progress } : was));
        });
        try {
          const result = await latest().run({ path: withExtension(chosen), jobId, language });
          change({ kind: 'done', result });
        } catch (error) {
          change({ kind: 'failed', error: uiErrorOf(error) });
        } finally {
          unlisten();
        }
      })();
    },
    [change, l10n, latest],
  );

  const dismiss = useCallback(() => {
    change((was) => (was.kind === 'done' || was.kind === 'failed' ? { kind: 'idle' } : was));
  }, [change]);

  // Esc (PRD FR-7.18): closes a finished notice; while the file is written it does nothing.
  useEscape(
    'overlay',
    () => {
      dismiss();
    },
    state.kind === 'running' || state.kind === 'done' || state.kind === 'failed',
  );

  return {
    state,
    busy: state.kind === 'choosing' || state.kind === 'running',
    start,
    dismiss,
  };
}
