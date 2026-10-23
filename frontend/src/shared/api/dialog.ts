// The system's "save as" box (Tauri dialog plugin, capability `dialog:allow-save`). The screens that
// write a file the user names (the support file of Settings, the Excel exports later) ask for the
// path here and hand it to their command. The backend checks the path again and is the final word
// (Code Standards V0).

import { save } from '@tauri-apps/plugin-dialog';

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
