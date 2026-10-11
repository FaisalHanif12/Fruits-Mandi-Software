// On-screen hints for names (Code Standards 4.1 layer 1, PRD FR-7.28): an Urdu name takes Urdu
// letters and an English name takes English letters. These hints only help the user at once; they
// are not trusted. The backend checks every name again and is the final word (ERR_NAME_UR_SCRIPT,
// ERR_NAME_EN_SCRIPT). Digits, spaces and `. - ' ( ) & /` are allowed in both scripts.

export type Script = 'ur' | 'en';

const LATIN_LETTER = /[A-Za-z]/;
/** The Arabic-script blocks Urdu uses, including presentation forms. */
const URDU_LETTER = new RegExp('[\\u0600-\\u06FF\\u0750-\\u077F\\uFB50-\\uFDFF\\uFE70-\\uFEFF]');

/**
 * The ID of the message to show when `text` has letters of the other script, else null. An Urdu
 * name with an English letter gives `err-name-ur-script`, an English name with an Urdu letter
 * gives `err-name-en-script`.
 */
export function scriptHintId(text: string, script: Script): string | null {
  if (script === 'ur') return LATIN_LETTER.test(text) ? 'err-name-ur-script' : null;
  return URDU_LETTER.test(text) ? 'err-name-en-script' : null;
}

/** The ID of the message for an empty required field, else null. Spaces alone are empty. */
export function requiredHintId(text: string): string | null {
  return text.trim() === '' ? 'err-required' : null;
}
