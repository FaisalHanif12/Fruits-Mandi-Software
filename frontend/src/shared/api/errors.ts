// Turns backend error codes into Fluent message IDs (Code Standards 4.3, NFR-USE-14).
// ERR_QTY_EXCEEDS_LOT becomes err-qty-exceeds-lot. The message lives in i18n/<lang>/errors.ftl.

import type { ErrorCode } from '../../bindings/ErrorCode';
import type { UiError } from '../../bindings/UiError';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { groupDigits, type Grouping } from '../format/money';
import { CommandError, toUiError } from './invoke';

/** The Fluent message ID for an error code. */
export function errorMessageId(code: ErrorCode): string {
  return `err-${code.replace(/^ERR_/, '').toLowerCase().replaceAll('_', '-')}`;
}

/** The UiError inside anything a query or mutation failed with. */
export function uiErrorOf(error: unknown): UiError {
  return error instanceof CommandError ? error.uiError : toUiError(error);
}

/** A whole number as grouped digits, with a minus sign when it is below zero. */
function groupedNumber(value: number, grouping: Grouping): string {
  if (!Number.isSafeInteger(value)) return String(value);
  const grouped = groupDigits(String(Math.abs(value)), grouping);
  return value < 0 ? `-${grouped}` : grouped;
}

/**
 * The Fluent variables of an error, ready to show. Error values are display units (rupees,
 * نگ, kg, characters), never paisa (plan 02 contract 4.6). Whole numbers are grouped with the
 * app's grouping, so 10000000000 reads "10,000,000,000" or "10,00,00,00,000" in both languages
 * and Fluent never formats a number by locale (PRD FR-7.32). Text values, such as a date,
 * pass as they are.
 */
export function messageVars(
  values: UiErrorItem['values'],
  grouping: Grouping,
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(values).map(([key, value]) => [
      key,
      typeof value === 'number' ? groupedNumber(value, grouping) : value,
    ]),
  );
}
