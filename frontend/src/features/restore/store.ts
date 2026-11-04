// What the Owner has typed and chosen on the first screen and on the restore pages (plan 56). It
// lives above the pages, so moving between steps and switching language lose nothing (Code
// Standards F39). The keys are held here only while they are needed: they are never written to disk
// or to a log, and they are dropped the moment the restore ends, fails or is left (NFR-SEC-7).

import { create } from 'zustand';
import type { UiError } from '../../bindings/UiError';
import { CLOUD_PATHS, FILE_PATH, FOLDER_PATH } from './field-paths';
import type {
  BackupRow,
  CloudFields,
  FirstRunPreview,
  FirstRunRestoreDone,
  RestoreProgress,
  RestoreWhere,
} from './types';

/** The first screen, the new-shop wizard, or the restore pages. */
export type FirstRunChoice = 'start' | 'new' | 'restore';

/** The pages of a restore: where, copy and key, check, then the restore itself (progress, done). */
export const STEP_WHERE = 0;
export const STEP_COPY = 1;
export const STEP_CHECK = 2;
export const STEP_RUN = 3;

/** The cloud folder and region most shops use (Settings, Backup). */
export const DEFAULT_PREFIX = 'mandi';
export const DEFAULT_REGION = 'auto';

/**
 * The refusal without the problems on the given fields. When he edits a field its old message goes
 * (it was about the old text), and the messages of the other fields stay until he looks again.
 */
function withoutFields(error: UiError | null, fields: readonly string[]): UiError | null {
  if (error === null) return null;
  const errors = error.errors.filter((item) => item.field === null || !fields.includes(item.field));
  if (errors.length === error.errors.length) return error;
  return errors.length === 0 ? null : { ...error, errors };
}

const EMPTY_CLOUD: CloudFields = {
  endpoint: '',
  bucket: '',
  region: DEFAULT_REGION,
  prefix: DEFAULT_PREFIX,
  key_id: '',
  secret: '',
};

interface RestoreState {
  choice: FirstRunChoice;
  step: number;
  where: RestoreWhere;
  cloud: CloudFields;
  path: string;
  /** The copies found, newest first. Null until a place has been searched. */
  copies: BackupRow[] | null;
  /** The name of the copy chosen. */
  copyName: string | null;
  backupKey: string;
  preview: FirstRunPreview | null;
  /** The restore has started and has not yet been left (it runs, then it is done). */
  active: boolean;
  progress: RestoreProgress | null;
  done: FirstRunRestoreDone | null;
  /** Why the last restore went back to the first screen. */
  failure: UiError | null;
  /** The last refusal of a step, shown next to its field. */
  error: UiError | null;
  /** One request at a time. */
  busy: boolean;
  setChoice: (choice: FirstRunChoice) => void;
  setWhere: (where: RestoreWhere) => void;
  setCloudField: (field: keyof CloudFields, text: string) => void;
  setPath: (path: string) => void;
  setBackupKey: (key: string) => void;
  chooseCopy: (name: string) => void;
  goTo: (step: number) => void;
  setBusy: (busy: boolean) => void;
  setError: (error: UiError | null) => void;
  copiesFound: (rows: BackupRow[]) => void;
  previewed: (preview: FirstRunPreview) => void;
  started: () => void;
  progressed: (progress: RestoreProgress | null) => void;
  finished: (result: FirstRunRestoreDone) => void;
  /** The restore failed: back to the first screen with the reason, and no keys kept. */
  failed: (error: UiError) => void;
  /** Leaves the restore pages for the sign-in (or the first screen), dropping everything typed. */
  reset: () => void;
}

const FRESH = {
  step: STEP_WHERE,
  where: 'cloud' as RestoreWhere,
  cloud: EMPTY_CLOUD,
  path: '',
  copies: null,
  copyName: null,
  backupKey: '',
  preview: null,
  active: false,
  progress: null,
  done: null,
  error: null,
  busy: false,
};

export const useRestoreStore = create<RestoreState>()((set) => ({
  choice: 'start',
  ...FRESH,
  failure: null,
  setChoice: (choice) => {
    // Back to the first screen leaves the restore pages: the three keys are dropped, what is not
    // secret (the address, the bucket, the folder) stays, so coming back costs little typing.
    set((state) =>
      choice === 'start'
        ? {
            choice,
            failure: null,
            backupKey: '',
            cloud: { ...state.cloud, key_id: '', secret: '' },
          }
        : { choice, failure: null },
    );
  },
  setWhere: (where) => {
    // A different place has different copies, and a path of one kind is not a path of another.
    set({ where, path: '', copies: null, copyName: null, preview: null, error: null });
  },
  setCloudField: (field, text) => {
    set((state) => ({
      cloud: { ...state.cloud, [field]: text },
      copies: null,
      copyName: null,
      error: withoutFields(state.error, [CLOUD_PATHS[field]]),
    }));
  },
  setPath: (path) => {
    set((state) => ({
      path,
      copies: null,
      copyName: null,
      error: withoutFields(state.error, [FOLDER_PATH, FILE_PATH]),
    }));
  },
  setBackupKey: (backupKey) => {
    set({ backupKey, error: null });
  },
  chooseCopy: (copyName) => {
    set((state) => (state.copyName === copyName ? {} : { copyName, preview: null }));
  },
  goTo: (step) => {
    set({ step, error: null });
  },
  setBusy: (busy) => {
    set({ busy });
  },
  setError: (error) => {
    set({ error });
  },
  copiesFound: (rows) => {
    // The newest is chosen; the backend sends them newest first.
    set({
      copies: rows,
      copyName: rows[0]?.name ?? null,
      preview: null,
      error: null,
      step: STEP_COPY,
    });
  },
  previewed: (preview) => {
    set({ preview, error: null, step: STEP_CHECK });
  },
  started: () => {
    set({ active: true, step: STEP_RUN, progress: null, done: null, error: null });
  },
  progressed: (progress) => {
    set({ progress });
  },
  finished: (result) => {
    // The keys have done their job.
    set({ done: result, backupKey: '', cloud: { ...EMPTY_CLOUD }, progress: null });
  },
  failed: (error) => {
    set({ ...FRESH, choice: 'start', failure: error });
  },
  reset: () => {
    set({ ...FRESH, choice: 'start', failure: null });
  },
}));

/** True while the restore pages must stay on screen, even after the app has a data file. */
export function isRestoreActive(): boolean {
  return useRestoreStore.getState().active;
}
