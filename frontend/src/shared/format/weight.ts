// A weight from the backend shown as kilograms (PRD FR-16.7, Code Standards 4.2): whole grams in,
// "2.5" out. It is text work on the digits, with no arithmetic on the number, like `formatPaisa`.
// Examples: 2500 -> "2.5", 125 -> "0.125", 3000 -> "3", 0 -> "0".

const GRAMS_PER_KG_DIGITS = 3;

export function formatGrams(grams: number): string {
  if (!Number.isSafeInteger(grams) || grams < 0) return String(grams);
  const digits = String(grams).padStart(GRAMS_PER_KG_DIGITS + 1, '0');
  const whole = digits.slice(0, -GRAMS_PER_KG_DIGITS);
  const fraction = digits.slice(-GRAMS_PER_KG_DIGITS).replace(/0+$/, '');
  return fraction === '' ? whole : `${whole}.${fraction}`;
}
