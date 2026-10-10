// Digits on prints (NFR-URD-14, PRD FR-7.32, Tech Stack 4.5).
//
// The screen always shows English digits (0-9) in both languages. Only an Urdu print may use
// Urdu digits (۰-۹), and only when the Printing setting turns them on. The default is off.

/** Which digits a print uses. 'latin' is the default (PRD Section 21, Printing). */
export type PrintDigits = 'latin' | 'urdu';

export const DEFAULT_PRINT_DIGITS: PrintDigits = 'latin';

/** Extended Arabic-Indic digits, the ones Urdu uses: ۰۱۲۳۴۵۶۷۸۹ (U+06F0 to U+06F9). */
const URDU_DIGITS = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'] as const;

/**
 * Changes every English digit in the text to its Urdu digit. Separators and other characters
 * stay as they are. Example: "13,16,040" -> "۱۳,۱۶,۰۴۰".
 */
export function toUrduDigits(text: string): string {
  return text.replace(/[0-9]/g, (digit) => URDU_DIGITS[Number(digit)] ?? digit);
}

/**
 * The digits a print shows, from text already formatted (for example by formatPaisa).
 * Example: printDigits("12,000", 'urdu') -> "۱۲,۰۰۰"; with 'latin' it is unchanged.
 */
export function printDigits(text: string, digits: PrintDigits = DEFAULT_PRINT_DIGITS): string {
  return digits === 'urdu' ? toUrduDigits(text) : text;
}
