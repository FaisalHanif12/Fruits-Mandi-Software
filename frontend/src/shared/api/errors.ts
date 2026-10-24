// Turns backend error codes into Fluent message IDs (Code Standards 4.3, NFR-USE-14).
// ERR_QTY_EXCEEDS_LOT becomes err-qty-exceeds-lot. The message lives in i18n/<lang>/errors.ftl.

import type { WireCode } from '../../bindings/WireCode';
import type { UiError } from '../../bindings/UiError';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { isoDayToDisplay } from '../format/dates';
import { formatPaisaValue, groupDigits, type Grouping } from '../format/money';
import { formatBps } from '../format/percent';
import { CommandError, toUiError } from './invoke';

/**
 * The Fluent message ID for an error code. A backup command may also send a code of its own
 * (`BackupCode`, plan 18); both kinds are the text `ERR_...` and are read the same way.
 */
export function errorMessageId(code: WireCode): string {
  return `err-${code.replace(/^ERR_/, '').toLowerCase().replaceAll('_', '-')}`;
}

/**
 * The message used when the backend sends a code this build has no words for (for example a
 * newer backend). The user still gets plain words, never the raw code (NFR-USE-14).
 */
export const FALLBACK_ERROR_MESSAGE_ID = errorMessageId('ERR_INTERNAL');

/** The UiError inside anything a query or mutation failed with. */
export function uiErrorOf(error: unknown): UiError {
  return error instanceof CommandError ? error.uiError : toUiError(error);
}

/** True when the problem points at one field or grid cell, not at the whole form. */
function isFieldError(item: UiErrorItem): item is UiErrorItem & { field: string } {
  return typeof item.field === 'string' && item.field !== '';
}

/** The problems of one error, split into whole-form ones and ones that point at a field. */
export interface SplitErrors {
  /** No field: the whole form or screen. These carry the reference number. */
  general: UiErrorItem[];
  /** Field path (for example `lots[0].sales[2].qty`) to the problems on that field. */
  byField: ReadonlyMap<string, UiErrorItem[]>;
}

export function splitErrors(error: UiError): SplitErrors {
  const general: UiErrorItem[] = [];
  const byField = new Map<string, UiErrorItem[]>();
  for (const item of error.errors) {
    if (!isFieldError(item)) {
      general.push(item);
      continue;
    }
    const onField = byField.get(item.field);
    if (onField === undefined) {
      byField.set(item.field, [item]);
    } else {
      onField.push(item);
    }
  }
  return { general, byField };
}

/** The problems on one field path. The path must match exactly, `lots[0]` is not `lots[1]`. */
export function errorsForField(error: UiError | null | undefined, field: string): UiErrorItem[] {
  if (error === null || error === undefined) return [];
  return error.errors.filter((item) => item.field === field);
}

/**
 * The problems that no field on the screen will show: the whole-form ones, and any whose field
 * is not in `fields`. A screen passes the field paths it draws, so a problem can never vanish
 * because its field is not there (Code Standards 4.3).
 */
export function errorsOutsideFields(error: UiError, fields: readonly string[]): UiErrorItem[] {
  const shown = new Set(fields);
  return error.errors.filter((item) => !isFieldError(item) || !shown.has(item.field));
}

const BPS_SUFFIX = '_bps';
const PAISA_SUFFIX = '_paisa';

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
 * pass as they are, except a date written YYYY-MM-DD, which is shown as DD-MM-YYYY. A rate in basis points (`total_bps`, 9000) also arrives as the percent a
 * person reads, under the same name without `_bps` (`total`, "90"), written from its digits.
 * An amount whose name ends in `_paisa` (`cash_after_paisa`, 500000) is money: it is written with
 * formatPaisa ("5,000"), under its own name and under the name without `_paisa`
 * (`cash_after`), so a message never shows raw paisa (plan 12 contract 4).
 */
export function messageVars(
  values: UiErrorItem['values'],
  grouping: Grouping,
): Record<string, string> {
  const vars: Record<string, string> = {};
  for (const [key, value] of Object.entries(values)) {
    if (typeof value === 'number' && key.endsWith(PAISA_SUFFIX) && Number.isSafeInteger(value)) {
      const money = formatPaisaValue(value, { grouping });
      vars[key] = money;
      vars[key.slice(0, -PAISA_SUFFIX.length)] = money;
      continue;
    }
    // A value that is exactly a real day, YYYY-MM-DD, is shown as DD-MM-YYYY (Code Standards G9);
    // any other text passes as it is.
    vars[key] = typeof value === 'number' ? groupedNumber(value, grouping) : isoDayToDisplay(value);
    if (typeof value === 'number' && key.endsWith(BPS_SUFFIX) && Number.isSafeInteger(value)) {
      vars[key.slice(0, -BPS_SUFFIX.length)] = formatBps(value);
    }
  }
  return vars;
}
