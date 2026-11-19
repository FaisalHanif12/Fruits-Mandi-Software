// The actions of the restore pages (plan 56): find the copies, check one, restore it. Each reads what
// the Owner typed from the store, asks the backend through api.ts, and puts the answer back in the
// store. One request at a time (NFR-REL-1). A refusal is kept in the store and shown next to its
// field; only a failure of the restore itself goes back to the first screen. Nothing here works out
// money: the totals are the backend's (CLAUDE.md rule 1).

import { listenEvent, uiErrorOf } from '../../shared/api';
import { useAfterRestoreStore } from './after-restore-store';
import {
  RESTORE_PROGRESS_EVENT,
  backupFirstRunList,
  backupFirstRunPreview,
  backupFirstRunRestore,
} from './api';
import { STEP_CHECK, useRestoreStore } from './store';
import {
  RESTORE_STAGES,
  type RestoreProgress,
  type RestoreSource,
  type RestoreStage,
} from './types';

/** The place to read from, as the commands take it. */
export function sourceOf(): RestoreSource {
  const { where, cloud, path } = useRestoreStore.getState();
  if (where === 'cloud') return { cloud };
  return where === 'folder' ? { folder: path.trim() } : { file: path.trim() };
}

/** What the backend sends as progress, kept only when it is a stage this screen knows. */
export function progressOf(payload: unknown): RestoreProgress | null {
  if (typeof payload !== 'object' || payload === null) return null;
  const stage: unknown = Reflect.get(payload, 'stage');
  const percent: unknown = Reflect.get(payload, 'percent');
  const known = stage === 'done' || RESTORE_STAGES.some((item) => item === stage);
  if (typeof stage !== 'string' || !known) return null;
  return {
    stage: stage as RestoreStage,
    percent: typeof percent === 'number' ? Math.min(100, Math.max(0, Math.trunc(percent))) : null,
  };
}

/** Runs `work` while the backend's progress is heard, and stops listening when it ends. */
async function withProgress<T>(work: () => Promise<T>): Promise<T> {
  useRestoreStore.getState().progressed(null);
  const stopListening = await listenEvent(RESTORE_PROGRESS_EVENT, (payload) => {
    const next = progressOf(payload);
    if (next !== null) useRestoreStore.getState().progressed(next);
  });
  try {
    return await work();
  } finally {
    stopListening();
  }
}

/** Looks for the copies at the place typed. A refusal stays on the "where" page, at its field. */
export async function findCopies(): Promise<void> {
  const store = useRestoreStore.getState();
  if (store.busy) return;
  store.setBusy(true);
  store.setError(null);
  try {
    const found = await backupFirstRunList({ source: sourceOf() });
    useRestoreStore.getState().copiesFound(found.rows);
  } catch (failure: unknown) {
    useRestoreStore.getState().setError(uiErrorOf(failure));
  } finally {
    useRestoreStore.getState().setBusy(false);
  }
}

/** Opens the chosen copy with the typed key and checks it. A wrong key stays at the key field. */
export async function checkCopy(): Promise<void> {
  const store = useRestoreStore.getState();
  if (store.busy || store.copyName === null) return;
  store.setBusy(true);
  store.setError(null);
  const { copyName, backupKey } = store;
  try {
    const preview = await withProgress(() =>
      backupFirstRunPreview({ source: sourceOf(), copy: copyName, key: backupKey.trim() }),
    );
    useRestoreStore.getState().previewed(preview);
  } catch (failure: unknown) {
    useRestoreStore.getState().setError(uiErrorOf(failure));
  } finally {
    useRestoreStore.getState().progressed(null);
    useRestoreStore.getState().setBusy(false);
  }
}

/**
 * Puts the books of the chosen copy on this computer. While it runs the pages show its progress. If
 * it fails, the first screen opens again with the reason and no key kept (the computer is as it
 * was). If it works, the "done" page shows.
 */
export async function runRestore(): Promise<void> {
  const store = useRestoreStore.getState();
  // Only from the check page, and only for a copy that passed its checks.
  if (store.busy || store.copyName === null || store.step !== STEP_CHECK) return;
  if (store.preview?.checks.every((check) => check.ok) !== true) return;
  store.setBusy(true);
  store.started();
  const { copyName, backupKey } = store;
  try {
    const result = await withProgress(() =>
      backupFirstRunRestore({ source: sourceOf(), copy: copyName, key: backupKey.trim() }),
    );
    useRestoreStore.getState().finished(result);
    useAfterRestoreStore.getState().remember({
      assistantWasOn: result.assistant_was_on,
      cloudNeedsKeys: result.cloud_needs_keys,
    });
  } catch (failure: unknown) {
    useRestoreStore.getState().failed(uiErrorOf(failure));
  } finally {
    useRestoreStore.getState().setBusy(false);
  }
}
