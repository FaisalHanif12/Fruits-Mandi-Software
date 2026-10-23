// The addresses of the capital and statement screens (PRD 17.4, FR-17.9): kept apart from the
// screens so the router can read them without loading a screen (Code Standards F36).
//
// - `/partners/capital?type=capital_out&partner=2`: the type of the entry and the partner whose
//   capital is shown. A value that is not valid is left out, and the screen starts without it.
// - `/partners/statement?partner=2&from=01-10-2026&to=31-10-2026`: the partner and the days.

/** The types in the type list of the سرمایہ screen: invested and taken back (PRD 17.4). */
export type CapitalType = 'capital_in' | 'capital_out';
export const CAPITAL_TYPES: readonly CapitalType[] = ['capital_in', 'capital_out'];

export interface CapitalSearch {
  type?: CapitalType;
  partner?: number;
}

function idOf(value: unknown): number | null {
  const id = typeof value === 'number' ? value : typeof value === 'string' ? Number(value) : NaN;
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

export function parseCapitalSearch(search: Record<string, unknown>): CapitalSearch {
  const type = CAPITAL_TYPES.find((one) => one === search['type']);
  const partner = idOf(search['partner']);
  return {
    ...(type === undefined ? {} : { type }),
    ...(partner === null ? {} : { partner }),
  };
}

export interface StatementSearch {
  partner?: number;
  from?: string;
  to?: string;
}

function textOf(value: unknown): string | null {
  return typeof value === 'string' && value.trim() !== '' ? value.trim() : null;
}

export function parseStatementSearch(search: Record<string, unknown>): StatementSearch {
  const partner = idOf(search['partner']);
  const from = textOf(search['from']);
  const to = textOf(search['to']);
  return {
    ...(partner === null ? {} : { partner }),
    ...(from === null ? {} : { from }),
    ...(to === null ? {} : { to }),
  };
}
