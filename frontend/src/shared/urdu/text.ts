// Plain text helpers for typing names in two scripts (PRD FR-7.22, FR-7.24). No money here.

const LATIN_RUN = /[A-Za-z]+$/;

/**
 * The English letters at the very end of the text: the word being typed, not yet turned into
 * Urdu. "طارق mehm" gives "mehm". Empty when the text ends with anything else.
 */
export function trailingLatinRun(text: string): string {
  return LATIN_RUN.exec(text)?.[0] ?? '';
}

/** The text without its trailing English word. */
export function withoutTrailingLatinRun(text: string): string {
  return text.slice(0, text.length - trailingLatinRun(text).length);
}

/** "arshad multani" becomes "Arshad Multani": a capital first letter, the rest as typed (FR-7.24). */
export function capitalizeWords(text: string): string {
  return text
    .split(/(\s+)/)
    .map((part) => (/^\s+$/.test(part) ? part : part.charAt(0).toUpperCase() + part.slice(1)))
    .join('');
}

/** The words of a name, split on spaces. */
export function wordsOf(text: string): string[] {
  return text.split(/\s+/).filter((word) => word !== '');
}
