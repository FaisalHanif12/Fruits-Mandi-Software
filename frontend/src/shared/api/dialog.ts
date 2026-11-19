// The system's "save as" box (Tauri dialog plugin, capability `dialog:allow-save`). The screens that
// write a file the user names (the support file of Settings, the Excel exports later) ask for the
// path here and hand it to their command. The backend checks the path again and is the final word
// (Code Standards V0).

import { open, save } from '@tauri-apps/plugin-dialog';

export interface SavePathOptions {
  /** The file name the box starts with, for example `mandi-support-20261023.txt`. */
  defaultName: string;
  /** The extension without its dot, for example `txt`. */
  extension: string;
  /** What the extension is, in the app language, for the box's type list ("Text file"). */
  filterName: string;
}

/** The path the user chose, or null when he closed the box without choosing. */
export async function chooseSavePath({
  defaultName,
  extension,
  filterName,
}: SavePathOptions): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    filters: [{ name: filterName, extensions: [extension] }],
  });
}

export interface OpenPathOptions {
  /** True to choose a folder (a USB drive), false to choose one file. */
  directory: boolean;
  /** For a file: the extension without its dot, for example `mbk`. */
  extension?: string;
  /** What the extension is, in the app language, for the box's type list ("Backup copy"). */
  filterName?: string;
}

/**
 * The system's "open" box (capability `dialog:allow-open`): a file or a folder the user picks, for
 * example the backup copy to restore from (plan 56). The path is only what the user chose; the
 * backend checks it again and is the final word (Code Standards V0). Null when he closed the box.
 */
export async function chooseOpenPath({
  directory,
  extension,
  filterName,
}: OpenPathOptions): Promise<string | null> {
  const chosen = await open({
    multiple: false,
    directory,
    ...(directory || extension === undefined
      ? {}
      : { filters: [{ name: filterName ?? extension, extensions: [extension] }] }),
  });
  return typeof chosen === 'string' ? chosen : null;
}
