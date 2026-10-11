// The three ways to type in an Urdu text field (PRD FR-7.21), switched with Alt+U.

/**
 * - `roman`: English letters become Urdu words (رومن سے اردو), the default in Urdu text fields.
 * - `keyboard`: the built-in phonetic keyboard types Urdu letters (اردو کی بورڈ).
 * - `english`: no conversion (انگریزی).
 */
export type TypingMode = 'roman' | 'keyboard' | 'english';

const ORDER: readonly TypingMode[] = ['roman', 'keyboard', 'english'];

/** The mode Alt+U moves to. */
export function nextTypingMode(mode: TypingMode): TypingMode {
  return ORDER[(ORDER.indexOf(mode) + 1) % ORDER.length] ?? 'roman';
}

/** Fluent IDs of the badge text and of the full name of each mode. */
export const MODE_BADGE_ID: Record<TypingMode, string> = {
  roman: 'translit-mode-roman',
  keyboard: 'translit-mode-urdu',
  english: 'translit-mode-english',
};

export const MODE_FULL_ID: Record<TypingMode, string> = {
  roman: 'translit-mode-full-roman',
  keyboard: 'translit-mode-full-urdu',
  english: 'translit-mode-full-english',
};
