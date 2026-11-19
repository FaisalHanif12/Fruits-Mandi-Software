// The Excel export for the development pages (D5-Q1): a stand-in for the system's save box, for the
// event listeners of the Tauri event plugin, and for the two export commands, so the Excel button and
// its progress notice can be opened and photographed in a plain browser tab. Never part of a build:
// only the development bridges import it.
//
// Flags in the hash, for example `#/dev/reports/R18?exporting`:
//   exporting   the export stops at 500 of 1,200 rows and never answers (the progress notice stays)
//   exportfail=path|rows|disk|job   the backend refuses
//   cancelbox   the save box is closed without a choice

import type { ExcelExported } from '../../src/bindings/ExcelExported';
import { devFlag, devValue } from './dev-bridge';

type Callback = (message: unknown) => void;

const callbacks = new Map<number, Callback>();
const listeners: { event: string; handlerId: number; eventId: number }[] = [];
let nextCallback = 1;
let nextEvent = 1;

/** What Tauri's `transformCallback` does: keeps the function and hands back its number. */
export function transformCallback(callback: Callback): number {
  const id = nextCallback;
  nextCallback += 1;
  callbacks.set(id, callback);
  return id;
}

function recordOf(args: unknown): Record<string, unknown> {
  return typeof args === 'object' && args !== null ? Object.fromEntries(Object.entries(args)) : {};
}

/** The commands of the plugins the export uses. `handled` is false for any other command. */
export function exportPluginCommand(
  command: string,
  args: unknown,
): { handled: boolean; value: unknown } {
  const input = recordOf(args);
  switch (command) {
    case 'plugin:dialog|save':
      return {
        handled: true,
        value: devFlag('cancelbox') ? null : 'C:\\Reports\\R18-20261024.xlsx',
      };
    case 'plugin:event|listen': {
      const eventId = nextEvent;
      nextEvent += 1;
      const event = typeof input['event'] === 'string' ? input['event'] : '';
      const handlerId = typeof input['handler'] === 'number' ? input['handler'] : 0;
      listeners.push({ event, handlerId, eventId });
      return { handled: true, value: eventId };
    }
    case 'plugin:event|unlisten': {
      const eventId = input['eventId'];
      const at = listeners.findIndex((listener) => listener.eventId === eventId);
      if (at !== -1) listeners.splice(at, 1);
      return { handled: true, value: null };
    }
    default:
      return { handled: false, value: null };
  }
}

function emitProgress(payload: unknown): void {
  for (const listener of listeners) {
    if (listener.event !== 'export-progress') continue;
    callbacks.get(listener.handlerId)?.({ event: listener.event, id: listener.eventId, payload });
  }
}

function refusal(code: string, field: string | null): Error {
  return Object.assign(new Error(code), {
    ok: false,
    errors: [{ code, field, values: {} }],
    ref: 'E-20261024-0001',
  });
}

const WAIT_MS = 350;
const ROWS = 1200;

function pause(): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, WAIT_MS);
  });
}

/** What the two export commands answer, with the progress the backend would send for the job. */
export async function devExport(input: unknown): Promise<ExcelExported> {
  const request = recordOf(input);
  const jobId = typeof request['job_id'] === 'string' ? request['job_id'] : '';
  const path = typeof request['path'] === 'string' ? request['path'] : '';
  const fail = devValue('exportfail');
  if (fail === 'path') throw refusal('ERR_OUT_OF_RANGE', 'path');
  if (fail === 'job') throw refusal('ERR_OUT_OF_RANGE', 'job_id');
  if (fail === 'rows') throw refusal('ERR_OUT_OF_RANGE', 'filters');
  if (fail === 'disk') throw refusal('ERR_DISK_FULL', null);
  emitProgress({ job_id: jobId, stage: 'reading', done: 0, total: 0 });
  await pause();
  emitProgress({ job_id: jobId, stage: 'writing', done: 500, total: ROWS });
  // The photograph of the progress notice: the file never finishes.
  if (devFlag('exporting')) return new Promise<ExcelExported>(() => undefined);
  await pause();
  emitProgress({ job_id: jobId, stage: 'done', done: ROWS, total: ROWS });
  return { path, rows: ROWS, size_bytes: 9000 };
}
