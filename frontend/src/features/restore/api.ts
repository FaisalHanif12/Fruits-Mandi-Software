// Typed wrappers around the commands that restore a new computer (plan 56 Section 5, PRD FR-20.11).
// Components never call invoke() (Code Standards F4). The three commands run before sign-in and only
// while this computer has no data file; each carries the secrets it needs, because the backend keeps
// nothing between calls. A key is never logged.

import { callCommand } from '../../shared/api';
import type {
  FirstRunCopies,
  FirstRunListInput,
  FirstRunPreview,
  FirstRunPreviewInput,
  FirstRunRestoreDone,
  FirstRunRestoreInput,
} from './types';

/** The copies at a place, newest first. An empty place is an empty list, not an error. */
export function backupFirstRunList(input: FirstRunListInput): Promise<FirstRunCopies> {
  return callCommand<FirstRunCopies>('backup_first_run_list', { input });
}

/**
 * Fetches one copy, unlocks it and checks it, and returns its date and its books. Changes nothing on
 * this computer. A copy that fails a check is refused with an error, so a preview that comes back has
 * every check passed.
 */
export function backupFirstRunPreview(input: FirstRunPreviewInput): Promise<FirstRunPreview> {
  return callCommand<FirstRunPreview>('backup_first_run_preview', { input });
}

/**
 * Puts the books of one copy on this computer. The Owner is not signed in afterwards. If it fails, this
 * computer is still a fresh install.
 */
export function backupFirstRunRestore(input: FirstRunRestoreInput): Promise<FirstRunRestoreDone> {
  return callCommand<FirstRunRestoreDone>('backup_first_run_restore', { input });
}

/** The name of the event the backend sends while a copy is fetched, checked or restored. */
export const RESTORE_PROGRESS_EVENT = 'backup-restore-progress';
