// The names of the fields a refusal can point at (plan 56 Section 5), so each message shows next to
// the field it concerns (PRD FR-7.6). They are the names of the backend's input.

import type { CloudFields } from './types';

export const CLOUD_PATHS: Record<keyof CloudFields, string> = {
  endpoint: 'source.cloud.endpoint',
  bucket: 'source.cloud.bucket',
  region: 'source.cloud.region',
  prefix: 'source.cloud.prefix',
  key_id: 'source.cloud.key_id',
  secret: 'source.cloud.secret',
};

export const FOLDER_PATH = 'source.folder';
export const FILE_PATH = 'source.file';
export const COPY_PATH = 'copy';
export const KEY_PATH = 'key';

/** Every field the "where" page draws. */
export const WHERE_PATHS: readonly string[] = [
  ...Object.values(CLOUD_PATHS),
  FOLDER_PATH,
  FILE_PATH,
];

/** The field the "copy and key" page draws: a problem with the copy is told above it, in a note. */
export const KEY_PATHS: readonly string[] = [KEY_PATH];
