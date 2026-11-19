// A short code in an item, packing or staff box (PRD FR-10.2, FR-7.38, FR-7.39, Code Standards 4.2). The code of
// these records is a type number (004, 005, 006) and a running number with no padding, so the running number
// alone is a short code: `1` is the record 0041 in an item box. `1` can also be the start of `10`, so a short
// code is never taken while it is typed; it is taken on Enter (the list's best match) or on leaving the field,
// and on leaving only the record that really has that running number is taken, never one that merely starts
// with the digits. The master search already ranks the exact code and the exact running number first, so
// one read of it is enough. There is no money here.

import { searchMasters } from './api';
import type { SearchHit, SearchScope } from './types';

/** The lists whose codes are a type number and a running number: items 004, packings 005, staff 006 (PRD FR-10.2). */
const SHORT_CODE_SCOPES: ReadonlySet<SearchScope> = new Set<SearchScope>([
  'item',
  'packing',
  'staff',
]);

/** A type number is three digits (PRD FR-10.2); what follows it is the running number. */
const TYPE_NUMBER_DIGITS = 3;

/** Only digits are a code or a short code (PRD FR-7.39). */
const DIGITS = /^\d+$/;

/** The exact record is the first of the answer; a few rows are read in case an older one has not gone. */
const ANSWER_ROWS = 10;

/** True for the lists in which the running number of a code is a short code. */
export function hasShortCodes(scope: SearchScope): boolean {
  return SHORT_CODE_SCOPES.has(scope);
}

/** True when the digits are exactly this record's code, or exactly its running number. */
export function namesTheRecord(hit: SearchHit, typed: string): boolean {
  if (hit.code === typed) return true;
  return hit.code.length > TYPE_NUMBER_DIGITS && hit.code.slice(TYPE_NUMBER_DIGITS) === typed;
}

/**
 * The active record that the typed digits name, by its code or its running number, or null when none does
 * (also for text that is not digits, and for a list with no short codes). Only reads.
 */
export async function resolveShortCode(
  scope: SearchScope,
  typed: string,
): Promise<SearchHit | null> {
  const text = typed.trim();
  if (!hasShortCodes(scope) || !DIGITS.test(text)) return null;
  const answer = await searchMasters({ scope, text, limit: ANSWER_ROWS });
  return answer.find((hit) => hit.is_active && namesTheRecord(hit, text)) ?? null;
}
