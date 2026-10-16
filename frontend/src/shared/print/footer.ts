// The page footer of a printed document: the print time and "Page 1 of 2" (PRD FR-19.3,
// reference.md). It is drawn by the `@bottom-center` page margin box of print.css from the custom
// property `--print-footer`, which holds a CSS `content` value with the page counters. The label
// words come from Fluent in the print language, so the print never has words typed in code.
//
// The value is set with the CSS object model on the root element. An inline <style> element
// would be refused by the production Content Security Policy (style-src 'self'); the object
// model is not.

/** The custom property the page margin boxes read. */
export const FOOTER_PROPERTY = '--print-footer';

/** The Unicode isolation marks Fluent puts around placeables. They have no place in a CSS string. */
const FIRST_ISOLATE = 0x2066;
const POP_CODE = 0x2069;
const ISOLATION_MARKS = new RegExp(
  `[${String.fromCodePoint(FIRST_ISOLATE)}-${String.fromCodePoint(POP_CODE)}]`,
  'g',
);

/** Left-to-right isolate and pop: a number or a date inside a margin box never flips (NFR-URD-13). */
const LTR_ISOLATE = String.fromCodePoint(FIRST_ISOLATE);
const POP_ISOLATE = String.fromCodePoint(POP_CODE);

/** `text` as a CSS string literal: quotes, backslashes and line breaks are escaped. */
export function cssString(text: string): string {
  const clean = text.replace(/[\r\n]+/g, ' ');
  return `"${clean.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

/** Fluent's own isolation marks removed from a label, so only the ones this file adds are left. */
export function stripIsolates(text: string): string {
  return text.replace(ISOLATION_MARKS, '');
}

export interface FooterWords {
  /** "Printed" or "پرنٹ کا وقت". */
  printed: string;
  /** "Page" or "صفحہ". */
  page: string;
  /** "of" or "از". */
  of: string;
}

/**
 * The CSS `content` value of the footer:
 * `"Printed 10-10-2026 17:40 · Page " counter(page) " of " counter(pages)`.
 * With Urdu digits on, the counters use the Urdu digit style, like every other number. Without
 * pages (the roll) it is only `"Printed 10-10-2026 17:40"`.
 */
export function footerContent(
  words: FooterWords,
  printedAt: string,
  urduDigits: boolean,
  withPages = true,
): string {
  const style = urduDigits ? ', persian' : '';
  // The 80 mm roll is one tall page, so "Page 1 of 1" would say nothing: only the print time.
  if (!withPages) {
    return cssString(`${stripIsolates(words.printed)} ${LTR_ISOLATE}${printedAt}${POP_ISOLATE}`);
  }
  return [
    // A page margin box has no <bdi>, so the print time is wrapped in a left-to-right isolate.
    // Without it "10-10-2026" is written "2026-10-10" inside an Urdu line.
    cssString(
      `${stripIsolates(words.printed)} ${LTR_ISOLATE}${printedAt}${POP_ISOLATE} · ${stripIsolates(words.page)} `,
    ),
    `counter(page${style})`,
    cssString(` ${stripIsolates(words.of)} `),
    `counter(pages${style})`,
  ].join(' ');
}

export interface PrintRoot {
  language: string;
  dir: string;
  footer: string;
}

/**
 * Sets the language, the direction and the footer on the root element of the hidden print window,
 * so the page margin boxes (which inherit from the root) use the print language's font and
 * direction. It returns a function that puts the old values back. Only the hidden window calls
 * this; the app window keeps the app language (Code Standards F37).
 */
export function applyPrintRoot(root: PrintRoot): () => void {
  const element = document.documentElement;
  const before = {
    lang: element.lang,
    dir: element.dir,
    footer: element.style.getPropertyValue(FOOTER_PROPERTY),
  };
  element.lang = root.language;
  element.dir = root.dir;
  element.style.setProperty(FOOTER_PROPERTY, root.footer);
  return () => {
    element.lang = before.lang;
    element.dir = before.dir;
    if (before.footer === '') element.style.removeProperty(FOOTER_PROPERTY);
    else element.style.setProperty(FOOTER_PROPERTY, before.footer);
  };
}
