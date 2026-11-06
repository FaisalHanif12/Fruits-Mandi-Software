// The quality of a sale line (درجہ, PRD FR-12.42) as words, for every screen and print that writes
// an item: "سیب اول" / "Apple First". A line with no quality writes the item alone, so an old
// print reads exactly as before. The three words and the way they follow the item are Fluent
// messages (`glossary-darja-*`, `common-item-grade`), so each language says them its own way.

import type { SaleGrade } from '../../bindings/SaleGrade';

/** The Fluent message of each grade: اول, دوم, سوم (First, Second, Third). */
export const GRADE_WORD_ID: Readonly<Record<SaleGrade, string>> = {
  first: 'glossary-darja-awwal',
  second: 'glossary-darja-doyam',
  third: 'glossary-darja-soyam',
};

/** Turns a message id and its variables into text in the language in use. */
type Say = (id: string, vars?: Record<string, string>) => string;

/** The item, with its quality word after it when the line has one. */
export function gradedItem(say: Say, item: string, grade: SaleGrade | undefined): string {
  if (grade === undefined) return item;
  return say('common-item-grade', { item, grade: say(GRADE_WORD_ID[grade]) });
}
