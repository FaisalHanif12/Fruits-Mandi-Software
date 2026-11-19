// The words of a changed cell (plan 16): the tooltip "Changed. It was: 600" of a cell the user
// changed in a posting open for edit. Pure; the old text is the loaded one, never worked out.

import type { Language } from '../../i18n';
import { displayName } from '../../shared/i18n';
import type { CellWas } from './edit-changes';
import { fixedWord } from './edit-fields';
import type { Say } from './preview-view';

type Cells = Record<string, Record<string, CellWas>>;

/** One changed cell as words. */
function wasWords(
  was: CellWas,
  scope: 'lot' | 'line',
  field: string,
  say: Say,
  language: Language,
): string {
  if (was.record !== null)
    return say('bikri-edit-was', { value: displayName(was.record, language) });
  if (was.text.trim() === '') return say('bikri-edit-was-empty');
  const word = fixedWord(scope, field, was.text);
  return say('bikri-edit-was', { value: word === null ? was.text : say(word) });
}

/** The tooltips of the changed cells of a grid, by row and field. */
export function changedCellWords(
  cells: Cells,
  scope: 'lot' | 'line',
  say: Say,
  language: Language,
): Record<string, Record<string, string>> {
  const out: Record<string, Record<string, string>> = {};
  for (const [rowId, fields] of Object.entries(cells)) {
    const row: Record<string, string> = {};
    for (const [field, was] of Object.entries(fields)) {
      row[field] = wasWords(was, scope, field, say, language);
    }
    out[rowId] = row;
  }
  return out;
}
