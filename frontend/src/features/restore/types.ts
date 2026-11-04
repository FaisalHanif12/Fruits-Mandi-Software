// What the restore commands send and take (plan 56 Section 5). All of it comes from the generated
// bindings (Code Standards F2); this file only gives the screens one place to import them from, and
// the order in which the stages of a restore are shown.

export type { BackupRow } from '../../bindings/BackupRow';
export type { BackupTotals } from '../../bindings/BackupTotals';
export type { CloudAccountInput as CloudFields } from '../../bindings/CloudAccountInput';
export type { FirstRunCheck } from '../../bindings/FirstRunCheck';
export type { FirstRunCheckName } from '../../bindings/FirstRunCheckName';
export type { FirstRunCopies } from '../../bindings/FirstRunCopies';
export type { FirstRunListInput } from '../../bindings/FirstRunListInput';
export type { FirstRunPreview } from '../../bindings/FirstRunPreview';
export type { FirstRunPreviewInput } from '../../bindings/FirstRunPreviewInput';
export type { FirstRunRestoreDone } from '../../bindings/FirstRunRestoreDone';
export type { FirstRunRestoreInput } from '../../bindings/FirstRunRestoreInput';
export type { RestoreProgress } from '../../bindings/RestoreProgress';
export type { RestoreSource } from '../../bindings/RestoreSource';
export type { RestoreStage } from '../../bindings/RestoreStage';

import type { RestoreStage } from '../../bindings/RestoreStage';

/** Where the copy is kept: the cloud, a USB drive or folder, or one file. */
export type RestoreWhere = 'cloud' | 'folder' | 'file';

/**
 * The stages of a restore in the order the backend runs them. `done` is not a line of its own: it
 * says every line is finished. The words are Fluent messages `restore-stage-<id>`.
 */
export const RESTORE_STAGES: readonly Exclude<RestoreStage, 'done'>[] = [
  'downloading',
  'decrypting',
  'checking',
  'upgrading',
  'keys',
  'putting_in_place',
  'search_index',
];
