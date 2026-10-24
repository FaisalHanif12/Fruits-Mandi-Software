// The paged answers of the list commands (plan 17b, PRD FR-19.11, Code Standards B31) as a fake
// backend gives them: a page of the rows, the count of the whole filter and the totals over the
// whole filter. A test double only: the real totals are the backend's, proved by its own tests and
// by the fixtures (the `*-real-*` tests). A double adds no money (the frontend never does, F3): its
// money totals are 0 unless a test passes the total it wants to see on screen.

import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriListRow } from '../../src/bindings/BikriListRow';
import type { PairRow } from '../../src/bindings/PairRow';
import type { PairsList } from '../../src/bindings/PairsList';
import type { PukhtaRegister } from '../../src/bindings/PukhtaRegister';
import type { PukhtaRegisterRow } from '../../src/bindings/PukhtaRegisterRow';
import type { RuleRow } from '../../src/bindings/RuleRow';
import type { RulesList } from '../../src/bindings/RulesList';
import type { MasterPage, MasterRecord } from '../../src/features/masters/types';
import { paisaForTest } from './paisa';

/** The page size the backend uses when `limit` is 0 (PRD FR-19.11). */
export const DEFAULT_PAGE = 100;

/** One page of `all`: `limit` rows from `offset`; a limit of 0 means the default page of 100. */
export function sliceOf<T>(all: readonly T[], offset = 0, limit = 0): T[] {
  const size = limit === 0 ? DEFAULT_PAGE : limit;
  return all.slice(offset, offset + size);
}

export function pairsOf(all: readonly PairRow[], offset = 0, limit = 0): PairsList {
  return {
    rows: sliceOf(all, offset, limit),
    total_count: all.length,
    totals: { use_count: all.reduce((sum, row) => sum + row.use_count, 0) },
  };
}

export const EMPTY_PAIRS: PairsList = pairsOf([]);

export function bikriListOf(
  all: readonly BikriListRow[],
  offset = 0,
  limit = 0,
  totals: Partial<BikriList['totals']> = {},
): BikriList {
  // Cancelled postings are left out of the sums and kept in the count (backend, plan 17b T1).
  const live = all.filter((row) => row.status !== 'cancelled');
  return {
    rows: sliceOf(all, offset, limit),
    total_count: all.length,
    totals: {
      qty_received: live.reduce((sum, row) => sum + row.qty_received, 0),
      qty_sold: live.reduce((sum, row) => sum + row.qty_sold, 0),
      gross: paisaForTest(0),
      net: paisaForTest(0),
      buyer_total: paisaForTest(0),
      ...totals,
    },
  };
}

export function pukhtaRegisterOf(
  all: readonly PukhtaRegisterRow[],
  offset = 0,
  limit = 0,
  totals: Partial<PukhtaRegister['totals']> = {},
): PukhtaRegister {
  return {
    rows: sliceOf(all, offset, limit),
    total_count: all.length,
    // As the backend: with no side chosen the totals are null (buyers and sellers would be added
    // together); a test that chooses a side gives the totals it wants to see.
    totals: { qty: null, amount: null, commission: null, open: null, ...totals },
  };
}

export function mastersPageOf(all: readonly MasterRecord[], offset = 0, limit = 0): MasterPage {
  return {
    rows: sliceOf(all, offset, limit),
    total_count: all.length,
    totals: { active_count: all.filter((row) => row.is_active).length },
  };
}

export function rulesListOf(all: readonly RuleRow[], offset = 0, limit = 0): RulesList {
  return {
    rows: sliceOf(all, offset, limit),
    total_count: all.length,
    totals: { active_count: all.filter((row) => row.is_active).length },
  };
}
