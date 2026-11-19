// What the address of the party ledgers screen can carry (PRD FR-12.43, plan 66): which kind of party
// the list is about (`side`), one party (`party`, level 2) and one date of his (`date`, level 3).
// The three levels are three addresses, so Esc and the back buttons are plain steps back, a language
// switch keeps the level, and another screen can open a party's ledger. Anything that does not read
// is left out, so a bad address shows level 1.

import type { LedgerSide } from '../../bindings/LedgerSide';
import { isIsoDay } from '../../shared/format';

export interface LedgerSearch {
  side?: LedgerSide;
  /** The account of the party whose statement is open. */
  party?: number;
  /** The date of the slip that is open, `YYYY-MM-DD`; only with a party. */
  date?: string;
  /**
   * The last day of the 30 days the statement shows, `YYYY-MM-DD`; only with a party and no `date`
   * (PRD FR-12.47). Nothing: the window ends today.
   */
  end?: string;
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

function positiveId(value: unknown): number | undefined {
  const number = typeof value === 'string' && /^\d{1,12}$/.test(value) ? Number(value) : value;
  return typeof number === 'number' && Number.isSafeInteger(number) && number > 0
    ? number
    : undefined;
}

/** Reads the address. A date without a party is dropped: there is no slip without a party. */
export function parseLedgerSearch(raw: Record<string, unknown>): LedgerSearch {
  const side: LedgerSide | undefined =
    raw['side'] === 'buyer' || raw['side'] === 'seller' ? raw['side'] : undefined;
  const party = positiveId(raw['party']);
  const date =
    party !== undefined && typeof raw['date'] === 'string' && ISO_DATE.test(raw['date'])
      ? raw['date']
      : undefined;
  const end =
    party !== undefined && isIsoDay(raw['end']) && date === undefined ? raw['end'] : undefined;
  return {
    ...(side === undefined ? {} : { side }),
    ...(party === undefined ? {} : { party }),
    ...(date === undefined ? {} : { date }),
    ...(end === undefined ? {} : { end }),
  };
}
