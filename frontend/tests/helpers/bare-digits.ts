// The guard of "a number in running text is in Inter" (Code Standards F25, NFR-URD-13, Umar 26-10-2026):
// it walks the text of a drawn screen or print and lists every digit that is not inside a `<Num>`,
// a `<Code>` or a grid number cell. A digit left outside one is drawn in the font of the words around
// it, and an Urdu font may draw it in an Eastern shape. The words of a message are not changed: only
// where its numbers sit.

/** What a digit may sit in: the number components, the number cells of a grid, a code, a field, a chart. */
const NUMBER_CONTEXT =
  '.type-number, .number-run, .cell-num, .header-num, code, input, textarea, select, svg, [data-number]';

/**
 * A title or a tab names a view ("Top 10"): it is a name, not a line of words with a number in it.
 * It is the one place a number may stay in the text of its message.
 */
const NAME_CONTEXT = 'h1, h2, [role="tab"]';

/**
 * Known gaps in a file that is not this module's to change: the firm line of the shared print header
 * (`PrintHeader`, Run A) draws the firm's address and phone in a plain isolate, not in `<Code>`.
 * Reported to the lead; remove this line when the header is fixed.
 */
const KNOWN_GAPS = '.print-firm-line';

/** A key chip such as F9 or Ctrl+K names a key, not a number. */
const KEY_CHIP = /^(?:F\d{1,2}|Ctrl\+?\S*|Alt\+?\S*|Esc|Enter)$/;

/** One digit that is outside a number context: the text around it and the element it is in. */
export interface BareDigit {
  text: string;
  tag: string;
  className: string;
}

/** The digits (0-9 and the Eastern shapes) of a text node that sit outside a number context. */
export function bareDigits(root: ParentNode): BareDigit[] {
  const found: BareDigit[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const text = node.textContent ?? '';
    if (!/[0-9۰-۹٠-٩]/.test(text)) continue;
    const parent = node.parentElement;
    if (parent === null) continue;
    if (parent.closest(NUMBER_CONTEXT) !== null) continue;
    if (parent.closest(NAME_CONTEXT) !== null) continue;
    if (parent.closest(KNOWN_GAPS) !== null) continue;
    if (parent.closest('[aria-hidden="true"]') !== null) continue;
    if (KEY_CHIP.test(text.trim())) continue;
    found.push({
      text: text.replace(/[⁦-⁩]/g, '').trim().slice(0, 80),
      tag: parent.tagName.toLowerCase(),
      className: parent.className,
    });
  }
  return found;
}
