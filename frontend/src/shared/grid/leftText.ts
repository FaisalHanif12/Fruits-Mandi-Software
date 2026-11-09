// Text typed in a picker cell and never chosen (plan 61, D8 bug 1). A picker cell keeps its text
// inside the editor; when the cell is left (Tab, an arrow, a click elsewhere) with text in it and no
// record picked, the text is kept on the row under this key, so the screen can say "«ہارون» was typed
// but no buyer was chosen" and the user is never left with an empty cell and no word. The key is in
// the row store like any typed cell, so a language switch or a restored draft keeps it (F39). It
// holds only words the user typed, never a figure (CLAUDE.md rule 1).

/** The key in a row's `typed` for the text left in the picker cell `field`. */
export function leftTextKey(field: string): string {
  return `${field}Left`;
}
