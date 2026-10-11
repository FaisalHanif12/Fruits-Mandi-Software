// The built-in phonetic Urdu keyboard (PRD FR-7.21, Tech Stack 4.4, NFR-URD-17). English keys type
// Urdu letters, with no Windows setting: `a` gives ا, `b` ب, `p` پ, `t` ت, `T` ٹ, `h` ہ, `H` ھ. It
// follows the idea of the widely used CRULP phonetic layout: the key is the letter's sound, and a
// capital gives the heavier or rarer sibling (t ت and T ٹ, d د and D ڈ, s س and S ص). The rest of
// the table is ours and goes to the Urdu reviewer with the other new words (NFR-URD-4).
//
// Only Latin letters and a few marks are mapped. Digits, spaces and the Urdu letters of a Windows
// Urdu keyboard pass through unchanged (PRD FR-7.21).

/** Lower-case keys. */
const LOWER: Readonly<Record<string, string>> = {
  a: 'ا',
  b: 'ب',
  c: 'چ',
  d: 'د',
  e: 'ع',
  f: 'ف',
  g: 'گ',
  h: 'ہ',
  i: 'ی',
  j: 'ج',
  k: 'ک',
  l: 'ل',
  m: 'م',
  n: 'ن',
  o: 'و',
  p: 'پ',
  q: 'ق',
  r: 'ر',
  s: 'س',
  t: 'ت',
  u: 'ء',
  v: 'ط',
  w: 'و',
  x: 'ش',
  y: 'ے',
  z: 'ز',
};

/** Capital keys. A capital with no entry gives the same letter as its lower-case key. */
const UPPER: Readonly<Record<string, string>> = {
  A: 'آ',
  C: 'ث',
  D: 'ڈ',
  G: 'غ',
  H: 'ھ',
  I: 'ئ',
  J: 'ض',
  K: 'خ',
  N: 'ں',
  O: 'ۃ',
  R: 'ڑ',
  S: 'ص',
  T: 'ٹ',
  U: 'ؤ',
  V: 'ظ',
  X: 'ژ',
  Y: 'ی',
  Z: 'ذ',
};

/** Marks: comma, semicolon and question mark have Urdu forms. */
const MARKS: Readonly<Record<string, string>> = { ',': '،', ';': '؛', '?': '؟' };

/** The Urdu text for a typed key, or null when the key passes through unchanged. */
export function mapPhoneticKey(key: string): string | null {
  if (key.length !== 1) return null;
  const mark = MARKS[key];
  if (mark !== undefined) return mark;
  const upper = UPPER[key];
  if (upper !== undefined) return upper;
  const lower = LOWER[key] ?? LOWER[key.toLowerCase()];
  return lower ?? null;
}

export interface KeyMapEntry {
  /** The key to press, for example `T`. */
  key: string;
  /** The Urdu letter it types. */
  letter: string;
}

/** The on-screen key map: every key that types a letter, lower case first, then capitals. */
export const KEY_MAP: readonly KeyMapEntry[] = [
  ...Object.keys(LOWER).map((key) => ({ key, letter: LOWER[key] ?? '' })),
  ...Object.keys(UPPER).map((key) => ({ key, letter: UPPER[key] ?? '' })),
  ...Object.keys(MARKS).map((key) => ({ key, letter: MARKS[key] ?? '' })),
];
