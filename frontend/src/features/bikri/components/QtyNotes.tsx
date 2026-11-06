// The problems of the quantity of the sale lines, written in plain view under the grid (PRD
// FR-12.6, FR-7.6). A line that sells more than the lot has left shows no figures, so the user must
// see at once what is wrong with it and what to do: "That quantity is not available: only 50 are
// left". The cell is red too, but colour and a tooltip are never the only signal (NFR-USE-12).
// One line of words for each problem, in line order; in the whole-sale view each says its lot.

import { Localized } from '@fluent/react';
import { CircleAlert } from 'lucide-react';
import type { QtyNote } from '../grid-views';

interface QtyNotesProps {
  notes: readonly QtyNote[];
  /** The whole-sale view: the lines of every lot are shown, so a note names its lot and line. */
  whole: boolean;
}

export function QtyNotes({ notes, whole }: QtyNotesProps) {
  if (notes.length === 0) return null;
  return (
    <ul className="flex flex-col gap-1">
      {notes.map((note) => (
        <li
          key={`${note.rowId}-${note.words}`}
          role="alert"
          className="flex items-start gap-2 type-body text-due"
        >
          <CircleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
          <span>
            {whole ? (
              <Localized
                id="bikri-where-line"
                vars={{ lot: String(note.lot + 1), line: String(note.line + 1) }}
              >
                <span className="type-label" />
              </Localized>
            ) : (
              <Localized id="bikri-qty-note-line" vars={{ line: String(note.line + 1) }}>
                <span className="type-label" />
              </Localized>
            )}{' '}
            {note.words}
          </span>
        </li>
      ))}
    </ul>
  );
}
